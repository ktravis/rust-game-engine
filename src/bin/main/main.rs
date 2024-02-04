use std::iter;

use glam::{vec2, vec3, Mat4, Quat, Vec3};
use rust_game_engine::renderer::sprite_renderer::MakeSpriteRenderer;
use ttf_parser::Face;
use wgpu::include_wgsl;
use winit::dpi::{PhysicalPosition, PhysicalSize, Size};
use winit::event::WindowEvent::KeyboardInput;
use winit::event::{ElementState, Event, KeyEvent, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowBuilder};

use rust_game_engine::color::Color;
use rust_game_engine::font::FontAtlas;
use rust_game_engine::geom::{ModelVertexData, Point, Rect};
use rust_game_engine::renderer::instance::{DrawInstance, InstanceRenderData};
use rust_game_engine::renderer::mesh::LoadMesh;
use rust_game_engine::renderer::state::DefaultUniforms;
use rust_game_engine::renderer::text::{DrawText, MakeFontFaceRenderer, TextDisplayOptions};
use rust_game_engine::renderer::{Display, PipelineRef, RenderData, RenderState};
use rust_game_engine::renderer::{
    DisplayMode, ModelInstanceData, OffscreenFramebuffer, RenderTarget, VertexLayout,
};
use rust_game_engine::renderer::{MeshRef, TextureBuilder, TextureRef};
use rust_game_engine::sprite_manager::SpriteManager;
use rust_game_engine::transform::Transform;

// impl EventHandler for Stage {
//     fn update(&mut self, ctx: &mut GraphicsContext) {
//         self.frame_timing.update(miniquad::date::now());
//         if !Stage::update(self, ctx) {
//             ctx.request_quit();
//         }
//         // Do this after the frame is done updating, so we can clear state and update controls for the next frame.
//         self.input.end_frame_update();
//     }
//
//     fn draw(&mut self, ctx: &mut GraphicsContext) {
//         Stage::draw(self, ctx);
//         // let current_frame_time = miniquad::date::now() - self.frame_start + 0.0005;
//         // if current_frame_time < self.target_frame_duration {
//         //     std::thread::sleep(Duration::from_secs_f64(
//         //         self.target_frame_duration - current_frame_time,
//         //     ))
//         // }
//     }
//
//     fn mouse_motion_event(&mut self, _ctx: &mut GraphicsContext, x: f32, y: f32) {
//         self.input.handle_mouse_motion(x, y)
//     }
//
//     fn mouse_wheel_event(&mut self, _ctx: &mut GraphicsContext, x: f32, y: f32) {
//         if x != 0. {
//             self.input.handle_analog_axis_change(MouseWheelX, x);
//         }
//         if y != 0. {
//             self.input.handle_analog_axis_change(MouseWheelY, y);
//         }
//     }
//
//     fn mouse_button_down_event(
//         &mut self,
//         _ctx: &mut GraphicsContext,
//         button: miniquad::MouseButton,
//         _x: f32,
//         _y: f32,
//     ) {
//         self.input
//             .handle_key_or_button_change(translate_button(button), input::StateChange::Pressed);
//     }
//
//     fn mouse_button_up_event(
//         &mut self,
//         _ctx: &mut GraphicsContext,
//         button: miniquad::MouseButton,
//         _x: f32,
//         _y: f32,
//     ) {
//         self.input
//             .handle_key_or_button_change(translate_button(button), input::StateChange::Released);
//     }
//
//     fn key_down_event(
//         &mut self,
//         _ctx: &mut GraphicsContext,
//         keycode: miniquad::KeyCode,
//         _keymods: miniquad::KeyMods,
//         _repeat: bool,
//     ) {
//         self.input
//             .handle_key_or_button_change(translate_key(keycode), input::StateChange::Pressed);
//     }
//
//     fn key_up_event(
//         &mut self,
//         _ctx: &mut GraphicsContext,
//         keycode: miniquad::KeyCode,
//         _keymods: miniquad::KeyMods,
//     ) {
//         self.input
//             .handle_key_or_button_change(translate_key(keycode), input::StateChange::Released);
//     }
// }

