use controlset_derive::ControlSet;
use miniquad::*;
use rust_game_engine::font::{FontAtlas, LayoutOptions};
use std::cell::Cell;
use std::path::Path;
use std::rc::Rc;
use ttf_parser::Face;

use rust_game_engine::{color::Color, resources, sprite_manager, transform};

use rust_game_engine::geom::{cube, quad, ModelVertexData, Point};
use rust_game_engine::input::{self, AnalogInput::*, Axis, Button, InputManager, KeyCode as Key};
use rust_game_engine::renderer::{
    BasicRenderPipeline, DisplayMode, InstancedRenderPipeline, ModelInstanceData,
    OffscreenFramebuffer, RawInstanceData, RenderPassOptions, RenderTarget, TextDisplayOptions,
};

use glam::{vec3, Mat4, Quat, Vec2};
use rust_game_engine::assets::AssetManager;
use rust_game_engine::mesh_manager::{GeometryBuffers, MeshManager, MeshOffsets};
use rust_game_engine::shader::Shader;
use rust_game_engine::sprite_manager::SpriteManager;

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
    blah: Button,
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

#[derive(Debug, Default)]
struct GameAssets {
    shader_sources: ShaderSource,
    game_font_face: Vec<u8>,
    sprite_manager: SpriteManager,
}

struct Stage {
    camera_offset: Vec2,
    font_atlas: FontAtlas,
    input: InputManager<Controls>,
    xy: Vec2,
    sprite_atlas_texture: Texture,
    frame_index: usize,
    s: sprite_manager::SpriteRef,
    sprite_pos: Vec<Point>,
    /// Last frame start time in seconds
    frame_start: f64,
    /// Delta from last frame start in seconds
    delta: f32,
    frame_counter: usize,
    frame_timer: f32,
    sampled_fps: f32,
    target_frame_duration: f64,
    crate_texture: Texture,
    render_target_size_px: Point<u32>,
    angle: f32,
    render_scale: f32,
    backbuffer_target: OffscreenFramebuffer,
    render_pipelines: RenderPipelines,
    mesh_manager: MeshManager<ModelVertexData>,
    font_texture: Texture,
    quad_mesh: Rc<Cell<MeshOffsets>>,
    cube_mesh: Rc<Cell<MeshOffsets>>,
    geometry_buffers: GeometryBuffers<ModelVertexData>,

    asset_manager: AssetManager<GameAssets>,
}

const SCALE: u32 = 1;

fn build_render_pipelines(
    ctx: &mut GraphicsContext,
    shader_sources: &ShaderSource,
) -> RenderPipelines {
    let basic_shader = Shader::new(
        ctx,
        &shader_sources.default_vert,
        &shader_sources.default_frag,
        vec!["tex".into()],
    )
    .expect("default shader creation failed");
    let model_shader = Shader::new(
        ctx,
        &shader_sources.model_vert,
        &shader_sources.model_frag,
        vec!["tex".into()],
    )
    .expect("model shader creation failed");
    let text_shader = Shader::new(
        ctx,
        &shader_sources.text_vert,
        &shader_sources.text_frag,
        vec!["tex".into()],
    )
    .expect("text shader creation failed");
    RenderPipelines {
        basic: BasicRenderPipeline::new(ctx, basic_shader),
        model: InstancedRenderPipeline::new(ctx, model_shader),
        text: InstancedRenderPipeline::new(ctx, text_shader),
    }
}

