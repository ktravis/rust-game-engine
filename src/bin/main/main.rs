// mod stage;
use glam::{vec2, vec3, Mat4, Quat, Vec2, Vec3};
use rust_game_engine::color::Color;
use rust_game_engine::display::Display;
use rust_game_engine::font::{FontAtlas, LayoutOptions};
use rust_game_engine::input::{Key, MouseButton};
use rust_game_engine::sprite::Sprite;
use rust_game_engine::sprite_manager::SpriteManager;
use std::borrow::BorrowMut;
use std::fs::File;
use std::ops::{Deref, DerefMut, Range};
use std::{io, iter};
use ttf_parser::Face;
use wgpu::util::{DeviceExt, StagingBelt};
use wgpu::{include_wgsl, BufferAddress, BufferDescriptor, BufferSize, BufferUsages};
use winit::dpi::{PhysicalPosition, PhysicalSize, Size};
use winit::event::WindowEvent::KeyboardInput;
use winit::event::{ElementState, Event, KeyEvent, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::WindowBuilder;

use rust_game_engine::renderer::{
    BindGroup, DisplayMode, ModelInstanceData, OffscreenFramebuffer, RenderTarget, UniformBuffer,
    VertexLayout, DEFAULT_TEXTURE_DATA,
};
use slotmap::{new_key_type, SlotMap};

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
//     fn resize_event(&mut self, _ctx: &mut GraphicsContext, _width: f32, _height: f32) {}
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
//     fn char_event(
//         &mut self,
//         _ctx: &mut GraphicsContext,
//         _character: char,
//         _keymods: miniquad::KeyMods,
//         _repeat: bool,
//     ) {
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
//
//     fn touch_event(
//         &mut self,
//         ctx: &mut GraphicsContext,
//         phase: miniquad::TouchPhase,
//         _id: u64,
//         x: f32,
//         y: f32,
//     ) {
//         use miniquad::TouchPhase::*;
//         if phase == Started {
//             self.mouse_button_down_event(ctx, miniquad::MouseButton::Left, x, y);
//         }
//
//         if phase == Ended {
//             self.mouse_button_up_event(ctx, miniquad::MouseButton::Left, x, y);
//         }
//
//         if phase == Moved {
//             self.mouse_motion_event(ctx, x, y);
//         }
//     }
//
//     fn raw_mouse_motion(&mut self, _ctx: &mut GraphicsContext, _dx: f32, _dy: f32) {}
//
//     fn window_minimized_event(&mut self, _ctx: &mut GraphicsContext) {}
//
//     fn window_restored_event(&mut self, _ctx: &mut GraphicsContext) {}
//
//     fn quit_requested_event(&mut self, _ctx: &mut GraphicsContext) {}
//
//     fn files_dropped_event(&mut self, _ctx: &mut GraphicsContext) {}
// }

fn translate_key(k: KeyCode) -> Key {
    match k {
        KeyCode::Space => Key::Space,
        KeyCode::Quote => Key::Apostrophe,
        KeyCode::Comma => Key::Comma,
        KeyCode::Minus => Key::Minus,
        KeyCode::Period => Key::Period,
        KeyCode::Slash => Key::Slash,
        KeyCode::Digit0 => Key::Key0,
        KeyCode::Digit1 => Key::Key1,
        KeyCode::Digit2 => Key::Key2,
        KeyCode::Digit3 => Key::Key3,
        KeyCode::Digit4 => Key::Key4,
        KeyCode::Digit5 => Key::Key5,
        KeyCode::Digit6 => Key::Key6,
        KeyCode::Digit7 => Key::Key7,
        KeyCode::Digit8 => Key::Key8,
        KeyCode::Digit9 => Key::Key9,
        KeyCode::Semicolon => Key::Semicolon,
        KeyCode::Equal => Key::Equal,
        KeyCode::KeyA => Key::A,
        KeyCode::KeyB => Key::B,
        KeyCode::KeyC => Key::C,
        KeyCode::KeyD => Key::D,
        KeyCode::KeyE => Key::E,
        KeyCode::KeyF => Key::F,
        KeyCode::KeyG => Key::G,
        KeyCode::KeyH => Key::H,
        KeyCode::KeyI => Key::I,
        KeyCode::KeyJ => Key::J,
        KeyCode::KeyK => Key::K,
        KeyCode::KeyL => Key::L,
        KeyCode::KeyM => Key::M,
        KeyCode::KeyN => Key::N,
        KeyCode::KeyO => Key::O,
        KeyCode::KeyP => Key::P,
        KeyCode::KeyQ => Key::Q,
        KeyCode::KeyR => Key::R,
        KeyCode::KeyS => Key::S,
        KeyCode::KeyT => Key::T,
        KeyCode::KeyU => Key::U,
        KeyCode::KeyV => Key::V,
        KeyCode::KeyW => Key::W,
        KeyCode::KeyX => Key::X,
        KeyCode::KeyY => Key::Y,
        KeyCode::KeyZ => Key::Z,
        KeyCode::BracketLeft => Key::LeftBracket,
        KeyCode::Backslash => Key::Backslash,
        KeyCode::BracketRight => Key::RightBracket,
        KeyCode::Backquote => Key::GraveAccent,
        KeyCode::Escape => Key::Escape,
        KeyCode::Enter => Key::Enter,
        KeyCode::Tab => Key::Tab,
        KeyCode::Backspace => Key::Backspace,
        KeyCode::Insert => Key::Insert,
        KeyCode::Delete => Key::Delete,
        KeyCode::ArrowRight => Key::Right,
        KeyCode::ArrowLeft => Key::Left,
        KeyCode::ArrowDown => Key::Down,
        KeyCode::ArrowUp => Key::Up,
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
        KeyCode::Numpad0 => Key::Numpad0,
        KeyCode::Numpad1 => Key::Numpad1,
        KeyCode::Numpad2 => Key::Numpad2,
        KeyCode::Numpad3 => Key::Numpad3,
        KeyCode::Numpad4 => Key::Numpad4,
        KeyCode::Numpad5 => Key::Numpad5,
        KeyCode::Numpad6 => Key::Numpad6,
        KeyCode::Numpad7 => Key::Numpad7,
        KeyCode::Numpad8 => Key::Numpad8,
        KeyCode::Numpad9 => Key::Numpad9,
        KeyCode::NumpadDecimal => Key::NumpadDecimal,
        KeyCode::NumpadDivide => Key::NumpadDivide,
        KeyCode::NumpadMultiply => Key::NumpadMultiply,
        KeyCode::NumpadSubtract => Key::NumpadSubtract,
        KeyCode::NumpadAdd => Key::NumpadAdd,
        KeyCode::NumpadEnter => Key::NumpadEnter,
        KeyCode::NumpadEqual => Key::NumpadEqual,
        KeyCode::ShiftLeft => Key::LeftShift,
        KeyCode::ControlLeft => Key::LeftControl,
        KeyCode::AltLeft => Key::LeftAlt,
        KeyCode::SuperLeft => Key::LeftSuper,
        KeyCode::ShiftRight => Key::RightShift,
        KeyCode::ControlRight => Key::RightControl,
        KeyCode::AltRight => Key::RightAlt,
        KeyCode::SuperRight => Key::RightSuper,
        KeyCode::ContextMenu => Key::Menu,
        _ => Key::Unknown,
    }
}

fn translate_button(b: winit::event::MouseButton) -> MouseButton {
    match b {
        winit::event::MouseButton::Right => MouseButton::Right,
        winit::event::MouseButton::Left => MouseButton::Left,
        winit::event::MouseButton::Middle => MouseButton::Middle,
        winit::event::MouseButton::Back => MouseButton::Back,
        winit::event::MouseButton::Forward => MouseButton::Forward,
        winit::event::MouseButton::Other(id) => MouseButton::Other(id),
    }
}

use rust_game_engine::geom::{cube, quad, ModelVertexData, Point, Rect};
use rust_game_engine::texture::{Texture, TextureBuilder};
use rust_game_engine::transform::Transform;
use winit::window::Window;

#[repr(C)]
#[derive(Default, Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct DefaultUniforms {
    view: [[f32; 4]; 4],
    projection: [[f32; 4]; 4],
    time: f32,
    _pad: [u8; 12],
}

