use std::collections::HashMap;
use std::ops::Deref;
use std::ops::DerefMut;

use glam::Vec2;

use super::{
    AnalogInput, AnyInput, Cursor, DigitalInput, InputChange, KeyOrMouseButton, StateChange,
};

#[derive(Debug)]
pub struct InputManager<Controls: ControlSet> {
    pub mouse: Cursor,
    pub controls: Controls,
    inputs_by_control: HashMap<Controls::Control, Vec<AnyInput>>,
    controls_by_input: HashMap<AnyInput, Vec<Controls::Control>>,
}

impl<C: ControlSet + Default> Default for InputManager<C> {
    fn default() -> Self {
        Self {
            mouse: Cursor::default(),
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

pub trait ControlSet {
    type Control: Sized + std::fmt::Debug + Copy + Eq + std::hash::Hash;
    fn controls<'a>() -> &'a [Self::Control];
    fn handle_input(&mut self, control: &Self::Control, change: Option<InputChange>);
    fn bound_inputs(&self, control: &Self::Control) -> Vec<AnyInput>;
    fn control_changed(&self, control: &Self::Control) -> bool;
    fn clear_control_changed(&mut self, control: &Self::Control);
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

    pub fn handle_mouse_motion(&mut self, x: f32, y: f32) {
        let Some(d) = self.mouse.update_position(Vec2::new(x, y)) else {
            return;
        };
        if d.x != 0. {
            self.handle_input_change(InputChange::Analog {
                input: AnalogInput::MouseMotionX,
                value: d.x,
            });
        }
        if d.y != 0. {
            self.handle_input_change(InputChange::Analog {
                input: AnalogInput::MouseMotionY,
                value: d.y,
            });
        }
    }

    fn handle_input_change(&mut self, input_change: InputChange) {
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

    pub fn end_frame_update(&mut self) {
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
