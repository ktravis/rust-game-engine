use std::borrow::Cow;
use std::io::Read;

use glam::{vec2, vec3, vec4};
use itertools::Itertools;
use rust_game_engine::app::{App, AppState, Context};
use rust_game_engine::renderer::geometry::GeometryPass;
use rust_game_engine::renderer::lighting::{Light, LightingPass};
use rust_game_engine::renderer::model::LoadModel;
use rust_game_engine::renderer::shadow_mapping::ShadowMappingPass;
use rust_game_engine::renderer::ssao::SSAOPass;
use rust_game_engine::renderer::{InstanceDataWithNormalMatrix, MeshRef, RenderTarget};
use ttf_parser::Face;
use winit::dpi::PhysicalSize;
use winit::event_loop::EventLoop;

use controlset_derive::ControlSet;
use rust_game_engine::assets::AssetManager;
use rust_game_engine::camera::Camera;
use rust_game_engine::font::FontAtlas;
use rust_game_engine::geom::{BasicVertexData, ModelVertexData, Point};
use rust_game_engine::input::{Axis, Button, ControlSet, Key, Toggle};
use rust_game_engine::renderer::{
    instance::InstanceRenderData,
    mesh::LoadMesh,
    state::{GlobalUniforms, ViewProjectionUniforms},
    text::TextDisplayOptions,
    BasicInstanceData, Display, OffscreenFramebuffer, PipelineRef, RenderData, ScalingMode,
    TextureBuilder, TextureRef,
};
use rust_game_engine::sprite_manager::SpriteManager;
use rust_game_engine::transform::{Transform, Transform2D, Transform3D};

const WINDOW_SIZE: PhysicalSize<u32> = PhysicalSize::new(960, 720);
const RENDER_SCALE: f32 = 1.0;

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
    #[bind(Key::GraveAccent)]
    debug: Toggle,
}

#[derive(Debug, Default, Clone)]
struct ShaderSource {
    dirty: bool,

    text: String,
}

#[derive(Default)]
struct RenderPipelines {
    text: PipelineRef<BasicVertexData, BasicInstanceData>,
    // ssao_blur: PipelineRef<BasicVertexData, BasicInstanceData>,
    // lighting: PipelineRef<BasicVertexData, BasicInstanceData>,
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
    font_render_data: RenderData<BasicVertexData, BasicInstanceData>,
    sprite_render_data: RenderData<BasicVertexData, BasicInstanceData>,
    offscreen_framebuffer: OffscreenFramebuffer,
    shadow_mapping_pass: ShadowMappingPass,
    geometry_pass: GeometryPass,
    occlusion_pass: SSAOPass,
    ssao_enabled: bool,
    deferred_lighting_pass: LightingPass,

    // "game" state
    camera: Camera,
    lights: Vec<Light>,
    sprite_instances: Vec<InstanceRenderData>,
    crate_texture: TextureRef,
    cat_texture: TextureRef,
    cube_mesh: MeshRef<ModelVertexData>,

    model_meshes: Vec<(MeshRef<ModelVertexData>, Option<tobj::Material>)>,
}

