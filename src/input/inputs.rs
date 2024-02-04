use derive_more::{Display, From};
use winit::keyboard::KeyCode;

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
    Numpad0,
    Numpad1,
    Numpad2,
    Numpad3,
    Numpad4,
    Numpad5,
    Numpad6,
    Numpad7,
    Numpad8,
    Numpad9,
    NumpadDecimal,
    NumpadDivide,
    NumpadMultiply,
    NumpadSubtract,
    NumpadAdd,
    NumpadEnter,
    NumpadEqual,
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

impl From<KeyCode> for Key {
    fn from(k: KeyCode) -> Self {
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
}

#[derive(Debug, Display, Copy, Clone, PartialEq, Hash, Eq)]
pub enum MouseButton {
    Right,
    Left,
    Middle,
    Back,
    Forward,
    Other(u16),
}

impl From<winit::event::MouseButton> for MouseButton {
    fn from(b: winit::event::MouseButton) -> Self {
        use winit::event::MouseButton as WinitMouseButton;
        match b {
            WinitMouseButton::Right => MouseButton::Right,
            WinitMouseButton::Left => MouseButton::Left,
            WinitMouseButton::Middle => MouseButton::Middle,
            WinitMouseButton::Back => MouseButton::Back,
            WinitMouseButton::Forward => MouseButton::Forward,
            WinitMouseButton::Other(id) => MouseButton::Other(id),
        }
    }
}
