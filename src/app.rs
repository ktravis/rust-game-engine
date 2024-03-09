use std::borrow::Cow;

use winit::{
    dpi::PhysicalPosition,
    error::EventLoopError,
    event::{Event, KeyEvent, WindowEvent},
    event_loop::{EventLoop, EventLoopWindowTarget},
    keyboard::PhysicalKey,
};

use crate::{
    input::{AnalogInput, ControlSet, InputManager, Key, MouseButton},
    renderer::{Display, RenderState},
    time::FrameTiming,
};

pub struct Context<C: ControlSet> {
    pub display: Display,
    pub render_state: RenderState,
    pub frame_timing: FrameTiming,
    pub input: InputManager<C>,
}

impl<C: ControlSet> Context<C> {
    pub fn new(display: Display, render_state: RenderState) -> Self {
        Self {
            display,
            render_state,
            frame_timing: Default::default(),
            input: Default::default(),
        }
    }

    pub fn handle_input(&mut self, event: &WindowEvent) {
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
}

pub struct App<A: AppState> {
    ctx: Context<A::Controls>,
    app_state: A,
}

impl<A: AppState> App<A> {
    pub fn run<T: 'static>(&mut self, event_loop: EventLoop<T>) -> Result<(), EventLoopError> {
        event_loop.run(|event, elwt| self.window_event_handler(event, elwt))
    }

    fn window_event_handler<T>(
        &mut self,
        event: winit::event::Event<T>,
        elwt: &EventLoopWindowTarget<T>,
    ) {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == self.ctx.display.window().id() => {
                self.ctx.handle_input(event);
                match event {
                    WindowEvent::CloseRequested => elwt.exit(),
                    WindowEvent::Resized(physical_size) => {
                        self.ctx.display.resize(*physical_size);
                        self.ctx.display.window().request_redraw();
                    }
                    // WindowEvent::ScaleFactorChanged { scale_factor, .. } => {}
                    WindowEvent::RedrawRequested => {
                        self.ctx.frame_timing.update();
                        if !self.app_state.update(&mut self.ctx) {
                            elwt.exit();
                            return;
                        }
                        // Do this after the frame is done updating, so we can clear state and update controls for the next frame.
                        self.ctx.input.end_frame_update();

                        match self.app_state.render(&mut self.ctx) {
                            Ok(_) => {}
                            // Reconfigure the surface if it's lost or outdated
                            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                                self.ctx.display.reconfigure();
                            }
                            // The system is out of memory, we should probably quit
                            Err(wgpu::SurfaceError::OutOfMemory) => {
                                log::error!("Out of memory?!");
                                elwt.exit();
                            }
                            Err(wgpu::SurfaceError::Timeout) => log::warn!("Surface timeout"),
                        }
                        self.ctx.display.window().request_redraw();
                    }
                    _ => {}
                }
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
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("instanced"),
                    source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
                        "../res/shaders/model.wgsl"
                    ))),
                }),
        )
    }

    fn create_app(display: Display) -> App<Self>
    where
        Self: Sized,
    {
        let render_state = Self::init_render_state(&display);
        let mut ctx = Context::new(display, render_state);
        let app_state = Self::new(&mut ctx);
        App { ctx, app_state }
    }
}