impl State {
    fn add_sprites(&mut self, display: &Display) {
        let size = display.size_pixels();
        let sprite_ref = self.asset_manager.sprites.get_sprite_ref("guy").unwrap();
        let sprite = self.asset_manager.sprites.get_sprite(sprite_ref);
        for _ in 0..100 {
            self.sprite_instances.push(
                self.sprite_render_data.for_instance(BasicInstanceData {
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

    fn create_or_update_render_pipelines<C: ControlSet>(&mut self, ctx: &mut Context<C>) {
        // TODO: this should probably just move into the renderer, or maybe have a separate text
        // renderer layer
        self.render_pipelines.text = ctx
            .render_state
            .pipeline_builder()
            .with_label("Text Render Pipeline")
            .with_key(self.render_pipelines.text)
            .build(
                ctx.display.device(),
                &ctx.display
                    .device()
                    .create_shader_module(wgpu::ShaderModuleDescriptor {
                        label: Some("text"),
                        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(
                            &self.asset_manager.shader_sources.text,
                        )),
                    }),
            );
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
            ctx.display
                .load_texture_bytes(
                    include_bytes!("../../../res/images/crate.png"),
                    TextureBuilder::labeled("crate"),
                )
                .unwrap(),
        );
        let cat_texture = ctx.render_state.load_texture(
            &ctx.display,
            ctx.display
                .load_texture_bytes(
                    include_bytes!("../../../res/images/sample.png"),
                    TextureBuilder::labeled("sample"),
                )
                .unwrap(),
        );

        asset_manager.track_glob("./res/sprites/*.aseprite", |state, path, f| {
            state.sprites.add_sprite_file(path.to_path_buf(), f);
        });
        let sprite_atlas = ctx.render_state.load_texture(
            &ctx.display,
            TextureBuilder::labeled("sprite_atlas").from_image(
                ctx.display.device(),
                ctx.display.queue(),
                asset_manager.sprites.atlas_image(),
            ),
        );

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

        // asset_manager.track_file("./res/shaders/model.wgsl", |state, _, f| {
        //     state.shader_sources.dirty = true;
        //     state.shader_sources.model_with_normals = std::io::read_to_string(f).unwrap();
        // });
        //
        let model = ctx
            .display
            .device()
            // .load_model("./res/models/jeep.obj")
            .load_model("./res/models/astronautB.obj")
            .unwrap();

        let model_meshes = model
            .meshes
            .into_iter()
            .map(|m| (ctx.render_state.prepare_mesh(m.mesh), m.material))
            .collect();

        let fb_size = Point::from((
            (WINDOW_SIZE.width as f32 / RENDER_SCALE) as u32,
            (WINDOW_SIZE.height as f32 / RENDER_SCALE) as u32,
        ));
        let offscreen_framebuffer =
            ctx.render_state
                .create_offscreen_framebuffer(&ctx.display, fb_size, None);

        let sprite_render_data = RenderData {
            pipeline: None,
            texture: sprite_atlas,
            mesh: ctx.render_state.quad_mesh(),
        };
        ctx.set_cursor_captured(true);

        let shadow_mapping_pass = ShadowMappingPass::new(&mut ctx.render_state, &ctx.display);
        let geometry_pass = GeometryPass::new(&mut ctx.render_state, &ctx.display, fb_size);

        let occlusion_pass = SSAOPass::new(
            &mut ctx.render_state,
            &ctx.display,
            geometry_pass.bind_group_layout(),
        );

        let deferred_lighting_pass = LightingPass::new(
            &mut ctx.render_state,
            &ctx.display,
            geometry_pass.bind_group_layout(),
        );

        let mut state = Self {
            asset_manager,
            crate_texture,
            cat_texture,
            cube_mesh,
            sprite_instances: vec![],
            camera: Camera::new(vec3(0.0, 2.3, 6.0), 960.0 / 720.0),
            lights: vec![],
            // lights: vec![Light {
            //     position: vec4(5.0, 2.5, 7.0, 1.0),
            //     color: Color::WHITE.into(),
            //     proj: Mat4::orthographic_rh(-20.0, 20.0, -20.0, 20.0, 1.0, 24.0),
            //     view: Mat4::look_at_rh(vec3(0.0, 5.0, 0.0), Vec3::ZERO, Vec3::X),
            // }],
            shadow_mapping_pass,
            geometry_pass,
            occlusion_pass,
            ssao_enabled: true,
            font_render_data: Default::default(),
            sprite_render_data,
            offscreen_framebuffer,
            render_pipelines: Default::default(),
            model_meshes,
            deferred_lighting_pass,
        };
        state.create_or_update_render_pipelines(ctx);
        state.font_render_data = RenderData {
            pipeline: Some(state.render_pipelines.text),
            texture: font_atlas_texture,
            mesh: ctx.render_state.quad_mesh(),
        };
        state
    }

    fn update(&mut self, ctx: &mut Context<GameControls>) -> bool {
        if self.asset_manager.check_for_updates() {
            if self.asset_manager.shader_sources.dirty {
                println!("shader sources updated");
                self.asset_manager.shader_sources.dirty = false;
                self.create_or_update_render_pipelines(ctx);
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
        if !ctx.input.debug.on {
            if let Some(delta) = ctx.input.mouse_delta {
                let delta = 0.8 * ctx.frame_timing.delta().as_secs_f32() * delta;
                self.camera.update_angle(delta.x, delta.y);
            }
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
        if ctx.input.debug.just_pressed() {
            ctx.set_cursor_captured(!ctx.input.debug.on);
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
        let view_proj = ViewProjectionUniforms::for_camera(&self.camera);

        let mut scene = vec![
            InstanceRenderData {
                texture: Some(self.crate_texture),
                mesh: self.cube_mesh,
                instance: InstanceDataWithNormalMatrix::from_basic(
                    Default::default(),
                    view_proj.view,
                ),
                pipeline: None,
            },
            InstanceRenderData {
                mesh: self.cube_mesh,
                instance: InstanceDataWithNormalMatrix::from_basic(
                    BasicInstanceData {
                        tint: vec4(1.0, 0.9, 0.9, 1.0).into(),
                        transform: Transform3D {
                            position: vec3(-1.0, 1.5, -0.4),
                            ..Default::default()
                        }
                        .as_mat4(),
                        ..Default::default()
                    },
                    view_proj.view,
                ),
                texture: None,
                pipeline: None,
            },
            InstanceRenderData {
                mesh: self.cube_mesh,
                instance: InstanceDataWithNormalMatrix::from_basic(
                    BasicInstanceData {
                        tint: vec4(0.88, 0.82, 0.8, 1.0).into(),
                        transform: Transform3D {
                            position: vec3(0.0, -1.5, 0.0),
                            scale: vec3(30.0, 0.5, 30.0),
                            ..Default::default()
                        }
                        .as_mat4(),
                        ..Default::default()
                    },
                    view_proj.view,
                ),
                texture: None,
                pipeline: None,
            },
        ];
        for (mesh, mat) in &self.model_meshes {
            scene.push(InstanceRenderData {
                mesh: *mesh,
                instance: InstanceDataWithNormalMatrix::from_basic(
                    BasicInstanceData {
                        tint: mat.as_ref().map(|m| m.diffuse.into()).unwrap_or_default(),
                        transform: Transform3D {
                            position: vec3(0.0, 2.0, 0.0),
                            // scale: vec3(0.01, 0.01, 0.01),
                            ..Default::default()
                        }
                        .as_mat4(),
                        ..Default::default()
                    },
                    view_proj.view,
                ),
                texture: None,
                pipeline: None,
            });
        }

        // Populate G buffers
        self.geometry_pass
            .run(&mut ctx.render_state, &ctx.display, &view_proj, &scene);

        let default_texture = ctx.render_state.default_texture();
        let occlusion_map = if self.ssao_enabled {
            self.occlusion_pass.run(
                &mut ctx.render_state,
                &ctx.display,
                &view_proj,
                self.geometry_pass.bind_group().clone(),
            )
        } else {
            default_texture
        };

        // self.shadow_mapping_pass
        //     .run(&mut ctx.render_state, &ctx.display, &self.lights, &scene);

        // Deferred lighting pass
        self.deferred_lighting_pass.run(
            &mut ctx.render_state,
            &ctx.display,
            RenderTarget::TextureRef(self.offscreen_framebuffer.color),
            &view_proj,
            self.geometry_pass.bind_group().clone(),
            occlusion_map,
            &self.lights,
        );

        // Draw offscreen buffer, overlay with 2d elements
        let display_view = ctx.display.view()?;
        let mut enc = ctx
            .render_state
            .render_pass(
                &ctx.display,
                "Default Pass",
                &[RenderTarget::TextureView(display_view.view())],
                Some(RenderTarget::TextureView(
                    &display_view.display().depth_texture().view,
                )),
                &ViewProjectionUniforms {
                    projection: display_view.orthographic_projection(),
                    ..Default::default()
                },
                |r| {
                    r.draw_quad(
                        self.offscreen_framebuffer.color,
                        ScalingMode::Centered.view_matrix(
                            self.offscreen_framebuffer.size_pixels().as_vec2(),
                            ctx.display.size_pixels().as_vec2(),
                        ),
                    );
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
                        format!("{:.2}", ctx.frame_timing.fps())
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
            .encoder();

        // Draw egui menu if debug is enabled
        if ctx.input.debug.on {
            ctx.egui.draw(
                &ctx.display,
                &mut enc,
                wgpu::RenderPassColorAttachment {
                    view: display_view.view(),
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                },
                Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &display_view.display().depth_texture().view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                |ui| {
                    egui::Window::new("Debug")
                        // .vscroll(true)
                        .default_open(true)
                        // .max_width(2000.0)
                        // .max_height(1200.0)
                        // .default_width(1000.0)
                        .resizable(true)
                        .anchor(egui::Align2::LEFT_TOP, [0.0, 0.0])
                        .show(&ui, |ui| {
                            ui.spacing_mut().slider_width = 200.0;
                            ui.label("Lights");
                            self.deferred_lighting_pass.debug_ui(ui);

                            ui.separator();
                            ui.label("SSAO");
                            ui.add(egui::Checkbox::new(&mut self.ssao_enabled, "enabled"));
                            self.occlusion_pass.debug_ui(ui);
                        });
                },
            );
        }
        ctx.display.queue().submit([enc.finish()]);

        display_view.present();

        Ok(())
    }
}

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let mut app = App::<State>::new(WINDOW_SIZE.into());
    event_loop.run_app(&mut app).unwrap();
}