pub struct Mesh {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
}

pub trait LoadMesh {
    type Error: std::fmt::Debug;

    fn load_mesh(&self, verts: &[ModelVertexData], indices: &[u16]) -> Result<Mesh, Self::Error>;

    fn load_quad_mesh(&self) -> Mesh {
        self.load_mesh(
            &quad::verts(0., 0., 1., 1., (0., 0.), (1., 1.)),
            &quad::INDICES,
        )
        .unwrap()
    }

    fn load_cube_mesh(&self) -> Mesh {
        self.load_mesh(&cube::VERTICES, &cube::INDICES).unwrap()
    }
}

// SpriteManager will be more complicated because it needs to rebuild lazily after loading

new_key_type! {
    pub struct MeshRef;
    pub struct TextureRef;
    pub struct PipelineRef;
}

impl LoadMesh for wgpu::Device {
    type Error = ();
    fn load_mesh(&self, verts: &[ModelVertexData], indices: &[u16]) -> Result<Mesh, Self::Error> {
        Ok(Mesh {
            vertex_buffer: self.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(verts),
                usage: BufferUsages::VERTEX,
            }),
            index_buffer: self.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(indices),
                usage: BufferUsages::INDEX,
            }),
            num_indices: indices.len() as _,
        })
    }
}

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

