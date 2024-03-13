use std::borrow::Cow;
use std::io::Read;

use glam::{vec2, vec3, Mat4, Quat, Vec3};
use itertools::Itertools;
use rust_game_engine::app::{AppState, Context};
use rust_game_engine::renderer::RenderTarget;
use ttf_parser::Face;
use winit::dpi::{PhysicalSize, Size};
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

use controlset_derive::ControlSet;
use rust_game_engine::assets::AssetManager;
use rust_game_engine::camera::Camera;
use rust_game_engine::font::FontAtlas;
use rust_game_engine::geom::{ModelVertexData, Point};
use rust_game_engine::input::{Axis, Button, Key, Toggle};
use rust_game_engine::renderer::{
    instance::InstanceRenderData,
    mesh::LoadMesh,
    state::{GlobalUniforms, ViewProjectionUniforms},
    text::TextDisplayOptions,
    Display, ModelInstanceData, OffscreenFramebuffer, PipelineRef, RenderData, RenderState,
    ScalingMode, TextureBuilder, TextureRef,
};
use rust_game_engine::sprite_manager::SpriteManager;
use rust_game_engine::transform::{Transform, Transform2D, Transform3D};

#[derive(ControlSet)]
struct GameControls {
    #[bind((Key::A, Key::D))]
    x: Axis,
    #[bind((Key::W, Key::S))]
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

#[derive(Debug, Default, Clone)]
struct ShaderSource {
    dirty: bool,

    text: String,
}

#[derive(Default)]
struct RenderPipelines {
    text: PipelineRef<ModelVertexData, ModelInstanceData>,
}

impl RenderPipelines {
    // TODO: use device error scopes to handle errors properly (need to make most things async
    // by default
    fn create_or_update(
        &mut self,
        render_state: &mut RenderState,
        display: &Display,
        src: &ShaderSource,
    ) {
        // TODO: this should probably just move into the renderer, or maybe have a separate text
        // renderer layer
        self.text = render_state.create_pipeline_with_key::<ModelVertexData, ModelInstanceData>(
            "Text Render Pipeline",
            &display,
            &display
                .device()
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("text"),
                    source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(&src.text)),
                }),
            if self.text.is_null() {
                None
            } else {
                Some(self.text)
            },
        );
    }
}

#[derive(Default)]
struct GameAssets {
    shader_sources: ShaderSource,
    font_atlas: FontAtlas,
    sprites: SpriteManager,
}

struct State {
    asset_manager: AssetManager<GameAssets>,

    render_pipelines: RenderPipelines,

    // TODO: BitmapFontRenderer
    font_render_data: RenderData<ModelVertexData, ModelInstanceData>,
    sprite_render_data: RenderData<ModelVertexData, ModelInstanceData>,
    offscreen_framebuffer: OffscreenFramebuffer,

    // "game" state
    instances: Vec<InstanceRenderData>,
    camera: Camera,
    sprite_instances: Vec<InstanceRenderData>,
    crate_texture: TextureRef,
    cat_texture: TextureRef,
}

impl State {
    fn add_sprites(&mut self, display: &Display) {
        let size = display.size_pixels();
        let sprite_ref = self.asset_manager.sprites.get_sprite_ref("guy").unwrap();
        let sprite = self.asset_manager.sprites.get_sprite(sprite_ref);
        for _ in 0..100 {
            self.sprite_instances.push(
                self.sprite_render_data
                    .for_model_instance(ModelInstanceData {
                        subtexture: sprite.frames[0].region,
                        transform: Transform2D {
                            position: vec2(
                                (rand::random::<u32>() % size.x) as f32,
                                (rand::random::<u32>() % size.y) as f32,
                            ),
                            scale: 4.0 * sprite.size.as_vec2(),
                            ..Default::default()
                        }
                        .as_mat4(),
                        ..Default::default()
                    }),
            );
        }
    }
}

impl AppState for State {
    type Controls = GameControls;

