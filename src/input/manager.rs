use std::ops::Deref;
use std::ops::DerefMut;

use glam::Vec2;
pub use miniquad::KeyCode;
pub use miniquad::MouseButton;

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub enum StateChange {
    Pressed,
    Released,
}

#[derive(Debug, Default, Copy, Clone)]
pub struct ButtonState {
    pub just_pressed: bool,
    pub is_down: bool,
}

#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
pub struct DigitalInput {
    pub raw: KeyCodeOrMouseButton,
}

impl From<KeyCodeOrMouseButton> for DigitalInput {
    fn from(raw: KeyCodeOrMouseButton) -> Self {
        Self { raw }
    }
}

impl From<&KeyCodeOrMouseButton> for DigitalInput {
    fn from(raw: &KeyCodeOrMouseButton) -> Self {
        Self { raw: *raw }
    }
}

impl ButtonState {
    pub fn update(&mut self, change: StateChange) {
        match change {
            StateChange::Pressed => {
                self.just_pressed = !self.is_down;
                self.is_down = true;
            }
            StateChange::Released => {
                self.just_pressed = false;
                self.is_down = false;
            }
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Hash, Eq)]
pub enum KeyCodeOrMouseButton {
    KeyCode(KeyCode),
    MouseButton(MouseButton),
}

impl From<KeyCode> for KeyCodeOrMouseButton {
    fn from(kc: KeyCode) -> Self {
        KeyCodeOrMouseButton::KeyCode(kc)
    }
}

impl From<MouseButton> for KeyCodeOrMouseButton {
    fn from(kc: MouseButton) -> Self {
        KeyCodeOrMouseButton::MouseButton(kc)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Hash, Eq)]
pub enum AnalogInput {
    MouseMotionX,
    MouseMotionY,
    MouseWheelX,
    MouseWheelY,
}

#[derive(Debug, Copy, Clone, PartialEq, Hash, Eq)]
pub enum AnyInput {
    Digital(DigitalInput),
    Analog(AnalogInput),
}

impl From<DigitalInput> for AnyInput {
    fn from(i: DigitalInput) -> Self {
        Self::Digital(i)
    }
}

impl From<&DigitalInput> for AnyInput {
    fn from(i: &DigitalInput) -> Self {
        Self::Digital(*i)
    }
}

impl From<&AnalogInput> for AnyInput {
    fn from(i: &AnalogInput) -> Self {
        Self::Analog(*i)
    }
}

impl From<KeyCodeOrMouseButton> for AnyInput {
    fn from(raw: KeyCodeOrMouseButton) -> Self {
        Self::Digital(raw.into())
    }
}

impl<T> From<T> for AnyInput
where
    T: Into<AnalogInput>,
{
    fn from(a: T) -> Self {
        Self::Analog(a.into())
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum InputChange {
    Digital {
        input: DigitalInput,
        state_change: StateChange,
    },
    Analog {
        input: AnalogInput,
        value: f32,
    },
}

impl InputChange {
    pub fn input(&self) -> AnyInput {
        match *self {
            InputChange::Digital { input, .. } => input.into(),
            InputChange::Analog { input, .. } => input.into(),
        }
    }
}

#[derive(Default, Debug)]
pub struct Cursor {
    position: Vec2,
    last_change: Option<Vec2>,
}

impl Cursor {
    pub fn update_position(&mut self, new_position: Vec2) {
        self.last_change = Some(match self.last_change {
            Some(_) => new_position - self.position,
            None => Vec2::ZERO,
        });
        self.position = new_position;
    }
}

#[derive(Debug, Default)]
pub struct InputManager<Controls: ControlsManager + Default> {
    pub mouse: Cursor,
    pub controls: Controls,
}

impl<C: ControlsManager + Default> Deref for InputManager<C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        &self.controls
    }
}

impl<C: ControlsManager + Default> DerefMut for InputManager<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.controls
    }
}

pub trait ControlsManager {
    type ControlSet: Sized + std::fmt::Debug + Copy + Eq + std::hash::Hash;
    fn handle_input(&mut self, change: InputChange);
    fn controls<'a>() -> &'a [Self::ControlSet];
    fn bound_inputs(&self, control: &Self::ControlSet) -> Vec<AnyInput>;

    /// Clear the last frame-specific state of all defined controls at the beginning of a frame.
    fn end_frame_update(&mut self);
}

impl<Controls> InputManager<Controls>
where
    Controls: ControlsManager + Default,
{
    pub fn handle_analog_axis_change(&mut self, input: AnalogInput, value: f32) {
        self.handle_input(InputChange::Analog { input, value });
    }

    pub fn handle_key_or_button_change(
        &mut self,
        key: impl Into<KeyCodeOrMouseButton>,
        state_change: StateChange,
    ) {
        self.handle_input(InputChange::Digital {
            input: DigitalInput { raw: key.into() },
            state_change,
        });
    }

    pub fn handle_mouse_motion(&mut self, x: f32, y: f32) {
        self.mouse.update_position(Vec2::new(x, y));
        if let Some(d) = self.mouse.last_change {
            if d.x != 0. {
                self.handle_input(InputChange::Analog {
                    input: AnalogInput::MouseMotionX,
                    value: d.x,
                });
            }
            if d.y != 0. {
                self.handle_input(InputChange::Analog {
                    input: AnalogInput::MouseMotionY,
                    value: d.y,
                });
            }
        }
    }
}
