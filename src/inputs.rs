use std::collections::HashMap;

use glam::Vec2;
pub use miniquad::KeyCode;
use miniquad::MouseButton;

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
struct DigitalInput {
    raw: KeyCodeOrMouseButton,
}

impl ButtonState {
    fn update(&mut self, change: StateChange) {
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

#[derive(Debug, Clone)]
pub enum Binding {
    Button(ButtonBinding),
    Axis(AxisBinding),
}

impl Binding {
    fn update(&mut self, input_change: InputChange) {
        match self {
            Binding::Button(b) => {
                let InputChange::Digital { input, state_change } = input_change else {
                    return;
                };
                b.update(input.raw, state_change);
            }
            Binding::Axis(a) => a.update(input_change),
        }
    }

    pub fn button(&self) -> &ButtonBinding {
        match self {
            Binding::Button(b) => b,
            _ => panic!("binding is not a button"),
        }
    }

    pub fn axis(&self) -> &AxisBinding {
        match self {
            Binding::Axis(a) => a,
            _ => panic!("binding is not an axis"),
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
enum AnalogInput {
    MouseMotionX,
    MouseMotionY,
    MouseWheelX,
    MouseWheelY,
}

#[derive(Debug, Copy, Clone, PartialEq, Hash, Eq)]
enum AnyInput {
    Digital(DigitalInput),
    Analog(AnalogInput),
}

impl From<DigitalInput> for AnyInput {
    fn from(i: DigitalInput) -> Self {
        Self::Digital(i)
    }
}

impl From<AnalogInput> for AnyInput {
    fn from(i: AnalogInput) -> Self {
        Self::Analog(i)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum InputChange {
    Digital {
        input: DigitalInput,
        state_change: StateChange,
    },
    Analog {
        input: AnalogInput,
        delta: f32,
    },
}

impl InputChange {
    fn input(&self) -> AnyInput {
        match *self {
            InputChange::Digital { input, .. } => input.into(),
            InputChange::Analog { input, .. } => input.into(),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct ButtonBinding {
    pub name: String,
    pub state: ButtonState,
    keys: Vec<KeyCodeOrMouseButton>,
    triggered_key: Option<KeyCodeOrMouseButton>,
    // TODO: timing window for press?
}

impl ButtonBinding {
    /// Returns whether this [`ButtonBinding`] is down.
    pub fn is_down(&self) -> bool {
        self.state.is_down
    }

    pub fn just_pressed(&self) -> bool {
        self.state.just_pressed
    }

    fn update(&mut self, key: KeyCodeOrMouseButton, change: StateChange) {
        match self.triggered_key {
            Some(k) if k == key => {
                self.state.update(change);
                if change == StateChange::Released {
                    self.triggered_key.take();
                }
            }
            // the triggered key does not match, no change
            Some(_) => {}
            None => {
                if change == StateChange::Pressed {
                    self.state.update(change);
                    self.triggered_key = Some(key);
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum OverlapMode {
    /// Keep the first key's direction
    First,
    /// Use whichever key is latest for direction
    Latest,
    /// Cancel out the value if both directions are held
    Neutral,
}

impl Default for OverlapMode {
    fn default() -> Self {
        Self::Neutral
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum AxisDirection {
    Left,
    Right,
}

impl AxisDirection {
    fn value(self) -> f32 {
        match self {
            AxisDirection::Left => -1.,
            AxisDirection::Right => 1.,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum DigitalAxisState {
    None,
    One(AxisDirection),
    Both {
        first: AxisDirection,
        second: AxisDirection,
    },
}

impl Default for DigitalAxisState {
    fn default() -> Self {
        Self::None
    }
}

impl DigitalAxisState {
    fn value(self, overlap_mode: OverlapMode) -> f32 {
        use DigitalAxisState::*;
        match self {
            None => 0.,
            One(dir) => dir.value(),
            Both { first, second } => match overlap_mode {
                OverlapMode::First => first.value(),
                OverlapMode::Latest => second.value(),
                OverlapMode::Neutral => 0.,
            },
        }
    }
    fn update(self, dir: AxisDirection, state_change: StateChange) -> Self {
        use DigitalAxisState::*;
        use StateChange::*;
        match (self, state_change) {
            (None, Pressed) => One(dir),
            (One(held), Pressed) if held != dir => Both {
                first: held,
                second: dir,
            },
            (One(held), Released) if held == dir => None,
            (Both { first, second }, Released) => {
                if first == dir {
                    One(second)
                } else if second == dir {
                    One(first)
                } else {
                    self
                }
            }
            _ => self,
        }
    }
}

#[derive(Copy, Clone, Debug)]
enum RawAxisBinding {
    Digital {
        pair: (KeyCodeOrMouseButton, KeyCodeOrMouseButton),
        state: DigitalAxisState,
    },
    Analog {
        input: AnalogInput,
        value: f32,
    },
}

impl From<(KeyCodeOrMouseButton, KeyCodeOrMouseButton)> for RawAxisBinding {
    fn from(pair: (KeyCodeOrMouseButton, KeyCodeOrMouseButton)) -> Self {
        RawAxisBinding::Digital {
            pair,
            state: DigitalAxisState::None,
        }
    }
}

impl RawAxisBinding {
    fn value(&self, overlap_mode: OverlapMode) -> f32 {
        match self {
            RawAxisBinding::Digital { state, .. } => state.value(overlap_mode),
            RawAxisBinding::Analog { value, .. } => *value,
        }
    }

    fn update(&mut self, input_change: InputChange) -> bool {
        use AxisDirection::*;
        use RawAxisBinding::*;
        match self {
            Digital {
                pair: (l, r),
                state,
            } => {
                let InputChange::Digital { input: DigitalInput { raw: key }, state_change } = input_change else {
                    // this is an analog change
                    return *state != DigitalAxisState::None;
                };
                let dir = if key == *l {
                    Left
                } else if key == *r {
                    Right
                } else {
                    return *state != DigitalAxisState::None;
                };
                *state = state.update(dir, state_change);
                return *state != DigitalAxisState::None;
            }
            Analog { value, .. } => {
                if let InputChange::Analog { delta, .. } = input_change {
                    *value += delta;
                };
                // else this is an analog change
                return *value != 0.0;
            }
        };
    }
}

#[derive(Debug, Default, Clone)]
pub struct AxisBinding {
    pub name: String,
    overlap_mode: OverlapMode,
    raw: Vec<RawAxisBinding>,
    active_indices: Vec<usize>,
}

impl AxisBinding {
    /// Value from -1.0 to 1.0
    pub fn value(&self) -> f32 {
        // TODO: overlap_mode isn't respected if two separate raw bindings
        // have input - the first to be registered will take precedence. Could
        // get around this by collecting all values and reducing according to
        // overlap_mode, but this might not make sense for analog axes.
        self.active_indices
            .first()
            .map(|i| self.raw[*i].value(self.overlap_mode))
            .unwrap_or_default()
    }

    fn update(&mut self, input_change: InputChange) {
        self.active_indices = self
            .raw
            .iter_mut()
            .enumerate()
            .filter_map(|(i, raw)| raw.update(input_change).then_some(i))
            .collect();
    }
}

#[derive(Debug, Copy, Clone)]
pub struct BindingRef(usize);

pub struct ButtonBuilder<'a> {
    input: &'a mut Input,
    button: ButtonBinding,
}

impl<'a> ButtonBuilder<'a> {
    pub fn with_key(mut self, key: impl Into<KeyCodeOrMouseButton>) -> Self {
        self.button.keys.push(key.into());
        self
    }

    pub fn register(self) -> BindingRef {
        debug_assert!(!self.button.keys.is_empty());
        self.input.register_button(self.button)
    }
}

pub struct AxisBuilder<'a> {
    input: &'a mut Input,
    axis: AxisBinding,
}

impl<'a> AxisBuilder<'a> {
    pub fn with_keys(
        mut self,
        left: impl Into<KeyCodeOrMouseButton>,
        right: impl Into<KeyCodeOrMouseButton>,
    ) -> Self {
        self.axis.raw.push((left.into(), right.into()).into());
        self
    }

    pub fn overlap_mode(mut self, overlap_mode: OverlapMode) -> Self {
        self.axis.overlap_mode = overlap_mode;
        self
    }

    pub fn register(self) -> BindingRef {
        debug_assert!(!self.axis.raw.is_empty());
        self.input.register_axis(self.axis)
    }
}

// #[derive(Debug, Copy, Clone)]
// enum InputEvent {
//     Key {
//         code: KeyCode,
//         state_change: StateChange,
//     },
//     MouseButton {
//         button: MouseButton,
//         state_change: StateChange,
//     },
//     MouseMotion(Vec2),
//     MouseWheel(Vec2),
// }

#[derive(Default, Debug)]
pub struct Input {
    bindings: Vec<Binding>,
    by_name: HashMap<String, BindingRef>,
    bound_inputs: HashMap<AnyInput, BindingRef>,
    buffered_inputs: Vec<InputChange>,
}

impl Input {
    pub fn update(&mut self) {
        for b in &mut self.bindings {
            if let Binding::Button(b) = b {
                b.state.just_pressed = false;
            }
        }
        for input_change in self.buffered_inputs.drain(..) {
            match input_change {
                InputChange::Digital { input, .. } => {
                    let b = self
                        .bound_inputs
                        .get(&AnyInput::Digital(input))
                        .unwrap_or_else(|| panic!("key {:?} not registered", input.raw));
                    self.bindings[b.0].update(input_change);
                }
                InputChange::Analog { input, .. } => {
                    let b = self
                        .bound_inputs
                        .get(&AnyInput::Analog(input))
                        .unwrap_or_else(|| panic!("analog input {:?} not registered", input));
                    self.bindings[b.0].update(input_change);
                }
            }
        }
    }

    pub fn new_button(&'_ mut self, name: impl Into<String>) -> ButtonBuilder<'_> {
        ButtonBuilder {
            input: self,
            button: ButtonBinding {
                name: name.into(),
                ..Default::default()
            },
        }
    }

    pub fn register_new_button(&mut self, name: impl Into<String>, keys: &[KeyCode]) -> BindingRef {
        let mut builder = self.new_button(name);
        for k in keys {
            builder = builder.with_key(*k);
        }
        builder.register()
    }

    pub fn new_axis(&'_ mut self, name: impl Into<String>) -> AxisBuilder<'_> {
        AxisBuilder {
            input: self,
            axis: AxisBinding {
                name: name.into(),
                ..Default::default()
            },
        }
    }

    pub fn get(&'_ self, b: BindingRef) -> &'_ Binding {
        &self.bindings[b.0]
    }

    pub fn axis(&'_ self, b: BindingRef) -> &'_ AxisBinding {
        &self.bindings[b.0].axis()
    }

    pub fn button(&'_ self, b: BindingRef) -> &'_ ButtonBinding {
        &self.bindings[b.0].button()
    }

    pub fn button_by_name(&'_ self, name: impl AsRef<str>) -> &'_ ButtonBinding {
        let name = name.as_ref();
        let b = self
            .by_name
            .get(name)
            .unwrap_or_else(|| panic!("no key {} found", name));
        match &self.bindings[b.0] {
            Binding::Button(b) => b,
            _ => panic!("binding {} is not a button", name),
        }
    }

    pub fn axis_by_name(&'_ self, name: impl AsRef<str>) -> &'_ AxisBinding {
        let name = name.as_ref();
        let b = self
            .by_name
            .get(name)
            .unwrap_or_else(|| panic!("no key {} found", name));
        match &self.bindings[b.0] {
            Binding::Axis(a) => a,
            _ => panic!("binding {} is not an axis", name),
        }
    }

    pub fn handle_key_or_button_change(
        &mut self,
        key: impl Into<KeyCodeOrMouseButton>,
        state_change: StateChange,
    ) {
        self.buffer_input_event(InputChange::Digital {
            input: DigitalInput { raw: key.into() },
            state_change,
        });
    }

    fn buffer_input_event(&mut self, change: InputChange) {
        // if we don't have any buttons registered for this key, just ignore
        if !self.bound_inputs.contains_key(&change.input()) {
            return;
        }
        self.buffered_inputs.push(change);
    }

    pub fn handle_mouse_motion(&mut self, x: f32, y: f32) {
        println!("testing {} {}", x, y);
    }

    fn register_button(&mut self, b: ButtonBinding) -> BindingRef {
        let index = self.bindings.len();
        let button_ref = BindingRef(index);
        for key in &b.keys {
            self.bound_inputs
                .insert(AnyInput::Digital(DigitalInput { raw: *key }), button_ref);
        }
        self.by_name.insert(b.name.clone(), button_ref);
        self.bindings.push(Binding::Button(b));
        button_ref
    }

    fn register_axis(&mut self, a: AxisBinding) -> BindingRef {
        let index = self.bindings.len();
        let axis_binding = BindingRef(index);
        for raw in &a.raw {
            match raw {
                RawAxisBinding::Digital { pair: (l, r), .. } => {
                    self.bound_inputs
                        .insert(AnyInput::Digital(DigitalInput { raw: *l }), axis_binding);
                    self.bound_inputs
                        .insert(AnyInput::Digital(DigitalInput { raw: *r }), axis_binding);
                }
                RawAxisBinding::Analog { input, .. } => {
                    self.bound_inputs
                        .insert(AnyInput::Analog(*input), axis_binding);
                }
            }
        }
        self.by_name.insert(a.name.clone(), axis_binding);
        self.bindings.push(Binding::Axis(a));
        axis_binding
    }
}
