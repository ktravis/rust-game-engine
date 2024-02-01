use anyhow::anyhow;
use controlset_derive::ControlSet;
use glam::{vec3, Mat4, Quat, Vec2};
use rust_game_engine::assets::AssetManager;
use rust_game_engine::color::Color;
use rust_game_engine::font::{FontAtlas, LayoutOptions};
use rust_game_engine::geom::{cube, quad, ModelVertexData, Point};
use rust_game_engine::input::{Axis, Button, ControlSet, InputManager, Toggle};
use rust_game_engine::mesh_manager::{GeometryBuffers, MeshManager, MeshOffsets};
use rust_game_engine::renderer::{
    BasicRenderPipeline, DisplayMode, InstancedRenderPipeline, ModelInstanceData,
    OffscreenFramebuffer, RawInstanceData, RenderPassOptions, RenderTarget, TextDisplayOptions,
};
use rust_game_engine::shader::Shader;
use rust_game_engine::sprite_manager::{SpriteManager, SpriteRef};
use rust_game_engine::texture::Texture;
use rust_game_engine::{resources, transform};
use std::cell::Cell;
use std::io::Read;
use std::path::Path;
use std::rc::Rc;
use ttf_parser::Face;

pub const WINDOW_DIM: Point = Point { x: 960, y: 720 };
const TARGET_FRAMERATE: u64 = 60;

#[derive(ControlSet)]
struct Controls {
    #[bind((Key::A, Key::D), MouseMotionX)]
    x: Axis,
    #[bind((Key::W, Key::S), MouseMotionY)]
    y: Axis,
    #[bind((Key::F, Key::R))]
    scale: Axis,
    #[bind(Key::Escape)]
    quit: Button,
    #[bind(Key::P)]
    next: Button,
    #[bind(Key::O)]
    add: Button,
    #[bind(Key::Slash)] // aka question mark
    show_help: Toggle,
}

struct RenderPipelines {
    basic: BasicRenderPipeline,
    model: InstancedRenderPipeline<ModelVertexData, RawInstanceData>,
    text: InstancedRenderPipeline<ModelVertexData, RawInstanceData>,
}

#[derive(Debug, Default, Clone)]
struct ShaderSource {
    dirty: bool,

    default_vert: String,
    default_frag: String,

    model_vert: String,
    model_frag: String,

    text_vert: String,
    text_frag: String,
}

#[derive(Default)]
struct GameAssets {
    shader_sources: ShaderSource,
    font_atlas: FontAtlas,
    sprites: SpriteManager,
    meshes: MeshManager<ModelVertexData>,
}

#[derive(Debug, Default)]
struct FrameTiming {
    /// Last frame start time in seconds
    frame_start: f64,
    /// Delta from last frame start in seconds
    delta: f32,
    frame_counter: usize,
    frame_timer: f32,
    sampled_fps: f32,
}

impl FrameTiming {
    pub fn update(&mut self, current_time: f64) {
        self.frame_counter += 1;
        self.delta = if self.frame_start < 0.0 {
            1.0 / 60.0
        } else {
            (current_time - self.frame_start) as f32
        };
        self.frame_timer += self.delta;
        self.frame_start = current_time;
        if self.frame_counter >= 50 {
            self.sampled_fps = self.frame_counter as f32 / self.frame_timer;
            self.frame_counter = 0;
            self.frame_timer = 0.0;
        }
    }
}

pub struct Stage {
    pub(crate) input: InputManager<Controls>,
    pub(crate) asset_manager: AssetManager<GameAssets>,
    pub(crate) frame_timing: FrameTiming,
    target_frame_duration: f64,
    backbuffer_target: OffscreenFramebuffer,
    render_pipelines: RenderPipelines,
    quad_mesh: Rc<Cell<MeshOffsets>>,
    cube_mesh: Rc<Cell<MeshOffsets>>,
    geometry_buffers: GeometryBuffers<ModelVertexData>,

    sprite_atlas_texture: Texture,
    crate_texture: Texture,
    font_texture: Texture,