    fn new(ctx: &mut Context<Self::Controls>) -> Self {
        let mut asset_manager = AssetManager::new(GameAssets::default(), "./res/");

        let cube_mesh = ctx
            .render_state
            .prepare_mesh(ctx.display.device().load_cube_mesh());

        let crate_texture = ctx.render_state.load_texture(
            &ctx.display,
            TextureBuilder::labeled("crate").from_image(
                ctx.display.device(),
                ctx.display.queue(),
                &image::load_from_memory(include_bytes!("../../../res/images/crate.png"))
                    .unwrap()
                    .as_rgba8()
                    .unwrap(),
            ),
        );
        let cat_texture = ctx.render_state.load_texture(
            &ctx.display,
            TextureBuilder::labeled("sample").from_image(
                ctx.display.device(),
                ctx.display.queue(),
                &image::load_from_memory(include_bytes!("../../../res/images/sample.png"))
                    .unwrap()
                    .as_rgba8()
                    .unwrap(),
            ),
        );

        asset_manager.track_glob("./res/sprites/*.aseprite", |state, path, f| {
            state.sprites.add_sprite_file(path.to_path_buf(), f);
        });
        // TODO: should have a way to not need this
        let sprite_atlas = {
            asset_manager.sprites.maybe_rebuild();
            ctx.render_state.load_texture(
                &ctx.display,
                TextureBuilder::labeled("sprite_atlas").from_image(
                    ctx.display.device(),
                    ctx.display.queue(),
                    asset_manager.sprites.atlas_image(),
                ),
            )
        };

        // TODO: these callbacks should be able to return an error, optionally
        asset_manager.track_file("./res/fonts/Ubuntu-M.ttf", |state, _, mut f| {
            let mut b = vec![];
            f.read_to_end(&mut b).unwrap();
            let face = Face::parse(&b, 0).unwrap();
            state.font_atlas = FontAtlas::new(face, Default::default()).unwrap();
        });
        let font_atlas_texture = ctx.render_state.load_texture(
            &ctx.display,
            TextureBuilder::labeled("font_atlas")
                .with_filter_mode(wgpu::FilterMode::Linear)
                .from_image(
                    ctx.display.device(),
                    ctx.display.queue(),
                    asset_manager.font_atlas.image(),
                ),
        );

        asset_manager.track_file("./res/shaders/text.wgsl", |state, _, f| {
            state.shader_sources.dirty = true;
            state.shader_sources.text = std::io::read_to_string(f).unwrap();
        });

        let mut render_pipelines = RenderPipelines::default();
        render_pipelines.create_or_update(
            &mut ctx.render_state,
            &ctx.display,
            &asset_manager.shader_sources,
        );

        let offscreen_framebuffer = ctx
            .render_state
            .create_offscreen_framebuffer(&ctx.display, Point::new(480, 360));

        let instances = vec![
            InstanceRenderData {
                texture: Some(crate_texture),
                mesh: cube_mesh,
                model: ModelInstanceData::default(),
                pipeline: None,
            },
            InstanceRenderData {
                texture: Some(cat_texture),
                mesh: ctx.render_state.quad_mesh(),
                model: ModelInstanceData {
                    transform: Transform3D {
                        position: vec3(4.0, 1.0, 3.0),
                        scale: Vec3::splat(2.5),
                        rotation: Quat::from_rotation_y(180.0f32.to_radians())
                            * Quat::from_rotation_z(10.0f32.to_radians()),
                    }
                    .as_mat4(),
                    ..Default::default()
                },
                pipeline: None,
            },
        ];

        let font_render_data = RenderData {
            pipeline: Some(render_pipelines.text),
            texture: font_atlas_texture,
            mesh: ctx.render_state.quad_mesh(),
        };
        let sprite_render_data = RenderData {
            pipeline: None,
            texture: sprite_atlas,
            mesh: ctx.render_state.quad_mesh(),
        };

        Self {
            asset_manager,
            crate_texture,
            cat_texture,
            instances,
            sprite_instances: vec![],
            camera: Camera::new(vec3(0.0, 2.3, -6.0), 960.0 / 720.0),
            font_render_data,
            sprite_render_data,
            offscreen_framebuffer,
            render_pipelines,
        }
    }