pub type BoundTexture = BindGroup<Texture>;

#[derive(Debug)]
pub struct InstanceRenderData {
    texture: Option<TextureRef>,
    pipeline: PipelineRef,
    mesh: MeshRef,
    model: ModelInstanceData,
}

impl Deref for InstanceRenderData {
    type Target = ModelInstanceData;

    fn deref(&self) -> &Self::Target {
        &self.model
    }
}

pub trait DrawInstance {
    fn draw_instance(&mut self, instance: &InstanceRenderData);
}

// 'enc: lifetime of the encoder itself (and the mutable reference it holds to a RenderPass)
// 'pass: the lifetime of the RenderPass this encoder operates on, as well as vertex buffers and
//   other render state. This must be at least as long as 'enc.
pub struct InstanceEncoder<'enc, 'pass: 'enc> {
    buffer_view: wgpu::QueueWriteBufferView<'pass>,
    render_pass: &'enc mut RenderPass<'pass>,

    start_index: u32,
    count: u32,
    active_texture: Option<TextureRef>,
    active_mesh: Option<MeshRef>,
    active_pipeline: Option<PipelineRef>,
}

impl<'pass> Deref for InstanceEncoder<'_, 'pass> {
    type Target = RenderPass<'pass>;

    fn deref(&self) -> &Self::Target {
        &self.render_pass
    }
}

impl<'pass> DerefMut for InstanceEncoder<'_, 'pass> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.render_pass
    }
}

impl<'enc, 'pass: 'enc> InstanceEncoder<'enc, 'pass> {
    pub fn new(
        display: &'pass Display,
        render_pass: &'enc mut RenderPass<'pass>,
        instance_buffer: &'pass wgpu::Buffer,
    ) -> Self {
        render_pass.set_vertex_buffer(1, instance_buffer.slice(..));

        let buffer_view = display
            .queue()
            .write_buffer_with(
                instance_buffer,
                0,
                wgpu::BufferSize::new(instance_buffer.size()).unwrap(),
            )
            .unwrap(); // this should never fail, since we use the instance_buffer size and
                       // offset zero
        Self {
            buffer_view,
            render_pass,
            start_index: 0,
            count: 0,
            active_texture: None,
            active_mesh: None,
            active_pipeline: None,
        }
    }
}

