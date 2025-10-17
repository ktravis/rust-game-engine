use std::ops::DerefMut;

use bytemuck::Zeroable;
use glam::{vec2, vec3, vec4, Mat3, Mat4, Quat, Vec2, Vec3};
use itertools::Itertools;
use rust_game_engine::app::{App, AppState, Context};
use rust_game_engine::color::Color;
use rust_game_engine::renderer::forward::ForwardGeometryPass;
use rust_game_engine::renderer::geometry::GeometryPass;
use rust_game_engine::renderer::lighting::{Light, LightKind};
use rust_game_engine::renderer::model::LoadModel;
use rust_game_engine::renderer::shader_type::GlobalUniforms;
use rust_game_engine::renderer::shadow_mapping::ShadowMappingPass;
use rust_game_engine::renderer::ssao_from_depth::SSAOPass;
use rust_game_engine::renderer::text::RenderableFont;
use rust_game_engine::renderer::{InstanceDataWithNormalMatrix, MeshRef, RenderTarget};
use winit::dpi::PhysicalSize;
use winit::event_loop::EventLoop;

use controlset_derive::ControlSet;
use rust_game_engine::assets::AssetManager;
use rust_game_engine::camera::Camera;
use rust_game_engine::geom::{BasicVertexData, ModelVertexData, Point};
use rust_game_engine::input::{Axis, Button, Key, Toggle};
use rust_game_engine::renderer::{
    instance::InstanceRenderData, mesh::LoadMesh, state::ViewProjectionUniforms,
    text::TextDisplayOptions, BasicInstanceData, Display, OffscreenFramebuffer, RenderData,
    ScalingMode, TextureBuilder, TextureRef,
};
use rust_game_engine::sprite_manager::SpriteManager;
use rust_game_engine::transform::{Transform, Transform2D, Transform3D};

const WINDOW_SIZE: PhysicalSize<u32> = PhysicalSize::new(1080, 720);
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

#[derive(Default)]
struct GameAssets {
    sprites: SpriteManager,
}

struct State {
    asset_manager: AssetManager<GameAssets>,

    // TODO: BitmapFontRenderer
    default_font: RenderableFont,

    sprite_render_data: RenderData<BasicVertexData, BasicInstanceData>,
    offscreen_framebuffer: OffscreenFramebuffer,
    shadow_mapping_pass: ShadowMappingPass,
    geometry_pass: GeometryPass,
    occlusion_pass: SSAOPass,
    ssao_enabled: bool,
    // deferred_lighting_pass: LightingPass,
    forward_pass: ForwardGeometryPass,

    // "game" state
    camera: Camera,
    lights: Vec<Light>,
    sprite_instances: Vec<InstanceRenderData>,
    crate_texture: TextureRef,
    cat_texture: TextureRef,
    cube_mesh: MeshRef<ModelVertexData>,

    model_meshes: Vec<(MeshRef<ModelVertexData>, Option<tobj::Material>)>,
    cubes: Vec<Transform3D>,

    scene: Scene,
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
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Scene {
    Cubes,
    Model,
}

impl AppState for State {
    type Controls = GameControls;