    fn update(&mut self, ctx: &mut Context<GameControls>) -> bool {
        if self.asset_manager.check_for_updates() {
            if self.asset_manager.shader_sources.dirty {
                self.asset_manager.shader_sources.dirty = false;
                self.render_pipelines.create_or_update(
                    &mut ctx.render_state,
                    &ctx.display,
                    &self.asset_manager.shader_sources,
                );
            }
            if self.asset_manager.sprites.maybe_rebuild() {
                ctx.render_state.replace_texture(
                    &ctx.display,
                    self.sprite_render_data.texture,
                    TextureBuilder::labeled("sprite_atlas").from_image(
                        ctx.display.device(),
                        ctx.display.queue(),
                        self.asset_manager.sprites.atlas_image(),
                    ),
                );
            }
        }
        if let Some(delta) = ctx.input.mouse.delta() {
            let delta = 2.0 * ctx.frame_timing.delta().as_secs_f32() * delta;
            self.camera.update_angle(delta.x, delta.y);
        }
        self.camera.update_position(
            ctx.frame_timing.delta().as_secs_f32()
                * 10.0
                * vec3(
                    ctx.input.x.value(),
                    ctx.input.scale.value(),
                    ctx.input.y.value(),
                )
                .normalize_or_zero(),
        );
        if ctx.input.add.is_down() {
            self.add_sprites(&ctx.display);
            println!("{}", self.sprite_instances.len());
        }
        if ctx.input.quit.is_down() {
            return false;
        }
        true
    }

    fn render(&mut self, ctx: &mut Context<GameControls>) -> Result<(), wgpu::SurfaceError> {
        ctx.render_state.global_uniforms.update(
            ctx.display.queue(),
            GlobalUniforms {
                time: ctx.frame_timing.time(),
            },
        );
        ctx.render_state
            .render_pass(
                &ctx.display,
                "Offscreen Pass",
                &self.offscreen_framebuffer,
                &ViewProjectionUniforms {
                    view: self.camera.view_matrix(),
                    projection: self.camera.perspective_matrix(),
                },
                |r| {
                    for instance in &self.instances {
                        r.draw_instance(instance);
                    }
                },
            )
            .submit();

        let display_view = ctx.display.view()?;
        ctx.render_state
            .render_pass(
                &ctx.display,
                "Default Pass",
                &display_view,
                &ViewProjectionUniforms {
                    view: Mat4::IDENTITY,
                    projection: display_view.orthographic_projection(),
                },
                |r| {
                    r.draw_instance(&InstanceRenderData {
                        texture: Some(self.offscreen_framebuffer.color),
                        ..self
                            .sprite_render_data
                            .for_model_instance(ModelInstanceData::transform(
                                ScalingMode::Centered.view_matrix(
                                    self.offscreen_framebuffer.size_pixels().as_vec2(),
                                    ctx.display.size_pixels().as_vec2(),
                                ),
                            ))
                    });
                    for instance in &self.sprite_instances {
                        r.draw_instance(instance);
                    }
                    let text = if ctx.input.show_help.on {
                        ctx.input
                            .status()
                            .map(|(c, bindings, state)| {
                                format!("{:?} : {} = {}", c, bindings.iter().join(", "), state)
                            })
                            .join("\n")
                    } else {
                        format!("{:.1}", ctx.frame_timing.fps())
                    };
                    r.draw_text(
                        &text,
                        &self.asset_manager.font_atlas,
                        &self.font_render_data,
                        Transform2D {
                            position: vec2(20.0, 40.0),
                            ..Default::default()
                        },
                        TextDisplayOptions::default(),
                    );
                },
            )
            .submit();

        display_view.present();

        Ok(())
    }
}

fn main() {
    pollster::block_on(async {
        let event_loop = EventLoop::new()?;
        let window = WindowBuilder::new()
            .with_inner_size(Size::new(PhysicalSize::new(960, 720)))
            .build(&event_loop)?;

        let display = Display::from_window(window).await;
        State::create_app(display).run(event_loop)
    })
    .expect("game crashed");
}
