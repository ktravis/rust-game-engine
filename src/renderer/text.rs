use glam::Vec2;

use crate::{
    color::Color,
    font::{FontAtlas, LayoutOptions},
    transform::Transform,
};

use super::{
    instance::{DrawInstance, InstanceRenderData},
    MeshRef, ModelInstanceData, PipelineRef, RenderData, TextureRef,
};

#[derive(Clone, Copy, Default, Debug)]
pub struct TextDisplayOptions {
    pub color: Color,
    pub layout: LayoutOptions,
}

pub trait DrawText {
    fn draw_text(&mut self, s: impl AsRef<str>, transform: Transform, opts: TextDisplayOptions);
    fn draw_text_2d(&mut self, s: impl AsRef<str>, position: Vec2, opts: TextDisplayOptions) {
        self.draw_text(
            s,
            Transform {
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
    fn draw_text(&mut self, s: impl AsRef<str>, transform: Transform, opts: TextDisplayOptions) {
        let m = transform.as_matrix();
        for glyph_data in self.font_atlas.layout_text(s.as_ref(), opts.layout) {
            let transform = Transform::from_matrix(
                m * Transform {
                    position: glyph_data.bounds.pos.extend(0.0),
                    scale: glyph_data.bounds.dim.extend(1.0),
                    ..Default::default()
                }
                .as_matrix(),
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
