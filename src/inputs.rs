pub use miniquad::KeyCode;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum KeyStateChange {
    Pressed,
    Released,
}

#[derive(Debug, Default, Copy, Clone)]
pub struct ButtonState {
    pub just_pressed: bool,
    pub is_down: bool,
}

impl ButtonState {
    fn update(&mut self, change: KeyStateChange) {
        match change {
            KeyStateChange::Pressed => {
                self.just_pressed = !self.is_down;
                self.is_down = true;
            }
            KeyStateChange::Released => {
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
    fn update(&mut self, key: KeyCode, change: KeyStateChange) {
        match self {
            Binding::Button(b) => b.update(key, change),
            Binding::Axis(a) => a.update(key, change),
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
    keys: Vec<KeyCode>,
    triggered_key: Option<KeyCode>,
    // TODO: timing window for press?
}

impl ButtonBinding {
    /// Returns whether this [`VirtualButton`] is down.
    pub fn is_down(&self) -> bool {
        self.state.is_down
    }

    pub fn just_pressed(&self) -> bool {
        self.state.just_pressed
    }

    fn update(&mut self, key: KeyCode, change: KeyStateChange) {
        match self.triggered_key {
            Some(k) if k == key => {
                self.state.update(change);
                if change == KeyStateChange::Released {
                    self.triggered_key.take();
                }
            }
            // the triggered key does not match, no change
            Some(_) => {}
            None => {
                if change == KeyStateChange::Pressed {
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

#[derive(Copy, Clone, Debug)]
enum AxisDirection {
    Left,
    Right,
}

#[derive(Debug, Default, Clone)]
pub struct AxisBinding {
    pub name: String,
    /// 0: not held, 1: held first, 2: held second
    state: (i8, i8),
    overlap: OverlapMode,
    keypairs: Vec<(KeyCode, KeyCode)>,
    activated: Option<(KeyCode, KeyCode)>,
}

impl AxisBinding {
    /// Value from -1.0 to 1.0
    pub fn value(&self) -> f32 {
        match self.state {
            (0, 0) => 0.,
            (0, _r) => 1.,
            (_l, 0) => -1.,
            (l, r) => match self.overlap {
                OverlapMode::First => -1. * (r - l) as f32,
                OverlapMode::Latest => (r - l) as f32,
                OverlapMode::Neutral => 0.,
            },
        }
    }

    fn update(&mut self, key: KeyCode, change: KeyStateChange) {
        use AxisDirection::*;
        use KeyStateChange::*;
        let dir = if let Some((l, r)) = self.activated {
            if key == l {
                Left
            } else if key == r {
                Right
            } else {
                // there is a different axis being held
                return;
            }
        } else {
            let (dir, activated) = self
                .keypairs
                .iter()
                .find_map(|(l, r)| {
                    if key == *l {
                        Some((AxisDirection::Left, (*l, *r)))
                    } else if key == *r {
                        Some((AxisDirection::Right, (*l, *r)))
                    } else {
                        None
                    }
                })
                .unwrap();
            self.activated = Some(activated);
            dir
        };
        match self.state {
            (0, 0) => {
                if change == KeyStateChange::Pressed {
                    self.state = match dir {
                        Left => (1, 0),
                        Right => (0, 1),
                    }
                }
            }
            (0, _r) => match (change, dir) {
                (Pressed, Left) => self.state = (2, 1),
                (Released, Right) => self.state = (0, 0),
                _ => {}
            },
            (_l, 0) => match (change, dir) {
                (Released, Left) => self.state = (0, 0),
                (Pressed, Right) => self.state = (1, 2),
                _ => {}
            },
            (l, r) => match (change, dir) {
                (Released, Left) => self.state = (0, 1),
                (Released, Right) => self.state = (1, 0),
                _ => {}
            },
        }
        if self.state == (0, 0) {
            self.activated.take();
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct BindingRef(usize);

// impl Deref for VirtualButtonRef {
//     type Target = VirtualButton;

//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }

// impl DerefMut for VirtualButtonRef {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.0
//     }
// }

pub struct ButtonBuilder<'a> {
    input: &'a mut Input,
    button: ButtonBinding,
}

impl<'a> ButtonBuilder<'a> {
    pub fn with_key(mut self, key: KeyCode) -> Self {
        self.button.keys.push(key);
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
    pub fn with_keys(mut self, left: KeyCode, right: KeyCode) -> Self {
        self.axis.keypairs.push((left, right));
        self
    }

    pub fn overlap_mode(mut self, overlap: OverlapMode) -> Self {
        self.axis.overlap = overlap;
        self
    }

    pub fn register(self) -> BindingRef {
        debug_assert!(!self.axis.keypairs.is_empty());
        self.input.register_axis(self.axis)
    }
}

#[derive(Default, Debug)]
pub struct Input {
    bindings: Vec<Binding>,
    by_name: std::collections::HashMap<String, BindingRef>,
    keycodes: std::collections::HashMap<KeyCode, BindingRef>,
    buffered_keys: Vec<(KeyCode, KeyStateChange)>,
}

impl Input {
    pub fn update(&mut self) {
        for b in &mut self.bindings {
            if let Binding::Button(b) = b {
                b.state.just_pressed = false;
            }
        }
        for (key, change) in self.buffered_keys.drain(..) {
            let b = self
                .keycodes
                .get(&key)
                .unwrap_or_else(|| panic!("key {:?} not registered", key));
            self.bindings[b.0].update(key, change);
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

    pub fn handle_key_change(&mut self, key: KeyCode, change: KeyStateChange) {
        // if we don't have any buttons registered for this key, just ignore
        if self.keycodes.get(&key).is_none() {
            return;
        }
        self.buffered_keys.push((key, change));
    }

    fn register_button(&mut self, b: ButtonBinding) -> BindingRef {
        let index = self.bindings.len();
        let button_ref = BindingRef(index);
        for key in &b.keys {
            self.keycodes.insert(*key, button_ref);
        }
        self.by_name.insert(b.name.clone(), button_ref);
        self.bindings.push(Binding::Button(b));
        button_ref
    }

    fn register_axis(&mut self, a: AxisBinding) -> BindingRef {
        let index = self.bindings.len();
        let button_ref = BindingRef(index);
        for (l, r) in &a.keypairs {
            self.keycodes.insert(*l, button_ref);
            self.keycodes.insert(*r, button_ref);
        }
        self.by_name.insert(a.name.clone(), button_ref);
        self.bindings.push(Binding::Axis(a));
        button_ref
    }
}