    s: SpriteRef,
    frame_index: usize,
    sprite_pos: Vec<Point>,
    xy: Vec2,
    angle: f32,
    render_scale: f32,
    camera_offset: Vec2,
}

const SCALE: u32 = 1;

// fn shader_error_mapper(err: miniquad::ShaderError) -> anyhow::Error {
//     use miniquad::ShaderError::*;
//     match err {
//         CompilationError {
//             shader_type,
//             error_message,
//         } => anyhow!(
//             "{:?} shader compilation failed: {}",
//             shader_type,
//             error_message
//         ),
//         LinkError(msg) => anyhow!("linking failed: {}", msg),
//         FFINulError(_) => anyhow!("shader has a null byte in it!"),
//     }
// }

fn create_render_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    color_format: wgpu::TextureFormat,
    depth_format: Option<wgpu::TextureFormat>,
    vertex_layouts: &[wgpu::VertexBufferLayout],
    shader: wgpu::ShaderModuleDescriptor,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(shader);

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: vertex_layouts,
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: color_format,
                blend: Some(wgpu::BlendState {
                    alpha: wgpu::BlendComponent::REPLACE,
                    color: wgpu::BlendComponent::REPLACE,
                }),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
            polygon_mode: wgpu::PolygonMode::Fill,
            // Requires Features::DEPTH_CLIP_CONTROL
            unclipped_depth: false,
            // Requires Features::CONSERVATIVE_RASTERIZATION
            conservative: false,
        },
        depth_stencil: depth_format.map(|format| wgpu::DepthStencilState {
            format,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    })
}

fn build_render_pipelines(
    ctx: &mut GraphicsContext,
    shader_sources: &ShaderSource,
) -> anyhow::Result<RenderPipelines> {
    let basic_shader = Shader::new(
        ctx,
        &shader_sources.default_vert,
        &shader_sources.default_frag,
        vec!["tex".into()],
    )
    .map_err(shader_error_mapper)?;
    let model_shader = Shader::new(
        ctx,
        &shader_sources.model_vert,
        &shader_sources.model_frag,
        vec!["tex".into()],
    )
    .map_err(shader_error_mapper)?;
    let text_shader = Shader::new(
        ctx,
        &shader_sources.text_vert,
        &shader_sources.text_frag,
        vec!["tex".into()],
    )
    .map_err(shader_error_mapper)?;
    Ok(RenderPipelines {
        basic: BasicRenderPipeline::new(ctx, basic_shader),
        model: InstancedRenderPipeline::new(ctx, model_shader),
        text: InstancedRenderPipeline::new(ctx, text_shader),
    })
}