// SpriteManager will be more complicated because it needs to rebuild lazily after loading

#[derive(Debug, Copy, Clone)]
struct Camera {
    position: Vec3,
    pitch: f32,
    yaw: f32,
    look_dir: Vec3,
    fov_radians: f32,
}

impl Camera {
    const DEFAULT_FOV_RADIANS: f32 = 60.0 * (std::f32::consts::PI / 180.0);

    pub fn view_matrix(&self) -> Mat4 {
        let (pitch_sin, pitch_cos) = self.pitch.sin_cos();
        let (yaw_sin, yaw_cos) = self.yaw.sin_cos();
        Mat4::look_to_lh(self.position, self.look_dir, Vec3::Y)
    }
}

impl Default for Camera {
    fn default() -> Self {
        Camera {
            position: Vec3::ZERO,
            pitch: -0.40,
            yaw: 4.7, // TODO: debug value
            fov_radians: Self::DEFAULT_FOV_RADIANS,
            look_dir: Vec3::Z,
        }
    }
}

struct State {
    // application configuration
    display: Display,

    // renderer state
    render_state: RenderState,
    offscreen_framebuffer: OffscreenFramebuffer,
    to_screen_pipeline: PipelineRef,
    instances: Vec<InstanceRenderData>,
    instance_buffer: wgpu::Buffer,
    instanced_render_pipeline: PipelineRef,

    // TODO: maybe TextRenderer trait that works on something with a DrawInstance trait (raw render
    // pass or instance encoder), can be implemented by FontFaceRenderer and a BitmapFontRenderer
    // text render state
    font_atlas: FontAtlas,
    font_render_data: RenderData,

    quad_mesh: MeshRef,
    cube_mesh: MeshRef,
    sprite_manager: SpriteManager,
    sprite_render_data: RenderData,

    // "game" state
    diffuse_texture: TextureRef,
    diffuse_texture2: TextureRef,
    cursor_position: Option<PhysicalPosition<f64>>,
    camera: Camera,
}

