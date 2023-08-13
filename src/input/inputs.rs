pub use miniquad::{KeyCode, MouseButton};

#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
pub enum AxisInput {
    Digital(DigitalInput, DigitalInput),
    Analog(AnalogInput),
}

impl<A, B> From<(A, B)> for AxisInput
where
    A: Into<DigitalInput>,
    B: Into<DigitalInput>,
{
    fn from((a, b): (A, B)) -> Self {
        Self::Digital(a.into(), b.into())
    }
}

impl From<AnalogInput> for AxisInput {
    fn from(input: AnalogInput) -> Self {
        Self::Analog(input)
    }
}

impl From<&AnalogInput> for AxisInput {
    fn from(input: &AnalogInput) -> Self {
        Self::Analog(*input)
    }
}

#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
pub struct DigitalInput {
    pub raw: KeyCodeOrMouseButton,
}

impl<T> From<T> for DigitalInput
where
    T: Into<KeyCodeOrMouseButton>,
{
    fn from(raw: T) -> Self {
        Self { raw: raw.into() }
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

impl From<AnalogInput> for AnyInput {
    fn from(a: AnalogInput) -> Self {
        Self::Analog(a)
    }
}

impl From<DigitalInput> for AnyInput {
    fn from(i: DigitalInput) -> Self {
        Self::Digital(i)
    }
}

impl From<KeyCodeOrMouseButton> for AnyInput {
    fn from(raw: KeyCodeOrMouseButton) -> Self {
        Self::Digital(raw.into())
    }
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub enum StateChange {
    Pressed,
    Released,
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