impl Stage {
    pub fn new(ctx: &mut GraphicsContext) -> Stage {
        // check features
        {
            assert!(ctx.features().instancing, "instancing is not supported");
        }

        let mut asset_manager = AssetManager::new(GameAssets::default(), "./res/");

        let crate_texture = resources::texture_from_image(
            ctx,
            &image::open(Path::new("../../../res/images/crate.png"))
                .unwrap()
                .into_rgba8(),
        );

        asset_manager.track_file("./res/fonts/Ubuntu-M.ttf", |state, _, mut f| {
            let mut b = vec![];
            f.read_to_end(&mut b).unwrap();
            let face = Face::parse(&b, 0).unwrap();
            state.font_atlas = FontAtlas::new(face, Default::default()).unwrap();
        });

        let font_texture = {
            let font_atlas_image = asset_manager.font_atlas.image();
            resources::texture_from_image_with_params(
                ctx,
                font_atlas_image,
                miniquad::TextureParams {
                    format: miniquad::TextureFormat::RGBA8,
                    wrap: miniquad::TextureWrap::Clamp,
                    filter: miniquad::FilterMode::Linear,
                    width: font_atlas_image.width(),
                    height: font_atlas_image.height(),
                },
            )
        };

        asset_manager.track_glob("./res/sprites/*.aseprite", |state, path, f| {
            state.sprites.add_sprite_file(path.to_path_buf(), f);
        });
        // TODO: should have a way to not need this
        let sprite_atlas_texture = {
            asset_manager.sprites.maybe_rebuild();
            resources::texture_from_image(ctx, asset_manager.sprites.atlas_image())
        };

        asset_manager
            .track_file("./res/shaders/default.vert", |state, _, f| {
                state.shader_sources.dirty = true;
                state.shader_sources.default_vert = std::io::read_to_string(&f).unwrap();
            })
            .track_file("./res/shaders/default.frag", |state, _, f| {
                state.shader_sources.dirty = true;
                state.shader_sources.default_frag = std::io::read_to_string(&f).unwrap();
            })
            .track_file("./res/shaders/model.vert", |state, _, f| {
                state.shader_sources.dirty = true;
                state.shader_sources.model_vert = std::io::read_to_string(&f).unwrap();
            })
            .track_file("./res/shaders/model.frag", |state, _, f| {
                state.shader_sources.dirty = true;
                state.shader_sources.model_frag = std::io::read_to_string(&f).unwrap();
            })
            .track_file("./res/shaders/text.vert", |state, _, f| {
                state.shader_sources.dirty = true;
                state.shader_sources.text_vert = std::io::read_to_string(&f).unwrap();
            })
            .track_file("./res/shaders/text.frag", |state, _, f| {
                state.shader_sources.dirty = true;
                state.shader_sources.text_frag = std::io::read_to_string(&f).unwrap();
            });

        let render_pipelines = build_render_pipelines(ctx, &asset_manager.shader_sources).unwrap();

        let render_target_size_px: Point<u32> = (240 * SCALE, 180 * SCALE).into();
        let backbuffer_target = OffscreenFramebuffer::new(ctx, render_target_size_px);

        let sprite_pos = (0..10)
            .map(|_| {
                Point::new(
                    (rand::random::<u32>() % render_target_size_px.x) as i32,
                    (rand::random::<u32>() % render_target_size_px.y) as i32,
                )
            })
            .collect();
        let s = asset_manager.sprites.get_sprite_ref("guy").unwrap();
        let quad_mesh = asset_manager.meshes.add("quad", quad::mesh());
        let cube_mesh = asset_manager.meshes.add("cube", cube::mesh());
        let geometry_buffers = asset_manager.meshes.buffers(ctx);
        Stage {
            sprite_pos,
            input: Default::default(),
            camera_offset: Default::default(),
            xy: Vec2::default(),
            frame_index: 0,
            s,
            frame_timing: Default::default(),
            target_frame_duration: 1. / TARGET_FRAMERATE as f64,
            angle: 0.,
            render_scale: 1.,
            backbuffer_target,
            font_texture,
            crate_texture,
            sprite_atlas_texture,
            quad_mesh,
            cube_mesh,
            render_pipelines,
            geometry_buffers,
            asset_manager,
        }
    }

    pub fn init(&mut self) {}

    pub fn update(&mut self, ctx: &mut GraphicsContext) -> bool {
        if self.input.quit.just_pressed {
            return false;
        }

        if self.asset_manager.check_for_updates() {
            if self.asset_manager.shader_sources.dirty {
                self.asset_manager.shader_sources.dirty = false;
                match build_render_pipelines(ctx, &self.asset_manager.shader_sources) {
                    Ok(p) => self.render_pipelines = p,
                    Err(e) => {
                        println!("Pipeline recreation failed: {}", e);
                    }
                }
            }
            if self.asset_manager.sprites.maybe_rebuild() {
                self.sprite_atlas_texture =
                    resources::texture_from_image(ctx, self.asset_manager.sprites.atlas_image());
            }
            if self.asset_manager.meshes.needs_rebuild {
                self.geometry_buffers = self.asset_manager.meshes.buffers(ctx);
            }
        }

        self.xy.x += 100. * self.frame_timing.delta * self.input.x.value();
        self.xy.y += 100. * self.frame_timing.delta * self.input.y.value();
        self.render_scale = (self.render_scale
            + 20. * self.frame_timing.delta * self.input.scale.value())
        .clamp(0.5, 40.);

        if self.input.next.just_pressed() {
            self.frame_index =
                (self.frame_index + 1) % self.asset_manager.sprites.get_sprite(self.s).frames.len();
        }
        if self.input.add.is_down() {
            println!("{}", self.sprite_pos.len());
            for _ in 0..100 {
                self.sprite_pos.push(Point::new(
                    (rand::random::<u32>() % self.backbuffer_target.color.width) as i32,
                    (rand::random::<u32>() % self.backbuffer_target.color.height) as i32,
                ));
            }
        }
        self.angle += self.frame_timing.delta;

        // keep running
        true
    }