impl State {
    // Creating some of the wgpu types requires async code
    async fn new(window: Window) -> anyhow::Result<Self> {
        let display = Display::from_window(window).await;

        // TODO: should the render state just hold onto the main vertex layout? Is there ever
        // going to be more than one we're using at a time? It would simplify creating pipelines
        // - I think yes, since pipelines are now swappable for a given mesh/model.
        let mut render_state = RenderState::new(&display);

        let quad_mesh = render_state.prepare_mesh(display.device().load_quad_mesh());
        let cube_mesh = render_state.prepare_mesh(display.device().load_cube_mesh());

        let diffuse_texture = render_state.load_texture(
            &display,
            TextureBuilder::labeled("crate").from_image(
                display.device(),
                display.queue(),
                &image::load_from_memory(include_bytes!("../../../res/images/crate.png"))?
                    .as_rgba8()
                    .unwrap(),
            ),
        );
        let diffuse_texture2 = render_state.load_texture(
            &display,
            TextureBuilder::labeled("sample").from_image(
                display.device(),
                display.queue(),
                &image::load_from_memory(include_bytes!("../../../res/images/sample.png"))
                    .unwrap()
                    .as_rgba8()
                    .unwrap(),
            ),
        );

        let mut sprite_manager = SpriteManager::default();
        sprite_manager.add_sprite_file_path("res/sprites/guy.aseprite");
        sprite_manager.rebuild_atlas()?;

        let sprite_atlas = render_state.load_texture(
            &display,
            TextureBuilder::labeled("sprite_atlas").from_image(
                display.device(),
                display.queue(),
                sprite_manager.atlas_image(),
            ),
        );

        let font_data = std::fs::read("./res/fonts/Ubuntu-M.ttf")?;
        let face = Face::parse(&font_data, 0)?;
        let font_atlas = FontAtlas::new(face, Default::default())?;
        let font_atlas_texture = render_state.load_texture(
            &display,
            TextureBuilder::labeled("font_atlas")
                .with_filter_mode(wgpu::FilterMode::Linear)
                .from_image(display.device(), display.queue(), font_atlas.image()),
        );

        let default_shader = display
            .device()
            .create_shader_module(include_wgsl!("../../../res/shaders/default.wgsl"));
        let to_screen_pipeline = render_state.create_pipeline(
            "Render to Screen Pipeline",
            &display,
            &default_shader,
            &[ModelVertexData::vertex_layout()],
        );

        let [instance_buffer] = render_state
            .create_vertex_buffers(display.device(), [ModelInstanceData::vertex_layout()]);
        let instanced_render_pipeline = render_state.create_pipeline(
            "Instanced Render Pipeline",
            &display,
            &display
                .device()
                .create_shader_module(include_wgsl!("../../../res/shaders/model.wgsl")),
            &[
                ModelVertexData::vertex_layout(),
                ModelInstanceData::vertex_layout(),
            ],
        );
        let text_render_pipeline = render_state.create_pipeline(
            "Text Render Pipeline",
            &display,
            &display
                .device()
                .create_shader_module(include_wgsl!("../../../res/shaders/text.wgsl")),
            &[
                ModelVertexData::vertex_layout(),
                ModelInstanceData::vertex_layout(),
            ],
        );
        let instances = vec![
            InstanceRenderData {
                texture: Some(diffuse_texture),
                mesh: cube_mesh,
                model: ModelInstanceData::default(),
                pipeline: instanced_render_pipeline,
            },
            InstanceRenderData {
                texture: Some(diffuse_texture2),
                mesh: quad_mesh,
                model: ModelInstanceData {
                    transform: Transform {
                        position: vec3(-2.2, 1.0, 0.0),
                        scale: Vec3::splat(2.5),
                        rotation: Quat::from_rotation_z(45.),
                    },
                    subtexture: Rect::new(0.0, 0.0, 0.25, 0.25),
                    ..Default::default()
                },
                pipeline: instanced_render_pipeline,
            },
        ];

        let offscreen_framebuffer = OffscreenFramebuffer::new(
            display.device(),
            // TODO: want this to be effectively private, should have a better way to
            // do this
            render_state.texture_bind_group_layout(),
            Point::new(960, 720),
        );

        Ok(Self {
            display,
            render_state,
            quad_mesh,
            cube_mesh,
            diffuse_texture,
            diffuse_texture2,
            font_render_data: RenderData {
                texture: font_atlas_texture,
                pipeline: text_render_pipeline,
                mesh: quad_mesh,
            },
            sprite_render_data: RenderData {
                pipeline: instanced_render_pipeline,
                texture: sprite_atlas,
                mesh: quad_mesh,
            },
            cursor_position: Default::default(),
            camera: Camera {
                position: vec3(0.0, 2.3, 6.0),
                ..Default::default()
            },
            offscreen_framebuffer,
            instances,
            instance_buffer,
            instanced_render_pipeline,
            to_screen_pipeline,
            sprite_manager,
            font_atlas,
        })
    }

