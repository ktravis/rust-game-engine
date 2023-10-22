use anyhow::anyhow;
use controlset_derive::ControlSet;
use rust_game_engine::font::{FontAtlas, LayoutOptions};
use std::cell::Cell;
use std::io::Read;
use std::path::Path;
use std::rc::Rc;
use ttf_parser::Face;

use rust_game_engine::{color::Color, resources, transform};

use rust_game_engine::geom::{cube, quad, ModelVertexData, Point};
use rust_game_engine::input::{
    self, AnalogInput::*, Axis, Button, ControlSet, InputManager, Key, MouseButton, Toggle,
};
use rust_game_engine::renderer::{
    BasicRenderPipeline, DisplayMode, InstancedRenderPipeline, ModelInstanceData,
    OffscreenFramebuffer, RawInstanceData, RenderPassOptions, RenderTarget, TextDisplayOptions,
};

use glam::{vec3, Mat4, Quat, Vec2};
use miniquad::{EventHandler, GraphicsContext, Texture};
use rust_game_engine::assets::AssetManager;
use rust_game_engine::mesh_manager::{GeometryBuffers, MeshManager, MeshOffsets};
use rust_game_engine::shader::Shader;
use rust_game_engine::sprite_manager::{SpriteManager, SpriteRef};

const WINDOW_DIM: Point = Point { x: 960, y: 720 };
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

struct Stage {
    camera_offset: Vec2,
    input: InputManager<Controls>,
    asset_manager: AssetManager<GameAssets>,
    frame_timing: FrameTiming,
    target_frame_duration: f64,
    backbuffer_target: OffscreenFramebuffer,
    render_pipelines: RenderPipelines,
    sprite_atlas_texture: Texture,
    crate_texture: Texture,
    font_texture: Texture,
    quad_mesh: Rc<Cell<MeshOffsets>>,
    cube_mesh: Rc<Cell<MeshOffsets>>,
    geometry_buffers: GeometryBuffers<ModelVertexData>,

    s: SpriteRef,
    frame_index: usize,
    sprite_pos: Vec<Point>,
    xy: Vec2,
    angle: f32,
    render_scale: f32,
}

const SCALE: u32 = 1;

