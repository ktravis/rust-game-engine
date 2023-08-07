mod binding;
mod manager;

pub use binding::*;
pub use manager::*;

macro_rules! declare_inputs {
    ($mod_name:ident :: $name:ident {
        axes [
            $($axis_name:ident => $($axis_value:expr),+;)*
        ]
        buttons [
            $($button_name:ident => $($button_value:expr),+;)*
        ]
    }) => {
        mod $mod_name {
            #[derive(Debug)]
            pub struct Axes {
                $(pub $axis_name: $crate::input::AxisBinding<ControlSet>),*
            }
            #[derive(Debug)]
            pub struct Buttons {
                $(pub $button_name: $crate::input::ButtonBinding<ControlSet>),*
            }

            #[allow(non_camel_case_types)]
            #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
            pub enum ControlSet {
                $($axis_name,)*
                $($button_name,)*
            }
        }

        #[derive(Debug)]
        pub struct $name {
            pub axes: $mod_name::Axes,
            pub buttons: $mod_name::Buttons,
            inputs_by_control: ::std::collections::HashMap<$mod_name::ControlSet, Vec<$crate::input::AnyInput>>,
            controls_by_input: ::std::collections::HashMap<$crate::input::AnyInput, Vec<$mod_name::ControlSet>>,
        }

        impl $crate::input::ControlsManager for $name {
            type ControlSet = $mod_name::ControlSet;
            fn handle_input(&mut self, change: $crate::input::InputChange) {
                let Some(controls) = self.controls_by_input.get(&change.input()) else {
                  return;
                };
                for control in controls {
                    use $mod_name::ControlSet::*;
                    match control {
                        $($axis_name => self.axes.$axis_name.update(Some(change)),)*
                        $($button_name => self.buttons.$button_name.update(Some(change)),)*
                    }
                }
            }

            fn controls<'a>() -> &'a [Self::ControlSet] {
                use $mod_name::ControlSet::*;
                &[
                    $($axis_name,)*
                    $($button_name,)*
                ]
            }

            fn bound_inputs(&self, control: &$mod_name::ControlSet) -> Vec<$crate::input::AnyInput> {
                use $mod_name::ControlSet::*;
                match control {
                    $($axis_name => self.axes.$axis_name.bound_inputs(),)*
                    $($button_name => self.buttons.$button_name.bound_inputs(),)*
                }
            }

            fn end_frame_update(&mut self) {
                if (
                    $(self.axes.$axis_name.changed() || )*
                    $(self.buttons.$button_name.changed() ||)*
                    false
                ) {
                    self.update_mappings();
                }
                for control in Self::controls() {
                    self.update_control(*control, None);
                }
            }
        }

        impl $name {
            fn update_control(&mut self, control: $mod_name::ControlSet, change: Option<$crate::input::InputChange>) {
                use $mod_name::ControlSet::*;
                match control {
                    $($axis_name => self.axes.$axis_name.update(change),)*
                    $($button_name => self.buttons.$button_name.update(change),)*
                }
            }
            fn update_mappings(&mut self) {
                self.inputs_by_control.clear();
                self.controls_by_input.clear();

                $(
                    self.inputs_by_control.insert($mod_name::ControlSet::$axis_name, self.axes.$axis_name.bound_inputs());
                    self.axes.$axis_name.bound_inputs().iter().for_each(|i| {
                        self.controls_by_input.entry(*i).or_default().push($mod_name::ControlSet::$axis_name);
                    });
                    self.axes.$axis_name.dirty = false;
                )*
                $(
                    self.inputs_by_control.insert($mod_name::ControlSet::$button_name, self.buttons.$button_name.bound_inputs());
                    self.buttons.$button_name.bound_inputs().iter().for_each(|i| {
                        self.controls_by_input.entry(*i).or_default().push($mod_name::ControlSet::$button_name);
                    });
                    self.buttons.$button_name.dirty = false;
                )*
            }
        }

        impl Default for $name {
            fn default() -> Self {
                let inputs_by_control: ::std::collections::HashMap<$mod_name::ControlSet, Vec<$crate::input::AnyInput>> = Default::default();
                let controls_by_input: ::std::collections::HashMap<$crate::input::AnyInput, Vec<$mod_name::ControlSet>> = Default::default();
                $(
                    let $axis_name = $crate::input::AxisBinding::new(
                        $mod_name::ControlSet::$axis_name,
                        vec![
                            $($axis_value.into(),)*
                        ],
                    );
                )*
                $(
                    let $button_name = $crate::input::ButtonBinding::new(
                        $mod_name::ControlSet::$button_name,
                        vec![
                            $($button_value.into(),)*
                        ],
                    );
                )*
                let mut x = Self {
                    axes: $mod_name::Axes {
                        $($axis_name,)*
                    },
                    buttons: $mod_name::Buttons {
                        $($button_name,)*
                    },
                    inputs_by_control,
                    controls_by_input,
                };
                x.update_mappings();
                x
            }
        }
    };
    ($name:ident { $($rest:tt)* }) => {
        declare_inputs! {
            inputs::$name { $($rest)* }
        }
    }
}