impl Stage {
    pub fn new(ctx: &mut Context) -> Stage {
        // check features
        {
            assert!(ctx.features().instancing, "instancing is not supported");
        }

        let crate_texture = resources::texture_from_image(
            ctx,
            &image::open(Path::new("res/images/crate.png"))
                .unwrap()
                .into_rgba8(),
        );

        let b = std::fs::read("./res/fonts/Ubuntu-M.ttf").unwrap();
        let face = Face::parse(&b, 0).unwrap();
        let font_atlas = FontAtlas::new(face, Default::default()).unwrap();
        let font_texture = resources::texture_from_image_with_params(
            ctx,
            font_atlas.image(),
            TextureParams {
                format: TextureFormat::RGBA8,
                wrap: TextureWrap::Clamp,
                filter: FilterMode::Linear,
                width: font_atlas.image().width(),
                height: font_atlas.image().height(),
            },
        );

        let mut asset_manager = AssetManager::new(GameAssets::default(), "./res/");

        asset_manager.track_glob("./res/sprites/*.aseprite", |state, path, f| {
            state.sprite_manager.add_sprite_file(path.to_path_buf(), f);
        });
        // TODO: should have a way to not need this
        let sprite_atlas_texture = {
            asset_manager.sprite_manager.maybe_rebuild();
            resources::texture_from_image(ctx, asset_manager.sprite_manager.atlas.image())
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

        let render_pipelines = build_render_pipelines(ctx, &asset_manager.shader_sources);

        let render_target_size_px: Point<u32> = (240 * SCALE, 180 * SCALE).into();
        let depth_texture = Texture::new_render_texture(
            ctx,
            TextureParams {
                width: render_target_size_px.x as _,
                height: render_target_size_px.y as _,
                format: TextureFormat::Depth,
                ..Default::default()
            },
        );
        let backbuffer_color_texture = Texture::new_render_texture(
            ctx,
            TextureParams {
                width: render_target_size_px.x,
                height: render_target_size_px.y,
                format: TextureFormat::RGBA8,
                filter: FilterMode::Nearest,
                ..Default::default()
            },
        );
        let backbuffer_target =
            OffscreenFramebuffer::new(ctx, backbuffer_color_texture, depth_texture);

        let sprite_pos = (0..10)
            .map(|_| {
                Point::new(
                    (rand::random::<u32>() % render_target_size_px.x) as i32,
                    (rand::random::<u32>() % render_target_size_px.y) as i32,
                )
            })
            .collect();
        let s = asset_manager.sprite_manager.get_sprite_ref("guy").unwrap();
        let mut mesh_manager = MeshManager::default();
        let quad_mesh = mesh_manager.add("quad", quad::mesh());
        let cube_mesh = mesh_manager.add("cube", cube::mesh());
        let geometry_buffers = mesh_manager.buffers(ctx);
        Stage {
            sprite_pos,
            font_atlas,
            input: Default::default(),
            camera_offset: Default::default(),
            xy: Vec2::default(),
            sprite_atlas_texture,
            frame_index: 0,
            s,
            frame_start: -1.0,
            delta: 0.,
            frame_counter: 0,
            frame_timer: 0.,
            sampled_fps: 0.,
            target_frame_duration: 1. / TARGET_FRAMERATE as f64,
            crate_texture,
            render_target_size_px,
            angle: 0.,
            render_scale: 1.,
            backbuffer_target,
            mesh_manager,
            font_texture,
            quad_mesh,
            cube_mesh,
            render_pipelines,
            geometry_buffers,
            asset_manager,
        }
    }

    fn init(&mut self) {}

    fn update(&mut self, ctx: &mut GraphicsContext) -> bool {
        if self.asset_manager.check_for_updates() {
            if self.asset_manager.shader_sources.dirty {
                self.asset_manager.shader_sources.dirty = false;
                self.render_pipelines =
                    build_render_pipelines(ctx, &self.asset_manager.shader_sources);
            }
            if self.asset_manager.sprite_manager.maybe_rebuild() {
                self.sprite_atlas_texture = resources::texture_from_image(
                    ctx,
                    self.asset_manager.sprite_manager.atlas.image(),
                );
            }
        }

        if self.mesh_manager.needs_rebuild {
            self.geometry_buffers = self.mesh_manager.buffers(ctx);
        }

        if self.input.quit.just_pressed {
            return false;
        }
        if self.frame_counter >= 50 {
            self.sampled_fps = self.frame_counter as f32 / self.frame_timer;
            self.frame_counter = 0;
            self.frame_timer = 0.0;
        }

        self.xy.x += 100. * self.delta * self.input.x.value();
        self.xy.y += 100. * self.delta * self.input.y.value();
        self.render_scale =
            (self.render_scale + 20. * self.delta * self.input.scale.value()).clamp(0.5, 40.);

        if self.input.next.just_pressed() {
            self.frame_index = (self.frame_index + 1)
                % self
                    .asset_manager
                    .sprite_manager
                    .get_sprite(self.s)
                    .frames
                    .len();
        }
        if self.input.add.is_down() {
            println!("{}", self.sprite_pos.len());
            for _ in 0..100 {
                self.sprite_pos.push(Point::new(
                    (rand::random::<u32>() % self.render_target_size_px.x) as i32,
                    (rand::random::<u32>() % self.render_target_size_px.y) as i32,
                ));
            }
        }
        self.angle += self.delta;

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

            let s = self.asset_manager.sprite_manager.get_sprite(self.s);
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
            let s = format!("fps: {:.0}", self.sampled_fps);
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
                &self.font_atlas,
                TextDisplayOptions {
                    color: Color::WHITE,
                    layout: LayoutOptions::scale(scale),
                },
            );
            r.draw_text(
                self.input.mouse.position(),
                "test\nokay right? nice.",
                &self.font_atlas,
                TextDisplayOptions {
                    color: Color::WHITE,
                    layout: LayoutOptions::scale(scale),
                },
            );
        }