fn shader_error_mapper(err: miniquad::ShaderError) -> anyhow::Error {
    use miniquad::ShaderError::*;
    match err {
        CompilationError {
            shader_type,
            error_message,
        } => anyhow!(
            "{:?} shader compilation failed: {}",
            shader_type,
            error_message
        ),
        LinkError(msg) => anyhow!("linking failed: {}", msg),
        FFINulError(_) => anyhow!("shader has a null byte in it!"),
    }
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
            &image::open(Path::new("res/images/crate.png"))
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

    fn init(&mut self) {}

    fn update(&mut self, ctx: &mut GraphicsContext) -> bool {
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

    fn draw(&mut self, ctx: &mut GraphicsContext) {
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

impl EventHandler for Stage {
    fn update(&mut self, ctx: &mut GraphicsContext) {
        self.frame_timing.update(miniquad::date::now());
        if !Stage::update(self, ctx) {
            ctx.request_quit();
        }
        // Do this after the frame is done updating, so we can clear state and update controls for the next frame.
        self.input.end_frame_update();
    }

    fn draw(&mut self, ctx: &mut GraphicsContext) {
        Stage::draw(self, ctx);
        // let current_frame_time = miniquad::date::now() - self.frame_start + 0.0005;
        // if current_frame_time < self.target_frame_duration {
        //     std::thread::sleep(Duration::from_secs_f64(
        //         self.target_frame_duration - current_frame_time,
        //     ))
        // }
    }

    fn resize_event(&mut self, _ctx: &mut GraphicsContext, _width: f32, _height: f32) {}

    fn mouse_motion_event(&mut self, _ctx: &mut GraphicsContext, x: f32, y: f32) {
        self.input.handle_mouse_motion(x, y)
    }

    fn mouse_wheel_event(&mut self, _ctx: &mut GraphicsContext, x: f32, y: f32) {
        if x != 0. {
            self.input.handle_analog_axis_change(MouseWheelX, x);
        }
        if y != 0. {
            self.input.handle_analog_axis_change(MouseWheelY, y);
        }
    }

    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut GraphicsContext,
        button: miniquad::MouseButton,
        _x: f32,
        _y: f32,
    ) {
        self.input
            .handle_key_or_button_change(translate_button(button), input::StateChange::Pressed);
    }

    fn mouse_button_up_event(
        &mut self,
        _ctx: &mut GraphicsContext,
        button: miniquad::MouseButton,
        _x: f32,
        _y: f32,
    ) {
        self.input
            .handle_key_or_button_change(translate_button(button), input::StateChange::Released);
    }

    fn char_event(
        &mut self,
        _ctx: &mut GraphicsContext,
        _character: char,
        _keymods: miniquad::KeyMods,
        _repeat: bool,
    ) {
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut GraphicsContext,
        keycode: miniquad::KeyCode,
        _keymods: miniquad::KeyMods,
        _repeat: bool,
    ) {
        self.input
            .handle_key_or_button_change(translate_key(keycode), input::StateChange::Pressed);
    }

    fn key_up_event(
        &mut self,
        _ctx: &mut GraphicsContext,
        keycode: miniquad::KeyCode,
        _keymods: miniquad::KeyMods,
    ) {
        self.input
            .handle_key_or_button_change(translate_key(keycode), input::StateChange::Released);
    }

    fn touch_event(
        &mut self,
        ctx: &mut GraphicsContext,
        phase: miniquad::TouchPhase,
        _id: u64,
        x: f32,
        y: f32,
    ) {
        use miniquad::TouchPhase::*;
        if phase == Started {
            self.mouse_button_down_event(ctx, miniquad::MouseButton::Left, x, y);
        }

        if phase == Ended {
            self.mouse_button_up_event(ctx, miniquad::MouseButton::Left, x, y);
        }

        if phase == Moved {
            self.mouse_motion_event(ctx, x, y);
        }
    }

    fn raw_mouse_motion(&mut self, _ctx: &mut GraphicsContext, _dx: f32, _dy: f32) {}

    fn window_minimized_event(&mut self, _ctx: &mut GraphicsContext) {}

    fn window_restored_event(&mut self, _ctx: &mut GraphicsContext) {}

    fn quit_requested_event(&mut self, _ctx: &mut GraphicsContext) {}

    fn files_dropped_event(&mut self, _ctx: &mut GraphicsContext) {}
}

fn translate_key(k: miniquad::KeyCode) -> Key {
    use miniquad::KeyCode;
    match k {
        KeyCode::Space => Key::Space,
        KeyCode::Apostrophe => Key::Apostrophe,
        KeyCode::Comma => Key::Comma,
        KeyCode::Minus => Key::Minus,
        KeyCode::Period => Key::Period,
        KeyCode::Slash => Key::Slash,
        KeyCode::Key0 => Key::Key0,
        KeyCode::Key1 => Key::Key1,
        KeyCode::Key2 => Key::Key2,
        KeyCode::Key3 => Key::Key3,
        KeyCode::Key4 => Key::Key4,
        KeyCode::Key5 => Key::Key5,
        KeyCode::Key6 => Key::Key6,
        KeyCode::Key7 => Key::Key7,
        KeyCode::Key8 => Key::Key8,
        KeyCode::Key9 => Key::Key9,
        KeyCode::Semicolon => Key::Semicolon,
        KeyCode::Equal => Key::Equal,
        KeyCode::A => Key::A,
        KeyCode::B => Key::B,
        KeyCode::C => Key::C,
        KeyCode::D => Key::D,
        KeyCode::E => Key::E,
        KeyCode::F => Key::F,
        KeyCode::G => Key::G,
        KeyCode::H => Key::H,
        KeyCode::I => Key::I,
        KeyCode::J => Key::J,
        KeyCode::K => Key::K,
        KeyCode::L => Key::L,
        KeyCode::M => Key::M,
        KeyCode::N => Key::N,
        KeyCode::O => Key::O,
        KeyCode::P => Key::P,
        KeyCode::Q => Key::Q,
        KeyCode::R => Key::R,
        KeyCode::S => Key::S,
        KeyCode::T => Key::T,
        KeyCode::U => Key::U,
        KeyCode::V => Key::V,
        KeyCode::W => Key::W,
        KeyCode::X => Key::X,
        KeyCode::Y => Key::Y,
        KeyCode::Z => Key::Z,
        KeyCode::LeftBracket => Key::LeftBracket,
        KeyCode::Backslash => Key::Backslash,
        KeyCode::RightBracket => Key::RightBracket,
        KeyCode::GraveAccent => Key::GraveAccent,
        KeyCode::World1 => Key::World1,
        KeyCode::World2 => Key::World2,
        KeyCode::Escape => Key::Escape,
        KeyCode::Enter => Key::Enter,
        KeyCode::Tab => Key::Tab,
        KeyCode::Backspace => Key::Backspace,
        KeyCode::Insert => Key::Insert,
        KeyCode::Delete => Key::Delete,
        KeyCode::Right => Key::Right,
        KeyCode::Left => Key::Left,
        KeyCode::Down => Key::Down,
        KeyCode::Up => Key::Up,
        KeyCode::PageUp => Key::PageUp,
        KeyCode::PageDown => Key::PageDown,
        KeyCode::Home => Key::Home,
        KeyCode::End => Key::End,
        KeyCode::CapsLock => Key::CapsLock,
        KeyCode::ScrollLock => Key::ScrollLock,
        KeyCode::NumLock => Key::NumLock,
        KeyCode::PrintScreen => Key::PrintScreen,
        KeyCode::Pause => Key::Pause,
        KeyCode::F1 => Key::F1,
        KeyCode::F2 => Key::F2,
        KeyCode::F3 => Key::F3,
        KeyCode::F4 => Key::F4,
        KeyCode::F5 => Key::F5,
        KeyCode::F6 => Key::F6,
        KeyCode::F7 => Key::F7,
        KeyCode::F8 => Key::F8,
        KeyCode::F9 => Key::F9,
        KeyCode::F10 => Key::F10,
        KeyCode::F11 => Key::F11,
        KeyCode::F12 => Key::F12,
        KeyCode::F13 => Key::F13,
        KeyCode::F14 => Key::F14,
        KeyCode::F15 => Key::F15,
        KeyCode::F16 => Key::F16,
        KeyCode::F17 => Key::F17,
        KeyCode::F18 => Key::F18,
        KeyCode::F19 => Key::F19,
        KeyCode::F20 => Key::F20,
        KeyCode::F21 => Key::F21,
        KeyCode::F22 => Key::F22,
        KeyCode::F23 => Key::F23,
        KeyCode::F24 => Key::F24,
        KeyCode::F25 => Key::F25,
        KeyCode::Kp0 => Key::Kp0,
        KeyCode::Kp1 => Key::Kp1,
        KeyCode::Kp2 => Key::Kp2,
        KeyCode::Kp3 => Key::Kp3,
        KeyCode::Kp4 => Key::Kp4,
        KeyCode::Kp5 => Key::Kp5,
        KeyCode::Kp6 => Key::Kp6,
        KeyCode::Kp7 => Key::Kp7,
        KeyCode::Kp8 => Key::Kp8,
        KeyCode::Kp9 => Key::Kp9,
        KeyCode::KpDecimal => Key::KpDecimal,
        KeyCode::KpDivide => Key::KpDivide,
        KeyCode::KpMultiply => Key::KpMultiply,
        KeyCode::KpSubtract => Key::KpSubtract,
        KeyCode::KpAdd => Key::KpAdd,
        KeyCode::KpEnter => Key::KpEnter,
        KeyCode::KpEqual => Key::KpEqual,
        KeyCode::LeftShift => Key::LeftShift,
        KeyCode::LeftControl => Key::LeftControl,
        KeyCode::LeftAlt => Key::LeftAlt,
        KeyCode::LeftSuper => Key::LeftSuper,
        KeyCode::RightShift => Key::RightShift,
        KeyCode::RightControl => Key::RightControl,
        KeyCode::RightAlt => Key::RightAlt,
        KeyCode::RightSuper => Key::RightSuper,
        KeyCode::Menu => Key::Menu,
        KeyCode::Unknown => Key::Unknown,
    }
}

fn translate_button(b: miniquad::MouseButton) -> MouseButton {
    match b {
        miniquad::MouseButton::Right => MouseButton::Right,
        miniquad::MouseButton::Left => MouseButton::Left,
        miniquad::MouseButton::Middle => MouseButton::Middle,
        miniquad::MouseButton::Unknown => MouseButton::Unknown,
    }
}

fn main() {
    let config = miniquad::conf::Conf {
        window_width: WINDOW_DIM.x,
        window_height: WINDOW_DIM.y,
        ..Default::default()
    };
    miniquad::start(config, |ctx| Box::new(Stage::new(ctx)));
}