    pub fn window(&self) -> &Window {
        self.display.window()
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        match *event {
            WindowEvent::CursorMoved { position, .. } => {
                if let Some(old_position) = self.cursor_position {
                    let delta_x = position.x - old_position.x;
                    let delta_y = position.y - old_position.y;
                    self.camera.yaw -= delta_x as f32 / 100.0;
                    self.camera.pitch -= delta_y as f32 / 100.0;
                    let (pitch_sin, pitch_cos) = self.camera.pitch.sin_cos();
                    let (yaw_sin, yaw_cos) = self.camera.yaw.sin_cos();
                    self.camera.look_dir =
                        vec3(pitch_cos * yaw_cos, pitch_sin, pitch_cos * yaw_sin).normalize();
                }
                self.cursor_position = Some(position);
            }
            KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        ..
                    },
                ..
            } => {
                let flat = vec3(self.camera.look_dir.x, 0.0, self.camera.look_dir.z);
                let dir = match code {
                    KeyCode::KeyA => flat.cross(Vec3::Y),
                    KeyCode::KeyW => flat,
                    KeyCode::KeyS => -flat,
                    KeyCode::KeyD => flat.cross(-Vec3::Y),
                    _ => {
                        return false;
                    }
                };
                self.camera.position += dir;
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if state.is_pressed() && button == winit::event::MouseButton::Left {
                    let i = self.instances.len() - 2;
                    self.instances.push(InstanceRenderData {
                        model: ModelInstanceData {
                            transform: Transform {
                                position: vec3(
                                    -12.0 + 4. * (i % 10) as f32,
                                    -12.0 + 4. * (i / 10) as f32,
                                    16.0,
                                ),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        texture: Some(self.sprite_render_data.texture),
                        pipeline: self.instanced_render_pipeline,
                        mesh: self.cube_mesh,
                    });
                }
            }
            _ => {}
        }
        false
    }

    fn update(&mut self) {
        if self.sprite_manager.maybe_rebuild() {
            self.sprite_render_data.texture = self.render_state.load_texture(
                &self.display,
                TextureBuilder::labeled("sprite_atlas").from_image(
                    self.display.device(),
                    self.display.queue(),
                    self.sprite_manager.atlas_image(),
                ),
            );
        }
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let device = self.display.device();
        let queue = self.display.queue();

        self.render_state.default_uniforms.update(
            queue,
            DefaultUniforms {
                view: self.camera.view_matrix(),
                projection: Mat4::perspective_lh(
                    self.camera.fov_radians,
                    960.0 / 720.0,
                    0.01,
                    100.0,
                ), // TODO add near/far/aspect to camera
                ..Default::default()
            },
        );
        let mut encoder =
            self.display
                .device()
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });
        {
            let mut render_pass = self.render_state.begin_render_pass(
                &mut encoder,
                &wgpu::RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[Some(
                        self.offscreen_framebuffer
                            .color_attachment(wgpu::LoadOp::Clear(wgpu::Color::BLACK)),
                    )],
                    depth_stencil_attachment: self
                        .offscreen_framebuffer
                        .depth_stencil_attachment(wgpu::LoadOp::Clear(1.0), None),
                    timestamp_writes: None,
                    occlusion_query_set: None,
                },
            );
            let mut instance_encoder =
                render_pass.instance_encoder(&self.display, &self.instance_buffer);
            for instance in &self.instances {
                instance_encoder.draw_instance(instance);
            }
            let mut text_renderer =
                instance_encoder.font_face_renderer(&self.font_atlas, self.font_render_data);
            text_renderer.draw_text(
                "howdy there",
                Transform {
                    position: vec3(1.5, 2.0, 1.0),
                    scale: vec3(0.05, -0.05, 1.0),
                    rotation: Quat::from_rotation_z(-15.0f32.to_radians())
                        * Quat::from_rotation_y(180.0f32.to_radians()),
                },
                TextDisplayOptions {
                    color: Color::RED,
                    ..Default::default()
                },
            );
        }
        queue.submit(iter::once(encoder.finish()));

        let display_view = self.display.view()?;
        let target_size = display_view.size_pixels().as_vec2();
        // TODO: this whole thing should be cached based on the source/target size
        self.render_state.default_uniforms.update(
            queue,
            DefaultUniforms {
                view: DisplayMode::Centered.scaling_matrix(
                    self.offscreen_framebuffer.size_pixels().as_vec2(),
                    target_size,
                ),
                projection: Mat4::orthographic_lh(
                    0.0,
                    target_size.x,
                    target_size.y,
                    0.0,
                    1.0,
                    -1.0,
                ),
                ..Default::default()
            },
        );
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });
        {
            let mut render_pass = self.render_state.begin_render_pass(
                &mut encoder,
                &wgpu::RenderPassDescriptor {
                    label: Some("Offscreen Framebuffer to Screen"),
                    color_attachments: &[Some(
                        display_view.color_attachment(wgpu::LoadOp::Clear(wgpu::Color::RED)),
                    )],
                    depth_stencil_attachment: display_view
                        .depth_stencil_attachment(wgpu::LoadOp::Clear(1.0), None),
                    ..Default::default()
                },
            );
            render_pass.set_active_pipeline(self.to_screen_pipeline);
            render_pass.bind_texture_data(&self.offscreen_framebuffer.color);
            render_pass.set_active_mesh(self.quad_mesh);
            render_pass.draw_active_mesh(0..1);
        }
        queue.submit(iter::once(encoder.finish()));

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });
        self.render_state.default_uniforms.update(
            queue,
            DefaultUniforms {
                projection: Mat4::orthographic_lh(
                    0.0,
                    target_size.x,
                    target_size.y,
                    0.0,
                    1.0,
                    -1.0,
                ),
                ..Default::default()
            },
        );
        {
            let mut render_pass = self.render_state.begin_render_pass(
                &mut encoder,
                &wgpu::RenderPassDescriptor {
                    label: Some("UI Pass"),
                    color_attachments: &[Some(display_view.color_attachment(wgpu::LoadOp::Load))],
                    depth_stencil_attachment: display_view
                        .depth_stencil_attachment(wgpu::LoadOp::Clear(1.0), None),
                    ..Default::default()
                },
            );
            let mut instance_encoder =
                render_pass.instance_encoder(&self.display, &self.instance_buffer);
            {
                let mut sprite_renderer =
                    instance_encoder.sprite_renderer(&self.sprite_manager, self.sprite_render_data);
                sprite_renderer.draw_sprite(
                    self.sprite_manager.get_sprite_ref("guy").unwrap(),
                    0,
                    Transform {
                        position: vec3(
                            self.cursor_position.unwrap_or_default().x as _,
                            self.cursor_position.unwrap_or_default().y as _,
                            0.0,
                        ),
                        scale: vec3(4.0, 4.0, 1.0),
                        rotation: Quat::from_rotation_z(45.0f32.to_radians()),
                    },
                )
            }
            {
                let mut text_renderer =
                    instance_encoder.font_face_renderer(&self.font_atlas, self.font_render_data);
                text_renderer.draw_text_2d(
                    format!("{}", self.camera.position),
                    vec2(12.0, 36.0),
                    TextDisplayOptions::default(),
                );
            }
        }
        queue.submit(iter::once(encoder.finish()));

        display_view.present();

        Ok(())
    }
}

