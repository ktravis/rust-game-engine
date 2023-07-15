use color::Color;
use miniquad::*;
use msdfgen::Vector2;
use std::{path::Path, time::Duration};
use transform::Transform2D;

mod assets;
mod atlas;
mod color;
mod default_shader;
mod font;
mod geom;
mod inputs;
mod mesh;
mod model;
mod renderer;
mod resources;
mod sprite;
mod text_shader;
mod transform;

use assets::{Assets, SpriteRef};

use geom::Point;
use renderer::{DisplayMode, InstanceData, RenderPassOptions, Renderer};

use glam::{vec2, vec3, Mat4, Quat, Vec2, Vec3};

const WINDOW_DIM: Point = Point { x: 960, y: 720 };
const TARGET_FRAMERATE: u64 = 60;

struct Stage {
    camera_offset: Vec2,
    renderer: Renderer,
    input: inputs::Input,
    input_axis_x: inputs::BindingRef,
    input_axis_y: inputs::BindingRef,
    xy: Vec2,
    sprite_atlas_texture: Texture,
    assets: Assets,
    frame_index: usize,
    s: SpriteRef,
    sprite_pos: Vec<Point>,
    /// Last frame start time in seconds
    frame_start: f64,
    /// Delta from last frame start in seconds
    delta: f32,
    frame_counter: usize,
    frame_timer: f32,
    target_frame_duration: f64,
    crate_texture: Texture,
    render_target_size_px: Point<u32>,
    angle: f32,
    render_scale: f32,
}

impl Stage {
    pub fn new(ctx: &mut Context) -> Stage {
        // check features
        {
            assert!(ctx.features().instancing, "instancing is not supported");
        }

        const SCALE: u32 = 1;

        let render_target_size_px: Point<u32> = (240 * SCALE, 180 * SCALE).into();
        let render_target_texture = Texture::new_render_texture(
            ctx,
            TextureParams {
                width: render_target_size_px.x,
                height: render_target_size_px.y,
                format: TextureFormat::RGBA8,
                filter: FilterMode::Nearest,
                ..Default::default()
            },
        );

        let assets = Assets::new(Path::new("./res"));

        let sprite_atlas_texture = resources::texture_from_image(ctx, assets.atlas.image());

        let crate_texture = resources::texture_from_image(
            ctx,
            &image::open(Path::new("res/images/crate.png"))
                .unwrap()
                .into_rgba8(),
        );

        let shader = Shader::new(
            ctx,
            default_shader::VERTEX,
            default_shader::FRAGMENT,
            default_shader::meta(),
        )
        .expect("default shader creation failed");

        let depth_texture = Texture::new_render_texture(
            ctx,
            TextureParams {
                width: render_target_size_px.x as _,
                height: render_target_size_px.y as _,
                format: TextureFormat::Depth,
                ..Default::default()
            },
        );

        let renderer = Renderer::new(
            ctx,
            render_target_texture,
            depth_texture,
            // None,
            shader,
            DisplayMode::Centered,
        );

        let mut input = inputs::Input::default();
        let input_axis_x = input
            .new_axis("x")
            .with_keys(KeyCode::A, KeyCode::D)
            .with_keys(KeyCode::Left, KeyCode::Right)
            .register();
        let input_axis_y = input
            .new_axis("y")
            .with_keys(KeyCode::W, KeyCode::S)
            .with_keys(KeyCode::Up, KeyCode::Down)
            .register();
        input
            .new_axis("scale")
            .with_keys(KeyCode::F, KeyCode::R)
            .register();
        input.register_new_button("quit", &[KeyCode::Escape]);
        input.register_new_button("next", &[KeyCode::P]);
        input
            .new_button("add")
            .with_key(KeyCode::O)
            .with_key(MouseButton::Left)
            .register();

        let sprite_pos = (0..10)
            .map(|_| {
                Point::new(
                    (rand::random::<u32>() % render_target_size_px.x) as i32,
                    (rand::random::<u32>() % render_target_size_px.y) as i32,
                )
            })
            .collect();
        let s = assets.get_sprite_ref("guy").unwrap();
        Stage {
            sprite_pos,
            renderer,
            input,
            input_axis_x,
            input_axis_y,
            camera_offset: Default::default(),
            xy: Vec2::default(),
            sprite_atlas_texture,
            assets,
            frame_index: 0,
            s,
            frame_start: -1.0,
            delta: 0.,
            frame_counter: 0,
            frame_timer: 0.,
            target_frame_duration: 1. / TARGET_FRAMERATE as f64,
            crate_texture,
            render_target_size_px,
            angle: 0.,
            render_scale: 1.,
        }
    }

    fn init(&mut self) {
        // self.assets.watch(Asset::Sprite("guy".into()));
    }