impl<'enc, 'pass: 'enc> DrawInstance for InstanceEncoder<'enc, 'pass> {
    fn draw_instance(&mut self, instance: &InstanceRenderData) {
        // TODO: handling out of buffer space
        let dst = &mut bytemuck::cast_slice_mut(&mut self.buffer_view)
            [(self.start_index + self.count) as usize];
        if Some(instance.pipeline) != self.active_pipeline {
            if self.count > 0 {
                self.render_pass
                    .draw_active_mesh(self.start_index..self.start_index + self.count);
                self.start_index += self.count;
                self.count = 0;
            }
            self.render_pass.set_active_pipeline(instance.pipeline);
            self.active_pipeline = Some(instance.pipeline);
        }
        if Some(instance.mesh) != self.active_mesh {
            if self.count > 0 {
                self.render_pass
                    .draw_active_mesh(self.start_index..self.start_index + self.count);
                self.start_index += self.count;
                self.count = 0;
            }
            self.render_pass.set_active_mesh(instance.mesh);
            self.active_mesh = Some(instance.mesh);
        }
        if instance.texture != self.active_texture {
            if self.count > 0 {
                self.render_pass
                    .draw_active_mesh(self.start_index..self.start_index + self.count);
                self.start_index += self.count;
                self.count = 0;
            }
            self.render_pass.bind_texture(instance.texture);
            self.active_texture = instance.texture;
        }
        self.count += 1;
        *dst = instance.as_raw();
    }
}

impl Drop for InstanceEncoder<'_, '_> {
    fn drop(&mut self) {
        if self.count > 0 {
            self.render_pass
                .draw_active_mesh(self.start_index..self.start_index + self.count);
        }
    }
}

pub struct RenderPass<'a> {
    pub render_state: &'a RenderState,
    raw_pass: wgpu::RenderPass<'a>,
    active_mesh: Option<MeshRef>,
    active_pipeline: Option<PipelineRef>,
}

impl<'a> Deref for RenderPass<'a> {
    type Target = wgpu::RenderPass<'a>;

    fn deref(&self) -> &Self::Target {
        &self.raw_pass
    }
}

impl<'a> DerefMut for RenderPass<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.raw_pass
    }
}

impl<'a> RenderPass<'a> {
    const TEXTURE_BIND_GROUP_INDEX: u32 = 0;
    const DEFAULT_UNIFORMS_BIND_GROUP_INDEX: u32 = 1;

    pub fn set_active_pipeline(&mut self, pipeline: PipelineRef) {
        if Some(pipeline) == self.active_pipeline {
            return;
        }
        let p = self.render_state.pipelines.get(pipeline).unwrap();
        self.set_pipeline(p);
        self.active_pipeline = Some(pipeline);
    }

    pub fn set_active_mesh(&mut self, mesh: MeshRef) {
        self.active_mesh = Some(mesh);
    }

    pub fn draw_active_mesh(&mut self, instances: Range<u32>) {
        let mesh = self
            .render_state
            .mesh_manager
            .get(self.active_mesh.expect("no active mesh"))
            .unwrap();
        self.raw_pass
            .set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.raw_pass
            .set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        self.raw_pass
            .draw_indexed(0..mesh.num_indices, 0, instances);
    }

    pub fn bind_texture_data(&mut self, texture_data: &'a BoundTexture) {
        self.raw_pass.set_bind_group(
            Self::TEXTURE_BIND_GROUP_INDEX,
            texture_data.bind_group(),
            &[],
        );
    }

    pub fn bind_texture(&mut self, texture: impl Into<Option<TextureRef>>) {
        self.bind_texture_data(self.render_state.get_texture(texture.into()))
    }

