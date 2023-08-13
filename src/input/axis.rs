use crate::input::AxisInput;

use super::{AnalogInput, AnyInput, DigitalInput, InputChange, StateChange};

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
    Low,
    High,
}

impl AxisDirection {
    fn value(self) -> f32 {
        match self {
            AxisDirection::Low => -1.,
            AxisDirection::High => 1.,
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
    // TODO: change this into input: AxisInput, state: AxisState
    Digital {
        pair: (DigitalInput, DigitalInput),
        state: DigitalAxisState,
    },
    Analog {
        input: AnalogInput,
        value: f32,
    },
}

impl From<AxisInput> for RawAxisBinding {
    fn from(value: AxisInput) -> Self {
        match value {
            AxisInput::Digital(a, b) => Self::Digital {
                pair: (a, b),
                state: Default::default(),
            },
            AxisInput::Analog(input) => Self::Analog { input, value: 0.0 },
        }
    }
}

impl RawAxisBinding {
    fn input(&self) -> AxisInput {
        match self {
            RawAxisBinding::Digital { pair, .. } => (*pair).into(),
            RawAxisBinding::Analog { input, .. } => input.into(),
        }
    }

    fn value(&self, overlap_mode: OverlapMode) -> f32 {
        match self {
            RawAxisBinding::Digital { state, .. } => state.value(overlap_mode),
            RawAxisBinding::Analog { value, .. } => *value,
        }
    }

    fn update(&mut self, input_change: InputChange) -> bool {
        use AxisDirection::*;
        use DigitalAxisState::*;
        use RawAxisBinding::*;
        match self {
            Digital {
                pair: (l, r),
                state,
            } => {
                let InputChange::Digital { input, state_change } = input_change else {
                    // this is an analog change
                    return *state != None;
                };
                let dir = if input == *l {
                    Low
                } else if input == *r {
                    High
                } else {
                    return *state != None;
                };
                *state = state.update(dir, state_change);
                return *state != None;
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
pub struct Axis {
    overlap_mode: OverlapMode,
    raw: Vec<RawAxisBinding>,
    active_indices: Vec<usize>,
    dirty: bool,
}

impl Axis {
    pub fn new(inputs: Vec<AxisInput>) -> Self {
        Self {
            raw: inputs.into_iter().map(RawAxisBinding::from).collect(),
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
            match r.input() {
                AxisInput::Digital(low, high) => {
                    inputs.push(AnyInput::Digital(low.into()));
                    inputs.push(AnyInput::Digital(high.into()));
                }
                AxisInput::Analog(input) => inputs.push(AnyInput::Analog(input)),
            };
        }
        inputs
    }

    pub fn changed(&self) -> bool {
        self.dirty
    }

    pub fn clear_changed(&mut self) {
        self.dirty = false;
    }

    pub fn set_overlap_mode(&mut self, overlap_mode: OverlapMode) {
        self.overlap_mode = overlap_mode;
    }

    pub fn replace_inputs(&mut self, inputs: Vec<AxisInput>) {
        self.raw = inputs.into_iter().map(RawAxisBinding::from).collect();
        self.dirty = true;
    }

    pub fn add_input(&mut self, input: AxisInput) {
        use AxisInput::*;
        for i in &self.raw {
            match (input, i.input()) {
                (Digital(a, b), Digital(c, d)) if (a, b) == (c, d) => {}
                (Analog(input), Analog(existing)) if input == existing => {}
                // TODO: make this a continue and put the assert below?
                _ => continue,
            }
            debug_assert!(false, "input {:?} is already registered", input);
            return;
        }
        self.raw.push(input.into());
        self.dirty = true;
    }

    fn clear(&mut self) {
        self.raw.iter_mut().for_each(|raw| match raw {
            RawAxisBinding::Digital { .. } => {}
            RawAxisBinding::Analog { value, .. } => *value = 0.,
        });
    }
}