pub async fn run() {
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_inner_size(Size::new(PhysicalSize::new(960, 720)))
        .build(&event_loop)
        .unwrap();

    let mut state = State::new(window).await.unwrap();

    event_loop
        .run(move |event, elwt| match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window().id() => {
                if !state.input(event) {
                    match event {
                        WindowEvent::CloseRequested
                        | KeyboardInput {
                            event:
                                KeyEvent {
                                    physical_key: PhysicalKey::Code(KeyCode::Escape),
                                    state: ElementState::Pressed,
                                    ..
                                },
                            ..
                        } => elwt.exit(),
                        WindowEvent::Resized(physical_size) => {
                            state.display.resize(*physical_size);
                            state.window().request_redraw();
                        }
                        // WindowEvent::ScaleFactorChanged { scale_factor, .. } => {}
                        WindowEvent::RedrawRequested => {
                            state.update();
                            match state.render() {
                                Ok(_) => {}
                                // Reconfigure the surface if it's lost or outdated
                                Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                                    state.display.reconfigure();
                                }
                                // The system is out of memory, we should probably quit
                                Err(wgpu::SurfaceError::OutOfMemory) => {
                                    elwt.exit();
                                }
                                Err(wgpu::SurfaceError::Timeout) => log::warn!("Surface timeout"),
                            }
                            state.window().request_redraw();
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        })
        .unwrap();
}

fn main() {
    pollster::block_on(run());
}
