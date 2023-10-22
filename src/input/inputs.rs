use derive_more::{Display, From};
use std::fmt::Display;

#[derive(Debug, Display, Copy, Clone, Eq, Hash, PartialEq)]
pub enum AxisInput {
    #[display(fmt = "<{}, {}>", _0, _1)]
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

#[derive(Debug, Display, Copy, Clone, Eq, Hash, PartialEq)]
pub struct DigitalInput {
    pub raw: KeyOrMouseButton,
}

impl<T> From<T> for DigitalInput
where
    T: Into<KeyOrMouseButton>,
{
    fn from(raw: T) -> Self {
        Self { raw: raw.into() }
    }
}

#[derive(Debug, Display, From, Copy, Clone, PartialEq, Hash, Eq)]
pub enum KeyOrMouseButton {
    Key(Key),
    MouseButton(MouseButton),
}

#[derive(Debug, Display, Copy, Clone, PartialEq, Hash, Eq)]
pub enum AnalogInput {
    MouseMotionX,
    MouseMotionY,
    MouseWheelX,
    MouseWheelY,
}

#[derive(Debug, Display, From, Copy, Clone, PartialEq, Hash, Eq)]
pub enum AnyInput {
    Digital(DigitalInput),
    Analog(AnalogInput),
}

impl From<KeyOrMouseButton> for AnyInput {
    fn from(raw: KeyOrMouseButton) -> Self {
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

#[derive(Debug, Display, Copy, Clone, PartialEq, Hash, Eq)]
pub enum Key {
    Space,
    Apostrophe,
    Comma,
    Minus,
    Period,
    Slash,
    Key0,
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    Key8,
    Key9,
    Semicolon,
    Equal,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    LeftBracket,
    Backslash,
    RightBracket,
    GraveAccent,
    World1,
    World2,
    Escape,
    Enter,
    Tab,
    Backspace,
    Insert,
    Delete,
    Right,
    Left,
    Down,
    Up,
    PageUp,
    PageDown,
    Home,
    End,
    CapsLock,
    ScrollLock,
    NumLock,
    PrintScreen,
    Pause,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,
    F25,
    Kp0,
    Kp1,
    Kp2,
    Kp3,
    Kp4,
    Kp5,
    Kp6,
    Kp7,
    Kp8,
    Kp9,
    KpDecimal,
    KpDivide,
    KpMultiply,
    KpSubtract,
    KpAdd,
    KpEnter,
    KpEqual,
    LeftShift,
    LeftControl,
    LeftAlt,
    LeftSuper,
    RightShift,
    RightControl,
    RightAlt,
    RightSuper,
    Menu,
    Unknown,
}

#[derive(Debug, Display, Copy, Clone, PartialEq, Hash, Eq)]
pub enum MouseButton {
    Right,
    Left,
    Middle,
    Unknown,
}
