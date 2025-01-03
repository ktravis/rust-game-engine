use std::borrow::Cow;

use wgpu::include_wgsl;
use winit::{
    application::ApplicationHandler,
    dpi::{PhysicalPosition, Size},
    event::{DeviceEvent, KeyEvent, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::PhysicalKey,
    window::Window,
};

use crate::{
    input::{AnalogInput, ControlSet, InputManager, Key, MouseButton},
    renderer::{egui::EguiRenderer, Display, RenderState},
    time::FrameTiming,
};

pub struct Context<C: ControlSet> {
    pub display: Display,
    pub render_state: RenderState,
    pub frame_timing: FrameTiming,
    pub input: InputManager<C>,
    pub egui: EguiRenderer,
}

impl<C: ControlSet> Context<C> {
    pub fn new(display: Display, render_state: RenderState) -> Self {
        let egui = EguiRenderer::new(&display, 1);
        Self {
            display,
            render_state,
            frame_timing: Default::default(),
            input: Default::default(),
            egui,
        }
    }

    pub fn handle_device_input(&mut self, event: &DeviceEvent) {
        match *event {
            winit::event::DeviceEvent::MouseMotion { delta: (dx, dy) } => {
                if dx != 0.0 {
                    self.input
                        .handle_analog_axis_change(AnalogInput::MouseMotionX, dx as f32);
                }
                if dy != 0.0 {
                    self.input
                        .handle_analog_axis_change(AnalogInput::MouseMotionY, dy as f32);
                }
            }
            _ => {}
        }
    }

    pub fn handle_input(&mut self, event: &WindowEvent) {
        if self.egui.handle_input(self.display.window(), event) {
            // input consumed
            return;
        }
        match *event {
            WindowEvent::CursorMoved { position, .. } => {
                self.input.mouse_position = glam::vec2(position.x as f32, position.y as f32);
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
            // TODO: we need to handle focus lost/regained for capturing cursor
            _ => {}
        }
    }

    pub fn set_cursor_captured(&self, captured: bool) {
        if captured {
            self.display
                .window()
                .set_cursor_grab(winit::window::CursorGrabMode::Confined)
                .or_else(|_| {
                    self.display
                        .window()
                        .set_cursor_grab(winit::window::CursorGrabMode::Locked)
                })
                .unwrap();
        } else {
            self.display
                .window()
                .set_cursor_grab(winit::window::CursorGrabMode::None)
                .unwrap();
        }
        self.display.window().set_cursor_visible(!captured);
    }
}

pub struct App<A: AppState> {
    size: Size,
    state: Option<(Context<A::Controls>, A)>,
}

impl<A: AppState> App<A> {
    pub fn new(size: Size) -> Self {
        Self { size, state: None }
    }
}

impl<A: AppState> ApplicationHandler for App<A> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_none() {
            let window = event_loop
                .create_window(Window::default_attributes().with_inner_size(self.size))
                .unwrap();

            let display = pollster::block_on(Display::from_window(window));
            let render_state = A::init_render_state(&display);
            let mut ctx = Context::new(display, render_state);
            let app_state = A::new(&mut ctx);
            self.state = Some((ctx, app_state));
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        if let Some((ctx, _)) = &mut self.state {
            ctx.handle_device_input(&event);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let Some((ctx, state)) = &mut self.state else {
            return;
        };
        if window_id != ctx.display.window().id() {
            return;
        }
        ctx.handle_input(&event);
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(new_size) => {
                self.size = Size::Physical(new_size);
                ctx.display.resize(new_size);
                ctx.display.window().request_redraw();
            }
            // WindowEvent::ScaleFactorChanged { scale_factor, .. } => {}
            WindowEvent::RedrawRequested => {
                ctx.frame_timing.update();
                if !state.update(ctx) {
                    event_loop.exit();
                    return;
                }
                // Do this after the frame is done updating, so we can clear state and update controls for the next frame.
                ctx.input.end_frame_update();

                match state.render(ctx) {
                    Ok(_) => {}
                    // Reconfigure the surface if it's lost or outdated
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        ctx.display.reconfigure();
                    }
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => {
                        log::error!("Out of memory?!");
                        event_loop.exit();
                    }
                    Err(wgpu::SurfaceError::Timeout) => log::warn!("Surface timeout"),
                }
                ctx.display.window().request_redraw();
            }
            _ => {}
        }
    }
}

pub trait AppState {
    type Controls: ControlSet;

    fn new(ctx: &mut Context<Self::Controls>) -> Self;
    fn update(&mut self, ctx: &mut Context<Self::Controls>) -> bool;
    fn render(&mut self, ctx: &mut Context<Self::Controls>) -> Result<(), wgpu::SurfaceError>;
    fn destroy(&mut self, _ctx: &mut Context<Self::Controls>) {}

    fn init_render_state(display: &Display) -> RenderState {
        RenderState::new(
            display,
            &display
                .device()
                .create_shader_module(include_wgsl!("../res/shaders/flat.wgsl")),
            &display
                .device()
                .create_shader_module(include_wgsl!("../res/shaders/text.wgsl")),
        )
    }
}
