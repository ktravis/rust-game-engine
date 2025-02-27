use std::collections::HashMap;
use std::ops::Deref;
use std::ops::DerefMut;

use glam::Vec2;

use super::InputState;
use super::{AnalogInput, AnyInput, DigitalInput, InputChange, KeyOrMouseButton, StateChange};

#[derive(Debug)]
pub struct InputManager<Controls: ControlSet> {
    pub mouse_position: Vec2,
    pub mouse_delta: Option<Vec2>,
    pub controls: Controls,
    inputs_by_control: HashMap<Controls::Control, Vec<AnyInput>>,
    controls_by_input: HashMap<AnyInput, Vec<Controls::Control>>,
}

impl<C: ControlSet> Default for InputManager<C> {
    fn default() -> Self {
        Self {
            mouse_position: Default::default(),
            mouse_delta: Default::default(),
            controls: C::default(),
            inputs_by_control: HashMap::default(),
            controls_by_input: HashMap::default(),
        }
    }
}

impl<C: ControlSet> Deref for InputManager<C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        &self.controls
    }
}

impl<C: ControlSet> DerefMut for InputManager<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.controls
    }
}

pub trait ControlSet: Default {
    type Control: Sized + std::fmt::Debug + Copy + Eq + std::hash::Hash + 'static;
    fn controls() -> &'static [Self::Control];
    fn handle_input(&mut self, control: &Self::Control, change: Option<InputChange>);
    fn bound_inputs(&self, control: &Self::Control) -> Vec<AnyInput>;
    fn control_changed(&self, control: &Self::Control) -> bool;
    fn clear_control_changed(&mut self, control: &Self::Control);
    fn control_state(&self, control: &Self::Control) -> InputState;
}

impl<Controls> InputManager<Controls>
where
    Controls: ControlSet,
{
    pub fn handle_analog_axis_change(&mut self, input: AnalogInput, value: f32) {
        self.handle_input_change(InputChange::Analog { input, value });
    }

    pub fn handle_key_or_button_change(
        &mut self,
        key: impl Into<KeyOrMouseButton>,
        state_change: StateChange,
    ) {
        self.handle_input_change(InputChange::Digital {
            input: DigitalInput { raw: key.into() },
            state_change,
        });
    }

    fn handle_input_change(&mut self, input_change: InputChange) {
        match input_change {
            InputChange::Analog {
                input: AnalogInput::MouseMotionX,
                value,
            } => self.mouse_delta.get_or_insert(Vec2::ZERO).x += value,
            InputChange::Analog {
                input: AnalogInput::MouseMotionY,
                value,
            } => self.mouse_delta.get_or_insert(Vec2::ZERO).y += value,
            _ => {}
        }
        let Some(controls_to_update) = self.controls_by_input.get(&input_change.input()) else {
            return;
        };
        for control in controls_to_update {
            self.controls.handle_input(control, Some(input_change));
        }
    }

    fn update_mappings(&mut self) {
        self.inputs_by_control.clear();
        self.controls_by_input.clear();

        for control in Controls::controls() {
            self.inputs_by_control
                .insert(*control, self.bound_inputs(&control));
            self.bound_inputs(control).iter().for_each(|i| {
                self.controls_by_input.entry(*i).or_default().push(*control);
            });
            self.clear_control_changed(control);
        }
    }

    pub fn status(
        &self,
    ) -> impl Iterator<Item = (Controls::Control, Vec<AnyInput>, InputState)> + '_ {
        Controls::controls()
            .iter()
            .map(|c| (*c, self.bound_inputs(c), self.control_state(c)))
    }

    pub fn end_frame_update(&mut self) {
        self.mouse_delta.take();
        for control in Controls::controls() {
            if self.control_changed(control) {
                self.update_mappings();
                break;
            }
        }
        for control in Controls::controls() {
            self.handle_input(control, None);
        }
    }
}
