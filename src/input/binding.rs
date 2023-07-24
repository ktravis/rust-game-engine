use crate::input::DigitalInput;

use super::{AnalogInput, ButtonState, InputChange, KeyCodeOrMouseButton, StateChange};

#[derive(Debug, Clone)]
pub enum Binding {
    Button(ButtonBinding),
    Axis(AxisBinding),
}

impl Binding {
    pub(super) fn update(&mut self, input_change: InputChange) {
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

    pub(super) fn clear(&mut self) {
        match self {
            Binding::Button(b) => b.state.just_pressed = false,
            Binding::Axis(a) => a.raw.iter_mut().for_each(|raw| match raw {
                RawAxisBinding::Digital { .. } => {}
                RawAxisBinding::Analog { value, .. } => *value = 0.,
            }),
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

#[derive(Debug, Default, Clone)]
pub struct ButtonBinding {
    pub name: String,
    pub state: ButtonState,
    pub(super) keys: Vec<KeyCodeOrMouseButton>,
    pub(super) triggered_key: Option<KeyCodeOrMouseButton>,
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
pub(super) enum AxisDirection {
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
pub(super) enum DigitalAxisState {
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
pub(super) enum RawAxisBinding {
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

#[derive(Debug, Default, Clone)]
pub struct AxisBinding {
    pub name: String,
    pub(super) overlap_mode: OverlapMode,
    pub(super) raw: Vec<RawAxisBinding>,
    pub(super) active_indices: Vec<usize>,
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
