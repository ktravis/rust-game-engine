use std::borrow::Cow;
use std::io::Read;

use glam::{vec2, vec3, Mat4, Quat, Vec3};
use slotmap::Key as SlotmapKey;
use ttf_parser::Face;
use winit::dpi::{PhysicalPosition, PhysicalSize, Size};
use winit::event::{Event, KeyEvent, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::PhysicalKey;
use winit::window::{Window, WindowBuilder};

use controlset_derive::ControlSet;
use rust_game_engine::assets::AssetManager;
use rust_game_engine::camera::Camera;
use rust_game_engine::font::FontAtlas;
use rust_game_engine::geom::{ModelVertexData, Point};
use rust_game_engine::input::{
    AnalogInput, Axis, Button, ControlSet, InputManager, Key, MouseButton, Toggle,
};
use rust_game_engine::renderer::instance::InstanceRenderData;
use rust_game_engine::renderer::mesh::LoadMesh;
use rust_game_engine::renderer::state::{GlobalUniforms, ViewProjectionUniforms};
use rust_game_engine::renderer::text::TextDisplayOptions;
use rust_game_engine::renderer::{
    Display, OffscreenFramebuffer, PipelineRef, RenderData, RenderState, RenderTarget, ScalingMode,
};
use rust_game_engine::renderer::{ModelInstanceData, VertexLayout};
use rust_game_engine::renderer::{TextureBuilder, TextureRef};
use rust_game_engine::sprite_manager::SpriteManager;
use rust_game_engine::time::FrameTiming;
use rust_game_engine::transform::{Transform, Transform2D, Transform3D};

#[derive(ControlSet)]
struct Controls {
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

    model: String,
    text: String,
}

#[derive(Default)]
struct RenderPipelines {
    instanced: PipelineRef,
    text: PipelineRef,
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
        self.instanced = render_state.create_pipeline_with_key(
            "Instanced Render Pipeline",
            &display,
            &display
                .device()
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("instanced"),
                    source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(&src.model)),
                }),
            &[
                ModelVertexData::vertex_layout(),
                ModelInstanceData::vertex_layout(),
            ],
            if self.instanced.is_null() {
                None
            } else {
                Some(self.instanced)
            },
        );
        self.text = render_state.create_pipeline_with_key(
            "Text Render Pipeline",
            &display,
            &display
                .device()
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("text"),
                    source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(&src.text)),
                }),
            &[
                ModelVertexData::vertex_layout(),
                ModelInstanceData::vertex_layout(),
            ],
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

struct AppBase<A: App> {
    app: A,
    frame_timing: FrameTiming,
    display: Display,
}

impl<A: App> AppBase<A> {
    pub async fn new(window: Window) -> Self {
        let display = Display::from_window(window).await;
        let app = A::new(&display);
        Self {
            app,
            frame_timing: Default::default(),
            display,
        }
    }

    pub fn handle_input(&mut self, event: &WindowEvent) {
        self.app.handle_input(event);
    }

    pub fn window(&self) -> &Window {
        self.display.window()
    }

    pub fn update(&mut self) -> bool {
        self.frame_timing.update();
        if !self.app.update(&self.display, &self.frame_timing) {
            return false;
        }
        true
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.app.render(&self.display);
        Ok(())
    }
}

pub trait App {
    fn new(display: &Display) -> Self;
    fn update(&mut self, display: &Display, frame_timing: &FrameTiming) -> bool;
    fn render(&mut self, display: &Display);
    fn handle_input(&mut self, _event: &WindowEvent) {}
    fn destroy(&mut self, _display: &Display) {}
}

struct State {
    frame_timing: FrameTiming,
    input: InputManager<Controls>,
    display: Display,
    asset_manager: AssetManager<GameAssets>,

    render_state: RenderState,
    render_pipelines: RenderPipelines,

    // TODO: BitmapFontRenderer
    font_render_data: RenderData,
    sprite_render_data: RenderData,
    world_view_projection: ViewProjectionUniforms,
    ui_view_projection: ViewProjectionUniforms,
    offscreen_fb_size_pixels: Point<u32>,
    offscreen_framebuffer: OffscreenFramebuffer,

    // "game" state
    instances: Vec<InstanceRenderData<Transform3D>>,
    camera: Camera,
    sprite_instances: Vec<InstanceRenderData<Mat4>>,
    crate_texture: TextureRef,
    cat_texture: TextureRef,
}

