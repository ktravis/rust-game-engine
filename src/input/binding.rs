use crate::input::DigitalInput;
use log::error;

use super::{AnalogInput, AnyInput, ButtonState, InputChange, KeyCodeOrMouseButton, StateChange};

#[derive(Debug, Clone)]
pub struct ButtonBinding<C: std::fmt::Debug> {
    pub control: C,
    pub state: ButtonState,
    pub(super) triggered_key: Option<KeyCodeOrMouseButton>,
    pub(super) inputs: Vec<KeyCodeOrMouseButton>,
    pub(crate) dirty: bool,
    // TODO: timing window for press?
}

impl<C> ButtonBinding<C>
where
    C: std::fmt::Debug,
{
    pub(crate) fn new(control: C, inputs: Vec<KeyCodeOrMouseButton>) -> Self {
        Self {
            control,
            state: Default::default(),
            triggered_key: None,
            inputs,
            dirty: true,
        }
    }

    /// Returns whether this [`ButtonBinding`] is down.
    pub fn is_down(&self) -> bool {
        self.state.is_down
    }

    pub fn just_pressed(&self) -> bool {
        self.state.just_pressed
    }

    pub fn changed(&self) -> bool {
        self.dirty
    }

    pub fn update(&mut self, input_change: Option<InputChange>) {
        let Some(input_change) = input_change else {
            // request to clear
            self.clear();
            return;
        };
        let InputChange::Digital { input: DigitalInput { raw }, state_change } = input_change else {
            error!("Button {:?} discarding non-digital input: {:?}", self.control, input_change);
            return;
        };
        match self.triggered_key {
            Some(key) if key == raw => {
                self.state.update(state_change);
                if state_change == StateChange::Released {
                    self.triggered_key.take();
                }
            }
            // the triggered key does not match, no change
            Some(_) => {}
            None => {
                if state_change == StateChange::Pressed {
                    self.state.update(state_change);
                    self.triggered_key = Some(raw);
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

    pub(super) fn clear(&mut self) {
        self.state.just_pressed = false;
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
pub(crate) enum AxisDirection {
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
pub(crate) enum DigitalAxisState {
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
pub(crate) enum RawAxisBinding {
    Digital {
        pair: (KeyCodeOrMouseButton, KeyCodeOrMouseButton),
        state: DigitalAxisState,
    },
    Analog {
        input: AnalogInput,
        value: f32,
    },
}

impl<A, B> From<(A, B)> for RawAxisBinding
where
    A: Into<KeyCodeOrMouseButton>,
    B: Into<KeyCodeOrMouseButton>,
{
    fn from((a, b): (A, B)) -> Self {
        RawAxisBinding::Digital {
            pair: (a.into(), b.into()),
            state: DigitalAxisState::None,
        }
    }
}

impl From<AnalogInput> for RawAxisBinding {
    fn from(input: AnalogInput) -> Self {
        Self::Analog { input, value: 0.0 }
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
                if let InputChange::Analog {
                    value: new_value, ..
                } = input_change
                {
                    *value = new_value;
                };
                return *value != 0.0;
            }
        };
    }
}

#[derive(Debug, Clone)]
pub struct AxisBinding<C> {
    pub control: C,
    pub(super) overlap_mode: OverlapMode,
    pub(super) raw: Vec<RawAxisBinding>,
    pub(super) active_indices: Vec<usize>,
    pub(crate) dirty: bool,
}

impl<C> AxisBinding<C> {
    pub(crate) fn new(control: C, raw: Vec<RawAxisBinding>) -> Self {
        Self {
            control,
            raw,
            overlap_mode: Default::default(),
            active_indices: Default::default(),
            dirty: true,
        }
    }

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

    pub fn update(&mut self, input_change: Option<InputChange>) {
        let Some(input_change) = input_change else {
            // request to clear
            self.clear();
            return;
        };
        self.active_indices = self
            .raw
            .iter_mut()
            .enumerate()
            .filter_map(|(i, raw)| raw.update(input_change).then_some(i))
            .collect();
    }

    pub fn bound_inputs(&self) -> Vec<AnyInput> {
        let mut inputs = Vec::new();
        for r in &self.raw {
            match r {
                RawAxisBinding::Digital {
                    pair: (low, high), ..
                } => {
                    inputs.push(AnyInput::Digital(low.into()));
                    inputs.push(AnyInput::Digital(high.into()));
                }
                RawAxisBinding::Analog { input, .. } => inputs.push(input.into()),
            };
        }
        inputs
    }

    pub fn changed(&self) -> bool {
        self.dirty
    }

    fn clear(&mut self) {
        self.raw.iter_mut().for_each(|raw| match raw {
            RawAxisBinding::Digital { .. } => {}
            RawAxisBinding::Analog { value, .. } => *value = 0.,
        });
    }
}
