use glam::{Mat4, Quat, Vec2};

use crate::{
    color::Color,
    font::{FontAtlas, LayoutOptions},
    transform::{Transform, Transform3D},
};

use super::{instance::DrawInstance, ModelInstanceData, RenderData};

#[derive(Clone, Copy, Default, Debug)]
pub struct TextDisplayOptions {
    pub color: Color,
    pub layout: LayoutOptions,
}

pub trait DrawText {
    fn draw_text(&mut self, s: impl AsRef<str>, transform: Transform3D, opts: TextDisplayOptions);
    fn draw_text_2d(&mut self, s: impl AsRef<str>, position: Vec2, opts: TextDisplayOptions) {
        self.draw_text(
            s,
            Transform3D {
                position: position.extend(0.0),
                ..Default::default()
            },
            opts,
        )
    }
}

pub trait MakeFontFaceRenderer: Sized + DrawInstance {
    fn font_face_renderer<'a>(
        &'a mut self,
        font_atlas: &'a FontAtlas,
        render_data: RenderData,
    ) -> FontFaceRenderer<'a, Self>;
}

impl<T: Sized + DrawInstance> MakeFontFaceRenderer for T {
    fn font_face_renderer<'a>(
        &'a mut self,
        font_atlas: &'a FontAtlas,
        render_data: RenderData,
    ) -> FontFaceRenderer<'a, Self> {
        FontFaceRenderer {
            raw: self,
            font_atlas,
            render_data,
        }
    }
}

pub struct FontFaceRenderer<'a, T: DrawInstance> {
    raw: &'a mut T,
    font_atlas: &'a FontAtlas,
    render_data: RenderData,
}

impl<'a, T: DrawInstance> DrawText for FontFaceRenderer<'a, T> {
    fn draw_text(&mut self, s: impl AsRef<str>, transform: Transform3D, opts: TextDisplayOptions) {
        let m = transform.as_mat4();
        for glyph_data in self.font_atlas.layout_text(s.as_ref(), opts.layout) {
            let transform = m * Mat4::from_scale_rotation_translation(
                glyph_data.bounds.dim.extend(1.0),
                Quat::IDENTITY,
                glyph_data.bounds.pos.extend(0.0),
            );
            self.raw
                .draw_instance(&self.render_data.for_model_instance(ModelInstanceData {
                    subtexture: glyph_data.subtexture,
                    tint: opts.color,
                    transform,
                    ..Default::default()
                }));
        }
    }
}