impl State {
    // Creating some of the wgpu types requires async code
    async fn new(window: Window) -> anyhow::Result<Self> {
        let display = Display::from_window(window).await;

        let mut asset_manager = AssetManager::new(GameAssets::default(), "./res/");

        // TODO: should the render state just hold onto the main vertex layout? Is there ever
        // going to be more than one we're using at a time? It would simplify creating pipelines
        // - I think yes, since pipelines are now swappable for a given mesh/model.
        let mut render_state = RenderState::new(&display);

        let cube_mesh = render_state.prepare_mesh(display.device().load_cube_mesh());

        let crate_texture = render_state.load_texture(
            &display,
            TextureBuilder::labeled("crate").from_image(
                display.device(),
                display.queue(),
                &image::load_from_memory(include_bytes!("../../../res/images/crate.png"))?
                    .as_rgba8()
                    .unwrap(),
            ),
        );
        let cat_texture = render_state.load_texture(
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

        asset_manager.track_glob("./res/sprites/*.aseprite", |state, path, f| {
            state.sprites.add_sprite_file(path.to_path_buf(), f);
        });
        // TODO: should have a way to not need this
        let sprite_atlas = {
            asset_manager.sprites.maybe_rebuild();
            render_state.load_texture(
                &display,
                TextureBuilder::labeled("sprite_atlas").from_image(
                    display.device(),
                    display.queue(),
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
        let font_atlas_texture = render_state.load_texture(
            &display,
            TextureBuilder::labeled("font_atlas")
                .with_filter_mode(wgpu::FilterMode::Linear)
                .from_image(
                    display.device(),
                    display.queue(),
                    asset_manager.font_atlas.image(),
                ),
        );

        asset_manager
            .track_file("./res/shaders/model.wgsl", |state, _, f| {
                state.shader_sources.dirty = true;
                state.shader_sources.model = std::io::read_to_string(f).unwrap();
            })
            .track_file("./res/shaders/text.wgsl", |state, _, f| {
                state.shader_sources.dirty = true;
                state.shader_sources.text = std::io::read_to_string(f).unwrap();
            });

        let mut render_pipelines = RenderPipelines::default();
        render_pipelines.create_or_update(
            &mut render_state,
            &display,
            &asset_manager.shader_sources,
        );

        let offscreen_fb_size_pixels = Point::new(480, 360);
        let offscreen_framebuffer =
            render_state.create_offscreen_framebuffer(&display, offscreen_fb_size_pixels);

        let instances = vec![
            InstanceRenderData {
                texture: Some(crate_texture),
                mesh: cube_mesh,
                model: ModelInstanceData::default(),
                pipeline: render_pipelines.instanced,
            },
            InstanceRenderData {
                texture: Some(cat_texture),
                mesh: render_state.quad_mesh(),
                model: ModelInstanceData {
                    transform: Transform3D {
                        position: vec3(4.0, 1.0, 3.0),
                        scale: Vec3::splat(2.5),
                        rotation: Quat::from_rotation_y(180.0f32.to_radians())
                            * Quat::from_rotation_z(10.0f32.to_radians()),
                    },
                    ..Default::default()
                },
                pipeline: render_pipelines.instanced,
            },
        ];

        let font_render_data = RenderData {
            pipeline: render_pipelines.text,
            texture: font_atlas_texture,
            mesh: render_state.quad_mesh(),
        };
        let sprite_render_data = RenderData {
            pipeline: render_pipelines.instanced,
            texture: sprite_atlas,
            mesh: render_state.quad_mesh(),
        };

        Ok(Self {
            asset_manager,
            frame_timing: Default::default(),
            input: Default::default(),
            display,
            crate_texture,
            cat_texture,
            instances,
            sprite_instances: vec![],
            camera: Camera::new(vec3(0.0, 2.3, -6.0), 960.0 / 720.0),
            font_render_data,
            sprite_render_data,
            world_view_projection: Default::default(),
            ui_view_projection: Default::default(),
            offscreen_fb_size_pixels,
            offscreen_framebuffer,
            render_state,
            render_pipelines,
        })
    }

    pub fn window(&self) -> &Window {
        self.display.window()
    }

    fn input(&mut self, event: &WindowEvent) {
        match *event {
            WindowEvent::CursorMoved { position, .. } => {
                self.input
                    .handle_mouse_motion(position.x as f32, position.y as f32);
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state,
                        ..
                    },
                ..
            } => {
                self.input
                    .handle_key_or_button_change(Key::from(code), state.into());
            }
            WindowEvent::MouseWheel { delta, .. } => {
                match delta {
                    winit::event::MouseScrollDelta::PixelDelta(PhysicalPosition { x, y }) => {
                        if x != 0. {
                            self.input
                                .handle_analog_axis_change(AnalogInput::MouseWheelX, x as f32);
                        }
                        if y != 0. {
                            self.input
                                .handle_analog_axis_change(AnalogInput::MouseWheelY, y as f32);
                        }
                    }
                    // winit::event::MouseScrollDelta::LineDelta(_, _) => todo!(),
                    _ => {}
                };
            }
            WindowEvent::MouseInput { state, button, .. } => {
                self.input
                    .handle_key_or_button_change(MouseButton::from(button), state.into());
            }
            _ => {}
        }
    }

    fn update(&mut self) -> bool {
        self.frame_timing.update();

        if self.asset_manager.check_for_updates() {
            if self.asset_manager.shader_sources.dirty {
                self.asset_manager.shader_sources.dirty = false;
                self.render_pipelines.create_or_update(
                    &mut self.render_state,
                    &self.display,
                    &self.asset_manager.shader_sources,
                );
            }
            if self.asset_manager.sprites.maybe_rebuild() {
                self.render_state.replace_texture(
                    &self.display,
                    self.sprite_render_data.texture,
                    TextureBuilder::labeled("sprite_atlas").from_image(
                        self.display.device(),
                        self.display.queue(),
                        self.asset_manager.sprites.atlas_image(),
                    ),
                );
            }
        }
        if let Some(delta) = self.input.mouse.delta() {
            let delta = 2.0 * self.frame_timing.delta().as_secs_f32() * delta;
            self.camera.update_angle(delta.x, delta.y);
        }
        self.camera.update_position(
            self.frame_timing.delta().as_secs_f32()
                * 10.0
                * vec3(
                    self.input.x.value(),
                    self.input.scale.value(),
                    self.input.y.value(),
                )
                .normalize_or_zero(),
        );
        if self.input.add.is_down() {
            self.add_sprites();
            println!("{}", self.sprite_instances.len());
        }
        if self.input.quit.is_down() {
            return false;
        }

        // Do this after the frame is done updating, so we can clear state and update controls for the next frame.
        self.input.end_frame_update();
        true
    }

    fn add_sprites(&mut self) {
        let size = self.display.size_pixels();
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

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.render_state.global_uniforms.update(
            self.display.queue(),
            GlobalUniforms {
                time: self.frame_timing.time(),
            },
        );
        self.world_view_projection = ViewProjectionUniforms {
            view: self.camera.view_matrix(),
            projection: self.camera.perspective_matrix(),
        };
        self.render_state.render_pass(
            &self.display,
            "Offscreen Pass",
            &self.offscreen_framebuffer,
            &self.world_view_projection,
            |r| {
                for instance in &self.instances {
                    r.draw_instance(instance);
                }
            },
        );

        self.render_state.default_render_pass(&self.display, |r| {
            r.draw_instance(&InstanceRenderData {
                texture: Some(self.offscreen_framebuffer.color),
                ..self
                    .sprite_render_data
                    .for_model_instance(ModelInstanceData::transform(
                        ScalingMode::Centered.view_matrix(
                            self.offscreen_fb_size_pixels.as_vec2(),
                            self.display.size_pixels().as_vec2(),
                        ),
                    ))
            });
            for instance in &self.sprite_instances {
                r.draw_instance(instance);
            }
            let text = if self.input.show_help.on {
                Controls::controls()
                    .iter()
                    .map(|c| {
                        format!(
                            "{:?}: {} = {}\n",
                            c,
                            self.input
                                .bound_inputs(c)
                                .iter()
                                .map(|input| { format!("{}", input) })
                                .collect::<Vec<String>>()
                                .join(", "),
                            self.input.control_state(c),
                        )
                    })
                    .collect()
            } else {
                format!("{:.1}", self.frame_timing.fps())
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
        })?;

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
                state.input(event);
                match event {
                    WindowEvent::CloseRequested => elwt.exit(),
                    WindowEvent::Resized(physical_size) => {
                        state.display.resize(*physical_size);
                        state.window().request_redraw();
                    }
                    // WindowEvent::ScaleFactorChanged { scale_factor, .. } => {}
                    WindowEvent::RedrawRequested => {
                        if !state.update() {
                            elwt.exit();
                            return;
                        }
                        match state.render() {
                            Ok(_) => {}
                            // Reconfigure the surface if it's lost or outdated
                            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                                state.display.reconfigure();
                            }
                            // The system is out of memory, we should probably quit
                            Err(wgpu::SurfaceError::OutOfMemory) => {
                                log::error!("Out of memory?!");
                                elwt.exit();
                            }
                            Err(wgpu::SurfaceError::Timeout) => log::warn!("Surface timeout"),
                        }
                        state.window().request_redraw();
                    }
                    _ => {}
                }
            }
            _ => {}
        })
        .unwrap();
}

fn main() {
    pollster::block_on(run());
}
