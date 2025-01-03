use std::ops::Deref;

use ttf_parser::Face;

use crate::{
    app::Context,
    color::Color,
    font::{FontAtlas, LayoutOptions},
    input::ControlSet,
};

use super::{TextureBuilder, TextureRef};

#[derive(Clone, Copy, Default, Debug)]
pub struct TextDisplayOptions {
    pub color: Color,
    pub layout: LayoutOptions,
}

#[derive(Clone)]
pub struct RenderableFont {
    font_atlas: FontAtlas,
    font_atlas_texture: TextureRef,
}

impl Deref for RenderableFont {
    type Target = FontAtlas;

    fn deref(&self) -> &Self::Target {
        &self.font_atlas
    }
}

impl RenderableFont {
    pub fn new<T: ControlSet>(ctx: &mut Context<T>) -> Self {
        // TODO: these callbacks should be able to return an error, optionally
        let ttf_bytes = include_bytes!("../../res/fonts/Ubuntu-M.ttf");
        let face = Face::parse(ttf_bytes, 0).unwrap();
        let font_atlas = FontAtlas::new(face, Default::default()).unwrap();
        let font_atlas_texture = ctx.render_state.load_texture(
            &ctx.display,
            TextureBuilder::labeled("font_atlas")
                .with_filter_mode(wgpu::FilterMode::Linear)
                .from_image(
                    ctx.display.device(),
                    ctx.display.queue(),
                    font_atlas.image(),
                ),
        );

        Self {
            font_atlas,
            font_atlas_texture,
        }
    }

    pub fn texture(&self) -> TextureRef {
        self.font_atlas_texture
    }
}
