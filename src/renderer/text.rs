use crate::{color::Color, font::LayoutOptions};

#[derive(Clone, Copy, Default, Debug)]
pub struct TextDisplayOptions {
    pub color: Color,
    pub layout: LayoutOptions,
}