    pub fn draw(&mut self, ctx: &mut GraphicsContext) {
        {
            let mut r = self.render_pipelines.model.begin_pass(
                ctx,
                self.backbuffer_target.into(),
                &self.geometry_buffers,
                Some(vec![self.sprite_atlas_texture]),
                self.quad_mesh.clone(),
                RenderPassOptions::clear(Color::from(0x3f3f74ffu32)),
            );
            r.push_transform(Mat4::from_translation(self.camera_offset.extend(1.0)));

            let s = self.asset_manager.sprites.get_sprite(self.s);
            for xy in &self.sprite_pos {
                r.draw_sprite_frame(self.xy + xy.as_vec2(), s, self.frame_index);
            }
        }

        {
            let mut r = self.render_pipelines.model.begin_pass(
                ctx,
                self.backbuffer_target.into(),
                &self.geometry_buffers,
                Some(vec![self.crate_texture]),
                self.quad_mesh.clone(),
                RenderPassOptions::clear_depth(1.)
                    .with_projection(Mat4::perspective_lh(
                        60f32.to_radians(),
                        WINDOW_DIM.x as f32 / WINDOW_DIM.y as f32,
                        0.01,
                        100.,
                    ))
                    .with_view_transform(Mat4::look_at_lh(
                        vec3(self.xy.x / 10., 1., self.xy.y / 10. - 4.),
                        vec3(0., 0., 0.),
                        vec3(0., -1., 0.),
                    )),
            );
            r.render_mesh(
                self.cube_mesh.clone(),
                ModelInstanceData {
                    transform: transform::Transform3D {
                        rotation: Quat::from_rotation_x(self.angle)
                            * Quat::from_rotation_y(self.angle),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            );
        }

        // draw a screen-sized quad using the previously rendered offscreen render-target as texture
        self.backbuffer_target.draw_to_screen(
            ctx,
            &mut self.render_pipelines.basic,
            DisplayMode::Centered,
        );

        {
            let s = format!("fps: {:.0}", self.frame_timing.sampled_fps);
            let scale = self.render_scale.clamp(1., 20.);
            let mut r = self.render_pipelines.text.begin_pass(
                ctx,
                RenderTarget::Default,
                &self.geometry_buffers,
                Some(vec![self.font_texture]),
                self.quad_mesh.clone(),
                RenderPassOptions::clear_depth(1.0).with_projection(Mat4::orthographic_lh(
                    0.0,
                    WINDOW_DIM.x as f32,
                    WINDOW_DIM.y as f32,
                    0.0,
                    1.0,
                    -1.0,
                )),
            );
            r.draw_text(
                (10., 40.).into(),
                &s,
                &self.asset_manager.font_atlas,
                TextDisplayOptions {
                    color: Color::WHITE,
                    layout: LayoutOptions::scale(scale),
                },
            );

            let mut help_text = String::new();

            Controls::controls().iter().for_each(|c| {
                help_text += &format!(
                    "{:?}: {}\n",
                    c,
                    self.input
                        .bound_inputs(c)
                        .iter()
                        .map(|input| { format!("{}", input) })
                        .collect::<Vec<String>>()
                        .join(", ")
                );
            });

            r.draw_text(
                // self.input.mouse.position(),
                WINDOW_DIM.as_vec2() / 2.0,
                if !self.input.show_help.on {
                    "test\nokay right? nice."
                } else {
                    &help_text
                },
                &self.asset_manager.font_atlas,
                TextDisplayOptions {
                    color: Color::WHITE,
                    layout: LayoutOptions::scale(scale / 2.0),
                },
            );
        }

        ctx.commit_frame();
    }
}