    pub fn instance_encoder<'enc>(
        &'enc mut self,
        display: &'a Display,
        buffer: &'a wgpu::Buffer,
    ) -> InstanceEncoder<'enc, 'a> {
        InstanceEncoder::new(display, self, buffer)
    }
}

pub struct RenderState {
    default_uniforms: BindGroup<UniformBuffer<DefaultUniforms>>,

    uniform_bind_group_layout: wgpu::BindGroupLayout,
    texture_bind_group_layout: wgpu::BindGroupLayout,

    texture_manager: SlotMap<TextureRef, BoundTexture>,
    default_texture: BoundTexture,

    mesh_manager: SlotMap<MeshRef, Mesh>,
    pipelines: SlotMap<PipelineRef, wgpu::RenderPipeline>,
}

impl RenderState {
    pub fn new(display: &Display) -> Self {
        let device = display.device();
        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });
        let uniform_bind_group_layout = UniformBuffer::<DefaultUniforms>::bind_group_layout(
            device,
            "uniform_bind_group_layout",
            wgpu::ShaderStages::VERTEX_FRAGMENT,
        );
        let default_uniforms = BindGroup::new(
            device,
            &uniform_bind_group_layout,
            UniformBuffer::new(
                device,
                DefaultUniforms {
                    view: Mat4::look_at_lh(vec3(0.0, 1.0, 6.0), Vec3::ZERO, Vec3::Y)
                        .to_cols_array_2d(),
                    projection: Mat4::perspective_lh(
                        45.0_f32.to_radians(),
                        960.0 / 720.0,
                        0.01,
                        100.0,
                    )
                    .to_cols_array_2d(),
                    time: 0.0,
                    _pad: Default::default(),
                },
            ),
        );

        let texture_manager = SlotMap::with_key();
        let default_texture = BoundTexture::new(
            display.device(),
            &texture_bind_group_layout,
            TextureBuilder::render_target()
                .with_label("default_texture")
                .from_raw_bytes(
                    display.device(),
                    display.queue(),
                    &DEFAULT_TEXTURE_DATA,
                    Point::new(2, 2),
                ),
        );
        let mesh_manager = SlotMap::with_key();
        let pipelines = SlotMap::with_key();
        Self {
            default_uniforms,
            texture_bind_group_layout,
            uniform_bind_group_layout,
            texture_manager,
            default_texture,
            mesh_manager,
            pipelines,
        }
    }

    pub fn create_pipeline<'a>(
        &mut self,
        name: &str,
        display: &Display,
        shader: &wgpu::ShaderModule,
        vertex_layouts: &[wgpu::VertexBufferLayout<'a>],
    ) -> PipelineRef {
        // TODO: do we need/want to dedupe or cache this?
        let layout = display
            .device()
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(&format!("{} Layout", name)),
                bind_group_layouts: &[
                    &self.texture_bind_group_layout,
                    &self.uniform_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });
        let pipeline = display
            .device()
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(name),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: shader,
                    entry_point: "vs_main",
                    buffers: vertex_layouts,
                },
                fragment: Some(wgpu::FragmentState {
                    module: shader,
                    entry_point: "fs_main",
                    // TODO: technically I think this should be some `for_render_targets` slice
                    // maybe `PipelineBuilder` eventually
                    targets: &[Some(wgpu::ColorTargetState {
                        format: display.format(),
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent {
                                src_factor: wgpu::BlendFactor::SrcAlpha,
                                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                                operation: wgpu::BlendOperation::Add,
                            },
                            alpha: wgpu::BlendComponent::OVER,
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    // Requires Features::DEPTH_CLIP_CONTROL
                    unclipped_depth: false,
                    // Requires Features::CONSERVATIVE_RASTERIZATION
                    conservative: false,
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: TextureBuilder::DEFAULT_DEPTH_FORMAT,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: Default::default(),
                    bias: Default::default(),
                }),
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                // If the pipeline will be used with a multiview render pass, this
                // indicates how many array layers the attachments will have.
                multiview: None,
            });
        self.pipelines.insert(pipeline)
    }

    pub fn create_vertex_buffers<'a, const N: usize>(
        &mut self,
        device: &wgpu::Device,
        vertex_layouts: [wgpu::VertexBufferLayout<'a>; N],
    ) -> [wgpu::Buffer; N] {
        std::array::from_fn(|i| {
            let layout = &vertex_layouts[i];
            device.create_buffer(&BufferDescriptor {
                label: None,                      // TODO: pass a name through
                size: 1024 * layout.array_stride, // TODO: pass the count through as well
                usage: BufferUsages::COPY_DST | BufferUsages::VERTEX,
                mapped_at_creation: false,
            })
        })
    }

    pub fn begin_render_pass<'pass>(
        &'pass self,
        encoder: &'pass mut wgpu::CommandEncoder,
        desc: &wgpu::RenderPassDescriptor<'pass, '_>,
    ) -> RenderPass<'pass> {
        let mut raw_pass = encoder.begin_render_pass(desc);
        raw_pass.set_bind_group(
            RenderPass::DEFAULT_UNIFORMS_BIND_GROUP_INDEX,
            self.default_uniforms.bind_group(),
            &[],
        );
        RenderPass {
            raw_pass,
            render_state: self,
            active_mesh: None,
            active_pipeline: None,
        }
    }

    pub fn load_texture(&mut self, display: &Display, t: Texture) -> TextureRef {
        self.texture_manager.insert(BoundTexture::new(
            display.device(),
            &self.texture_bind_group_layout,
            t,
        ))
    }

    pub fn get_texture(&self, texture: impl Into<Option<TextureRef>>) -> &BoundTexture {
        texture
            .into()
            .map(|t| self.texture_manager.get(t).unwrap())
            .unwrap_or(&self.default_texture)
    }

    pub fn replace_texture(&mut self, display: &Display, texture_ref: TextureRef, value: Texture) {
        *self.texture_manager.get_mut(texture_ref).unwrap() =
            BoundTexture::new(display.device(), &self.texture_bind_group_layout, value);
    }

    pub fn prepare_mesh(&mut self, mesh: Mesh) -> MeshRef {
        self.mesh_manager.insert(mesh)
    }
}

