use super::{
    AnalogInput, AxisBinding, ButtonBinding, Input, KeyCodeOrMouseButton, OverlapMode,
    RawAxisBinding,
};

#[derive(Debug, Copy, Clone)]
pub struct BindingRef(pub(crate) usize);

pub struct ButtonBuilder<'a> {
    input: &'a mut Input,
    button: ButtonBinding,
}

impl<'a> ButtonBuilder<'a> {
    pub(super) fn for_input(input: &'a mut Input, name: String) -> Self {
        Self {
            input,
            button: ButtonBinding {
                name,
                ..Default::default()
            },
        }
    }

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
    pub(super) fn for_input(input: &'a mut Input, name: String) -> Self {
        Self {
            input,
            axis: AxisBinding {
                name,
                ..Default::default()
            },
        }
    }

    pub fn with_keys(
        mut self,
        left: impl Into<KeyCodeOrMouseButton>,
        right: impl Into<KeyCodeOrMouseButton>,
    ) -> Self {
        self.axis.raw.push((left.into(), right.into()).into());
        self
    }

    pub fn with_analog_input_axis(mut self, analog_input_axis: AnalogInput) -> Self {
        self.axis.raw.push(RawAxisBinding::Analog {
            input: analog_input_axis,
            value: 0.,
        });
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