    fn new(ctx: &mut Context<Self::Controls>) -> Self {
        let mut asset_manager = AssetManager::new(GameAssets::default(), "./res/");

        let camera = Camera::new(vec3(0.0, 2.3, 6.0), 960.0 / 720.0);

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

        let model = ctx
            .display
            .device()
            // .load_model("./res/models/jeep.obj")
            .load_model("./res/models/room_thickwalls.obj")
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
        let forward_pass = ForwardGeometryPass::new(
            &mut ctx.render_state,
            &ctx.display,
            fb_size,
            &shadow_mapping_pass.shadow_map_texture(),
        );
        let geometry_pass = GeometryPass::new(&mut ctx.render_state, &ctx.display, fb_size);
        let occlusion_pass = SSAOPass::new(
            &mut ctx.render_state,
            &ctx.display,
            fb_size,
            &forward_pass.depth_target,
            &camera,
        );

        let mut cubes = vec![];
        for x in 0..32 {
            for z in 0..32 {
                let dx = x as f32 - 16.0;
                let dz = z as f32 - 16.0;
                let dr = (dx * dx + dz * dz) / (24.0 * 24.0);
                cubes.push(Transform3D {
                    position: vec3(2.0 * dx, rand::random::<f32>() * 1.5, 2.0 * dz),
                    scale: vec3(1.0, 1.0 + 20.0 * dr, 1.0),
                    ..Default::default()
                });
            }
        }

        Self {
            asset_manager,
            crate_texture,
            cat_texture,
            cube_mesh,
            default_font: RenderableFont::new(ctx),
            sprite_instances: vec![],
            camera,
            // lights: vec![],
            lights: vec![
                // LightKind::Directional {
                //     theta: -65.0,
                //     phi: -30.0,
                // }
                // .into(),
                LightKind::Spot {
                    position: vec3(0.0, 5.0, 0.0),
                    direction: vec3(0.0, -8.0, 30.0),
                    fov_degrees: 60.0,
                    reach: 40.0,
                }
                .into(),
                Light {
                    color: Color::RED,
                    kind: LightKind::Spot {
                        position: vec3(0.0, 5.0, 0.0),
                        direction: vec3(0.0, -8.0, 30.0),
                        fov_degrees: 60.0,
                        reach: 40.0,
                    },
                },
                Light {
                    color: Color::GREEN,
                    kind: LightKind::Spot {
                        position: vec3(0.0, 5.0, 0.0),
                        direction: vec3(0.0, -8.0, 30.0),
                        fov_degrees: 60.0,
                        reach: 40.0,
                    },
                },
            ],
            shadow_mapping_pass,
            geometry_pass,
            forward_pass,
            occlusion_pass,
            ssao_enabled: true,
            // font_render_data: Default::default(),
            sprite_render_data,
            offscreen_framebuffer,
            // render_pipelines: Default::default(),
            model_meshes,
            // deferred_lighting_pass,
            cubes,
            scene: Scene::Cubes,
        }
    }

    fn update(&mut self, ctx: &mut Context<GameControls>) -> bool {
        if self.asset_manager.check_for_updates() {
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
                screen_size: ctx.display.size_pixels().as_vec2() / RENDER_SCALE,
                ..Zeroable::zeroed()
            },
        );
        let view_proj = ViewProjectionUniforms::for_camera(&self.camera);