#[derive(Clone, Copy, Default, Debug)]
pub struct TextDisplayOptions {
    pub color: Color,
    pub layout: LayoutOptions,
}

pub trait DrawText {
    fn draw_text(&mut self, s: impl AsRef<str>, transform: Transform, opts: TextDisplayOptions);
    fn draw_text_2d(&mut self, s: impl AsRef<str>, position: Vec2, opts: TextDisplayOptions) {
        self.draw_text(
            s,
            Transform {
                position: position.extend(0.0),
                ..Default::default()
            },
            opts,
        )
    }
}

pub struct FontFaceRenderer<'a, T: DrawInstance> {
    raw: &'a mut T,
    font_atlas: &'a FontAtlas,
    pipeline: PipelineRef,
    texture: TextureRef,
    quad_mesh: MeshRef,
}

impl<'a, T: DrawInstance> DrawText for FontFaceRenderer<'a, T> {
    fn draw_text(&mut self, s: impl AsRef<str>, transform: Transform, opts: TextDisplayOptions) {
        for glyph_data in self.font_atlas.layout_text(s.as_ref(), opts.layout) {
            self.raw.draw_instance(&InstanceRenderData {
                texture: Some(self.texture),
                pipeline: self.pipeline,
                mesh: self.quad_mesh,
                model: ModelInstanceData {
                    subtexture: glyph_data.subtexture,
                    tint: opts.color,
                    transform: Transform {
                        position: glyph_data.bounds.pos.extend(0.0) * transform.scale
                            + transform.position,
                        scale: glyph_data.bounds.dim.extend(1.0) * transform.scale,
                        ..transform
                    },
                    ..Default::default()
                },
            });
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
    text_render_pipeline: PipelineRef,
    font_atlas: FontAtlas,
    font_atlas_texture: TextureRef,

    quad_mesh: MeshRef,
    cube_mesh: MeshRef,
    sprite_manager: SpriteManager,
    sprite_atlas_texture: TextureRef,

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
            &render_state.texture_bind_group_layout,
            Point::new(960, 720),
        );

        Ok(Self {
            display,
            render_state,
            quad_mesh,
            cube_mesh,
            diffuse_texture,
            diffuse_texture2,
            font_atlas_texture,
            cursor_position: Default::default(),
            camera: Camera {
                position: vec3(0.0, 2.3, 6.0),
                ..Default::default()
            },
            offscreen_framebuffer,
            instances,
            instance_buffer,
            instanced_render_pipeline,
            text_render_pipeline,
            to_screen_pipeline,
            sprite_manager,
            sprite_atlas_texture: sprite_atlas,
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
                        texture: Some(self.sprite_atlas_texture),
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
            self.sprite_atlas_texture = self.render_state.load_texture(
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
                view: self.camera.view_matrix().to_cols_array_2d(),
                projection: Mat4::perspective_lh(
                    self.camera.fov_radians,
                    960.0 / 720.0,
                    0.01,
                    100.0,
                ) // TODO add near/far/aspect to camera
                .to_cols_array_2d(),
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
            let mut text_renderer = FontFaceRenderer {
                raw: &mut instance_encoder,
                font_atlas: &self.font_atlas,
                texture: self.font_atlas_texture,
                pipeline: self.text_render_pipeline,
                quad_mesh: self.quad_mesh,
            };
            text_renderer.draw_text(
                "howdy there",
                Transform {
                    position: vec3(1.0, 2.0, 3.0),
                    scale: vec3(0.1, -0.1, -1.0),
                    rotation: Quat::IDENTITY,
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
                view: DisplayMode::Centered
                    .scaling_matrix(
                        self.offscreen_framebuffer.size_pixels().as_vec2(),
                        target_size,
                    )
                    .to_cols_array_2d(),
                projection: Mat4::orthographic_lh(
                    0.0,
                    target_size.x,
                    target_size.y,
                    0.0,
                    1.0,
                    -1.0,
                )
                .to_cols_array_2d(),
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
                view: Mat4::IDENTITY.to_cols_array_2d(),
                projection: Mat4::orthographic_lh(
                    0.0,
                    target_size.x,
                    target_size.y,
                    0.0,
                    1.0,
                    -1.0,
                )
                .to_cols_array_2d(),
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
                // draw_sprite
                let s = self
                    .sprite_manager
                    .get_sprite(self.sprite_manager.get_sprite_ref("guy").unwrap());
                let frame = &s.frames[0];
                // let origin = pos - s.pivot.unwrap_or_default().as_vec2();
                // let pos = origin;
                let scale = 4. * vec3(s.size.x as f32, s.size.y as f32, 1.0);
                instance_encoder.draw_instance(&InstanceRenderData {
                    texture: Some(self.sprite_atlas_texture),
                    pipeline: self.instanced_render_pipeline,
                    mesh: self.quad_mesh,
                    model: ModelInstanceData {
                        subtexture: frame.region,
                        transform: Transform {
                            position: vec3(
                                self.cursor_position.unwrap_or_default().x as _,
                                self.cursor_position.unwrap_or_default().y as _,
                                0.,
                            ),
                            scale,
                            rotation: Quat::IDENTITY,
                        },
                        ..Default::default()
                    },
                });
            }
            let mut text_renderer = FontFaceRenderer {
                raw: &mut instance_encoder,
                font_atlas: &self.font_atlas,
                texture: self.font_atlas_texture,
                pipeline: self.text_render_pipeline,
                quad_mesh: self.quad_mesh,
            };
            text_renderer.draw_text_2d(
                format!("{}", self.camera.position),
                vec2(10.0, 32.0),
                TextDisplayOptions::default(),
            );
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
