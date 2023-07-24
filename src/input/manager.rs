use std::collections::HashMap;

use glam::Vec2;
pub use miniquad::KeyCode;
use miniquad::MouseButton;

use super::{
    AxisBinding, AxisBuilder, Binding, BindingRef, ButtonBinding, ButtonBuilder, RawAxisBinding,
};

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
pub(crate) struct DigitalInput {
    pub raw: KeyCodeOrMouseButton,
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
pub(crate) enum InputChange {
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
    fn input(&self) -> AnyInput {
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

#[derive(Default, Debug)]
pub struct Input {
    bindings: Vec<Binding>,
    by_name: HashMap<String, BindingRef>,
    bound_inputs: HashMap<AnyInput, BindingRef>,
    buffered_inputs: Vec<InputChange>,
    mouse: Cursor,
}

impl Input {
    pub fn axis(&'_ self, b: BindingRef) -> &'_ AxisBinding {
        &self.bindings[b.0].axis()
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

    fn buffer_input_event(&mut self, change: InputChange) {
        // if we don't have any buttons registered for this key, just ignore
        if !self.bound_inputs.contains_key(&change.input()) {
            return;
        }
        self.buffered_inputs.push(change);
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

    pub fn get(&'_ self, b: BindingRef) -> &'_ Binding {
        &self.bindings[b.0]
    }

    pub fn handle_analog_axis_change(&mut self, input: AnalogInput, value: f32) {
        self.buffer_input_event(InputChange::Analog { input, value });
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

    pub fn handle_mouse_motion(&mut self, x: f32, y: f32) {
        self.mouse.update_position(Vec2::new(x, y));
        if let Some(d) = self.mouse.last_change {
            println!("testing {}", d);
            if d.x != 0. {
                self.buffer_input_event(InputChange::Analog {
                    input: AnalogInput::MouseMotionX,
                    value: d.x,
                });
            }
            if d.y != 0. {
                self.buffer_input_event(InputChange::Analog {
                    input: AnalogInput::MouseMotionY,
                    value: d.y,
                });
            }
        }
    }

    pub fn new_axis(&'_ mut self, name: impl Into<String>) -> AxisBuilder<'_> {
        AxisBuilder::for_input(self, name.into())
    }

    pub fn new_button(&'_ mut self, name: impl Into<String>) -> ButtonBuilder<'_> {
        ButtonBuilder::for_input(self, name.into())
    }

    pub(super) fn register_axis(&mut self, a: AxisBinding) -> BindingRef {
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

    pub(super) fn register_button(&mut self, b: ButtonBinding) -> BindingRef {
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

    pub fn register_new_button(&mut self, name: impl Into<String>, keys: &[KeyCode]) -> BindingRef {
        let mut builder = self.new_button(name);
        for k in keys {
            builder = builder.with_key(*k);
        }
        builder.register()
    }

    pub fn update(&mut self) {
        for b in &mut self.bindings {
            b.clear();
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
}