        ctx.commit_frame();
    }
}

impl EventHandler for Stage {
    fn update(&mut self, ctx: &mut Context) {
        self.frame_counter += 1;
        let time = date::now();
        self.delta = if self.frame_start < 0.0 {
            1.0 / 60.0
        } else {
            (time - self.frame_start) as f32
        };
        self.frame_timer += self.delta;
        self.frame_start = time;
        if !Stage::update(self, ctx) {
            ctx.request_quit();
        }
        // Do this after the frame is done updating, so we can clear state and update controls for the next frame.
        self.input.end_frame_update();
    }

    fn draw(&mut self, ctx: &mut Context) {
        Stage::draw(self, ctx);
        // let current_frame_time = miniquad::date::now() - self.frame_start + 0.0005;
        // if current_frame_time < self.target_frame_duration {
        //     std::thread::sleep(Duration::from_secs_f64(
        //         self.target_frame_duration - current_frame_time,
        //     ))
        // }
    }

    fn resize_event(&mut self, _ctx: &mut Context, _width: f32, _height: f32) {}

    fn mouse_motion_event(&mut self, _ctx: &mut Context, x: f32, y: f32) {
        self.input.handle_mouse_motion(x, y)
    }

    fn mouse_wheel_event(&mut self, _ctx: &mut Context, x: f32, y: f32) {
        if x != 0. {
            self.input.handle_analog_axis_change(MouseWheelX, x);
        }
        if y != 0. {
            self.input.handle_analog_axis_change(MouseWheelY, y);
        }
    }

    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        button: MouseButton,
        _x: f32,
        _y: f32,
    ) {
        self.input
            .handle_key_or_button_change(button, input::StateChange::Pressed);
    }

    fn mouse_button_up_event(&mut self, _ctx: &mut Context, button: MouseButton, _x: f32, _y: f32) {
        self.input
            .handle_key_or_button_change(button, input::StateChange::Released);
    }

    fn char_event(
        &mut self,
        _ctx: &mut Context,
        _character: char,
        _keymods: KeyMods,
        _repeat: bool,
    ) {
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        keycode: KeyCode,
        _keymods: KeyMods,
        _repeat: bool,
    ) {
        self.input
            .handle_key_or_button_change(keycode, input::StateChange::Pressed);
    }

    fn key_up_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymods: KeyMods) {
        self.input
            .handle_key_or_button_change(keycode, input::StateChange::Released);
    }

    fn touch_event(&mut self, ctx: &mut Context, phase: TouchPhase, _id: u64, x: f32, y: f32) {
        if phase == TouchPhase::Started {
            self.mouse_button_down_event(ctx, MouseButton::Left, x, y);
        }

        if phase == TouchPhase::Ended {
            self.mouse_button_up_event(ctx, MouseButton::Left, x, y);
        }

        if phase == TouchPhase::Moved {
            self.mouse_motion_event(ctx, x, y);
        }
    }

    fn raw_mouse_motion(&mut self, _ctx: &mut Context, _dx: f32, _dy: f32) {}

    fn window_minimized_event(&mut self, _ctx: &mut Context) {}

    fn window_restored_event(&mut self, _ctx: &mut Context) {}

    fn quit_requested_event(&mut self, _ctx: &mut Context) {}

    fn files_dropped_event(&mut self, _ctx: &mut Context) {}
}

fn main() {
    let config = conf::Conf {
        window_width: WINDOW_DIM.x,
        window_height: WINDOW_DIM.y,
        ..Default::default()
    };
    start(config, |ctx| Box::new(Stage::new(ctx)));
}