        // let mut scene = vec![];
        let mut scene = vec![
            // InstanceRenderData {
            //     texture: Some(self.crate_texture),
            //     mesh: self.cube_mesh,
            //     instance: InstanceDataWithNormalMatrix::from_basic(
            //         Default::default(),
            //         view_proj.view,
            //     ),
            //     pipeline: None,
            // },
            // InstanceRenderData {
            //     mesh: self.cube_mesh,
            //     instance: InstanceDataWithNormalMatrix::from_basic(
            //         BasicInstanceData {
            //             tint: vec4(1.0, 0.9, 0.9, 1.0).into(),
            //             transform: Transform3D {
            //                 position: vec3(
            //                     -1.0 + f32::sin(ctx.frame_timing.time()) * 5.0,
            //                     1.5,
            //                     -0.4,
            //                 ),
            //                 ..Default::default()
            //             }
            //             .as_mat4(),
            //             ..Default::default()
            //         },
            //         view_proj.view,
            //     ),
            //     texture: None,
            //     pipeline: None,
            // },
            // InstanceRenderData {
            //     mesh: self.cube_mesh,
            //     instance: InstanceDataWithNormalMatrix::from_basic(
            //         BasicInstanceData {
            //             // tint: vec4(0.88, 0.82, 0.8, 1.0).into(),
            //             transform: Transform3D {
            //                 position: vec3(0.0, -1.5, 0.0),
            //                 scale: vec3(30.0, 0.5, 30.0),
            //                 ..Default::default()
            //             }
            //             .as_mat4(),
            //             ..Default::default()
            //         },
            //         view_proj.view,
            //     ),
            //     texture: None,
            //     pipeline: None,
            // },
        ];
        match self.scene {
            Scene::Cubes => {
                scene.extend(self.cubes.iter().map(|t| InstanceRenderData {
                    mesh: self.cube_mesh,
                    instance: InstanceDataWithNormalMatrix::from_basic(
                        BasicInstanceData {
                            transform: t.as_mat4(),
                            ..Default::default()
                        },
                        view_proj.view,
                    ),
                    texture: None,
                    pipeline: None,
                }));
            }
            Scene::Model => {
                for (mesh, mat) in &self.model_meshes {
                    scene.push(InstanceRenderData {
                        mesh: *mesh,
                        instance: InstanceDataWithNormalMatrix::from_basic(
                            BasicInstanceData {
                                tint: mat.as_ref().map(|m| m.diffuse.into()).unwrap_or_default(),
                                transform: Transform3D {
                                    position: vec3(0.0, 0.0, 5.0),
                                    // scale: vec3(0.02, 0.02, 0.02),
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
            }
        }

        // Populate G buffers
        // self.geometry_pass
        //     .run(&mut ctx.render_state, &ctx.display, &view_proj, &scene);

        self.forward_pass
            .depth_prepass(&mut ctx.render_state, &ctx.display, &view_proj, &scene);

        let occlusion_map = if self.ssao_enabled {
            self.occlusion_pass
                .run(&mut ctx.render_state, &ctx.display, &view_proj)
        } else {
            ctx.render_state.default_texture()
        };

        self.lights
            .iter_mut()
            .enumerate()
            .for_each(|(i, light)| match &mut light.kind {
                LightKind::Directional { .. } => todo!(),
                LightKind::Spot { direction, .. } => {
                    *direction = Mat3::from_rotation_y((i as f32 + 0.5) * 0.01) * *direction;
                }
            });

        self.forward_pass
            .lights_uniform
            .update_with(ctx.display.queue(), |u| {
                u.lights = self.lights.clone();
                u.view_frustum = self.camera.frustum();
            });

        self.shadow_mapping_pass.run(
            &mut ctx.render_state,
            &ctx.display,
            &self.forward_pass.lights_uniform,
            &scene,
        );

        if ctx.input.debug.on {
            for light in &self.lights {
                let pos = light.kind.position();
                scene.push(InstanceRenderData {
                    mesh: self.cube_mesh,
                    instance: InstanceDataWithNormalMatrix::from_basic(
                        BasicInstanceData {
                            transform: Mat4::from_scale_rotation_translation(
                                vec3(0.05, 2.5, 0.05),
                                Quat::from_rotation_arc(Vec3::Y, pos.normalize()),
                                Vec3::ZERO,
                            ) * Mat4::from_translation(Vec3::Y),
                            tint: light.color.into(),
                            ..Default::default()
                        },
                        view_proj.view,
                    ),
                    texture: None,
                    pipeline: None,
                });

                // scene.push(InstanceRenderData {
                //     mesh: self.cube_mesh,
                //     instance: InstanceDataWithNormalMatrix::from_basic(
                //         BasicInstanceData {
                //             transform: light.view.inverse() * Mat4::from_scale(vec3(30.0, 40.0, 20.0)),
                //             tint: light.color.into(),
                //             ..Default::default()
                //         },
                //         view_proj.view,
                //     ),
                //     texture: None,
                //     pipeline: None,
                // });
            }
        }

        self.forward_pass.run(
            &mut ctx.render_state,
            &ctx.display,
            &view_proj,
            &scene,
            occlusion_map,
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
                        // self.offscreen_framebuffer.color,
                        self.forward_pass.color_target,
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
                        &self.default_font,
                        &text,
                        Transform2D {
                            position: vec2(20.0, 40.0),
                            ..Default::default()
                        },
                        TextDisplayOptions::default(),
                    );

                    // r.draw_quad(
                    //     // self.shadow_mapping_pass.shadow_map_debug_textures[0],
                    //     occlusion_map,
                    //     Transform2D {
                    //         position: vec2(ctx.display.size_pixels().x as f32 - 266.0, 10.0),
                    //         // position: ctx.display.size_pixels().as_vec2() - vec2(256.0, 256.0),
                    //         scale: Vec2::splat(256.0),
                    //         ..Default::default()
                    //     },
                    // );
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
                        .vscroll(true)
                        .default_open(true)
                        // .max_width(2000.0)
                        .max_height(ctx.display.size_pixels().y as f32)
                        // .default_width(1000.0)
                        .resizable(true)
                        .anchor(egui::Align2::LEFT_TOP, [0.0, 0.0])
                        .show(&ui, |ui| {
                            ui.spacing_mut().slider_width = 200.0;

                            egui::ComboBox::from_label("Scene")
                                .selected_text(format!("{:?}", self.scene))
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut self.scene, Scene::Cubes, "cubes");
                                    ui.selectable_value(&mut self.scene, Scene::Model, "model");
                                });

                            ui.separator();
                            ui.label("Lights");
                            let lights_uniform = self.forward_pass.lights_uniform.deref_mut();
                            ui.label("Bias min");
                            ui.add(egui::Slider::new(
                                &mut self.shadow_mapping_pass.depth_bias_state.constant,
                                -10..=10,
                            ));
                            ui.label("Bias factor");
                            ui.add(egui::Slider::new(
                                &mut self.shadow_mapping_pass.depth_bias_state.slope_scale,
                                -1.0..=5.0,
                            ));
                            ui.label("Bias factor");
                            ui.add(egui::Slider::new(
                                &mut self.shadow_mapping_pass.depth_bias_state.clamp,
                                -1.0..=5.0,
                            ));
                            ui.label("Blur factor");
                            ui.add(egui::Slider::new(
                                &mut lights_uniform.shadow_blur_half_kernel_size,
                                0..=10,
                            ));
                            {
                                let mut c = egui::Rgba::from_rgba_premultiplied(
                                    lights_uniform.ambient_color.r,
                                    lights_uniform.ambient_color.g,
                                    lights_uniform.ambient_color.b,
                                    lights_uniform.ambient_color.a,
                                );
                                ui.horizontal(|ui| {
                                    ui.label("Ambient Light");
                                    egui::color_picker::color_edit_button_rgba(
                                        ui,
                                        &mut c,
                                        egui::color_picker::Alpha::OnlyBlend,
                                    );
                                });
                                lights_uniform.ambient_color = c.into();
                            }

                            for mut i in 0..(self.lights.len() as usize) {
                                if i >= self.lights.len() {
                                    break;
                                }
                                if i > 0 {
                                    ui.separator();
                                }
                                let removed = ui
                                    .horizontal(|ui| {
                                        ui.label(&format!("Light {}", i));
                                        if ui.button("remove").clicked() {
                                            self.lights.remove(i);
                                            i = i.saturating_sub(1);
                                            true
                                        } else {
                                            false
                                        }
                                    })
                                    .inner;
                                if removed {
                                    continue;
                                }
                                let light = &mut self.lights[i];
                                light.debug_ui(ui);
                            }
                            // if self.lights.len() < lighting::MAX_LIGHTS {
                            //     if ui.add(egui::Button::new("Add Light")).clicked() {
                            //         self.lights.push(Light {
                            //             position: vec4(0.0, 5.0, 0.0, 1.0),
                            //             color: Color::WHITE.into(),
                            //             proj: Mat4::orthographic_rh(
                            //                 -20.0, 20.0, -20.0, 20.0, 0.01, 100.0,
                            //             ),
                            //             view: Mat4::look_at_rh(
                            //                 vec3(0.0, 5.0, 0.0),
                            //                 Vec3::ZERO,
                            //                 Vec3::X,
                            //             ),
                            //         });
                            //     }
                            // }

                            ui.separator();
                            ui.label("SSAO");
                            ui.add(egui::Checkbox::new(&mut self.ssao_enabled, "enabled"));
                            if self.ssao_enabled {
                                self.occlusion_pass.debug_ui(ui);
                            }
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
