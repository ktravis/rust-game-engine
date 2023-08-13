use std::ops::Deref;

use log::error;

use super::{AnyInput, DigitalInput, InputChange, StateChange};

#[derive(Debug, Default, Copy, Clone)]
pub struct ButtonState {
    pub just_pressed: bool,
    pub is_down: bool,
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

    pub fn clear(&mut self) {
        self.just_pressed = false;
    }
}

#[derive(Debug, Clone)]
pub struct Button {
    state: ButtonState,
    triggered_input: Option<DigitalInput>,
    inputs: Vec<DigitalInput>,
    dirty: bool,
    // TODO: timing window for press?
}

impl Deref for Button {
    type Target = ButtonState;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl Button {
    pub fn new(inputs: Vec<DigitalInput>) -> Self {
        Self {
            state: Default::default(),
            triggered_input: None,
            inputs,
            dirty: true,
        }
    }

    /// Returns whether this [`Button`] is down.
    pub fn is_down(&self) -> bool {
        self.state.is_down
    }

    pub fn just_pressed(&self) -> bool {
        self.state.just_pressed
    }

    pub fn changed(&self) -> bool {
        self.dirty
    }

    pub fn clear_changed(&mut self) {
        self.dirty = false;
    }

    pub fn update(&mut self, input_change: Option<InputChange>) {
        let Some(input_change) = input_change else {
            // request to clear
            self.state.clear();
            return;
        };
        let InputChange::Digital { input, state_change } = input_change else {
            error!("Button discarding non-digital input: {:?}", input_change);
            return;
        };
        match self.triggered_input {
            Some(key) if key == input => {
                self.state.update(state_change);
                if state_change == StateChange::Released {
                    self.triggered_input.take();
                }
            }
            // the triggered key does not match, no change
            Some(_) => {}
            None => {
                if state_change == StateChange::Pressed {
                    self.state.update(state_change);
                    self.triggered_input = Some(input);
                }
            }
        }
    }

    pub fn bound_inputs(&self) -> Vec<AnyInput> {
        self.inputs
            .iter()
            .cloned()
            .map(DigitalInput::from)
            .map(AnyInput::from)
            .collect()
    }

    pub fn replace_inputs(&mut self, inputs: Vec<DigitalInput>) {
        self.inputs = inputs;
        self.dirty = true;
    }

    pub fn add_input(&mut self, input: DigitalInput) {
        for i in &self.inputs {
            if *i == input {
                debug_assert!(false, "input {:?} is already registered", input);
                return;
            }
        }
        self.inputs.push(input);
        self.dirty = true;
    }
}