    fn update(&mut self, ctx: &mut GraphicsContext) -> bool {
        if self.assets.check_for_updates() {
            self.sprite_atlas_texture =
                resources::texture_from_image(ctx, self.assets.atlas.image());
        }

        self.input.update();

        if self.input.button_by_name("quit").state.just_pressed {
            return false;
        }

        self.xy.x += 100. * self.delta * self.input.axis(self.input_axis_x).value();
        self.xy.y += 100. * self.delta * self.input.axis(self.input_axis_y).value();
        self.render_scale = (self.render_scale
            + 20. * self.delta * self.input.axis_by_name("scale").value())
        .clamp(0.5, 40.);

        if self.input.button_by_name("next").just_pressed() {
            self.frame_index = (self.frame_index + 1) % self.assets.get_sprite(self.s).frames.len();
        }
        if self.input.button_by_name("add").is_down() {
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
        if self.frame_counter >= 50 {
            println!("fps: {:.1}", self.frame_counter as f32 / self.frame_timer);
            self.frame_counter = 0;
            self.frame_timer = 0.0;
        }
        {
            let mut r = self
                .renderer
                .begin_offscreen_pass(ctx, RenderPassOptions::clear(Color::from(0x3f3f74ffu32)));
            r.push_transform(Mat4::from_translation(self.camera_offset.extend(1.0)));
            r.set_texture(self.sprite_atlas_texture);

            let s = self.assets.get_sprite(self.s);
            let offset = Point::new(self.xy.x.floor() as _, self.xy.y.floor() as _);
            for xy in &self.sprite_pos {
                r.draw_sprite_frame(offset + *xy, s, self.frame_index);
            }
        }

        // draw a screen-sized quad using the previously rendered offscreen render-target as texture
        self.renderer.draw_to_screen(ctx);

        {
            let mut r = self.renderer.begin_screen_pass(
                ctx,
                RenderPassOptions::clear_depth(1.)
                    .with_projection(glam::Mat4::perspective_lh(
                        60f32.to_radians(),
                        WINDOW_DIM.x as f32 / WINDOW_DIM.y as f32,
                        0.01,
                        100.,
                    ))
                    .with_view_transform(Mat4::look_at_lh(
                        vec3(self.xy.x / 10., 1., self.xy.y / 10. - 4.),
                        vec3(0., 0., 0.),
                        vec3(0., 1., 0.),
                    )),
            );
            r.set_texture(self.crate_texture);
            r.draw_cube(InstanceData {
                transform: transform::Transform3D {
                    rotation: Quat::from_rotation_x(self.angle) * Quat::from_rotation_y(self.angle),
                    ..Default::default()
                },
                ..Default::default()
            });
        }

        {
            let s = "abcdefghijkl";
            let scale = self.render_scale.clamp(1., 20.);
            let glyphs = s
                .chars()
                .map(|c| (self.renderer.font_atlas.glyph_data(c)))
                .collect::<Vec<font::GlyphData>>();
            let mut r = self.renderer.begin_text_pass(
                ctx,
                PassAction::Clear {
                    color: None,
                    depth: Some(1.),
                    stencil: None,
                },
                Mat4::IDENTITY,
                glam::Mat4::orthographic_lh(
                    0.0,
                    WINDOW_DIM.x as f32,
                    WINDOW_DIM.y as f32,
                    0.0,
                    1.0,
                    -1.0,
                ),
            );
            let mut pos = glam::vec2(15. + self.xy.x, 430. + self.xy.y);
            for glyph_data in glyphs {
                let offset = glyph_data.bounds.pos * scale;
                let glyph_quad_size = glyph_data.bounds.dim * scale;

                r.draw_quad(InstanceData::<Transform2D> {
                    transform: Transform2D {
                        pos: pos + offset,
                        scale: glyph_quad_size,
                        angle: 0.,
                    },
                    subtexture: glyph_data.subtexture,
                    ..Default::default()
                });
                pos.x += scale * glyph_data.metrics.bounds.dim.x;
            }
        }

        ctx.commit_frame();
    }
}

impl EventHandler for Stage {
    fn update(&mut self, ctx: &mut Context) {
        self.frame_counter += 1;
        let time = miniquad::date::now();
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
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        keycode: KeyCode,
        _keymods: KeyMods,
        _repeat: bool,
    ) {
        self.input
            .handle_key_or_button_change(keycode, inputs::StateChange::Pressed);
    }

    fn key_up_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymods: KeyMods) {
        self.input
            .handle_key_or_button_change(keycode, inputs::StateChange::Released);
    }

    fn resize_event(&mut self, _ctx: &mut Context, _width: f32, _height: f32) {}

    fn draw(&mut self, ctx: &mut Context) {
        Stage::draw(self, ctx);
        // let current_frame_time = miniquad::date::now() - self.frame_start + 0.0005;
        // if current_frame_time < self.target_frame_duration {
        //     std::thread::sleep(Duration::from_secs_f64(
        //         self.target_frame_duration - current_frame_time,
        //     ))
        // }
    }

    fn mouse_motion_event(&mut self, _ctx: &mut Context, x: f32, y: f32) {
        self.input.handle_mouse_motion(x, y)
    }

    fn mouse_wheel_event(&mut self, _ctx: &mut Context, _x: f32, _y: f32) {}

    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        button: MouseButton,
        _x: f32,
        _y: f32,
    ) {
        self.input
            .handle_key_or_button_change(button, inputs::StateChange::Pressed);
    }

    fn mouse_button_up_event(&mut self, _ctx: &mut Context, button: MouseButton, _x: f32, _y: f32) {
        self.input
            .handle_key_or_button_change(button, inputs::StateChange::Released);
    }

    fn char_event(
        &mut self,
        _ctx: &mut Context,
        _character: char,
        _keymods: KeyMods,
        _repeat: bool,
    ) {
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
    miniquad::start(config, |ctx| Box::new(Stage::new(ctx)));
}
