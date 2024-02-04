use glam::Vec2;

use crate::{
    color::Color,
    font::{FontAtlas, LayoutOptions},
    transform::Transform,
};

use super::{
    instance::{DrawInstance, InstanceRenderData},
    MeshRef, ModelInstanceData, PipelineRef, TextureRef,
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

// TODO: find a better plan for this so the fields don't need to all be pub?
pub struct FontFaceRenderer<'a, T: DrawInstance> {
    pub raw: &'a mut T,
    pub font_atlas: &'a FontAtlas,
    pub pipeline: PipelineRef,
    pub texture: TextureRef,
    pub quad_mesh: MeshRef,
}

impl<'a, T: DrawInstance> DrawText for FontFaceRenderer<'a, T> {
    fn draw_text(&mut self, s: impl AsRef<str>, transform: Transform, opts: TextDisplayOptions) {
        for glyph_data in self.font_atlas.layout_text(s.as_ref(), opts.layout) {
            self.raw.draw_instance(&InstanceRenderData {
                texture: Some(self.texture),
                pipeline: self.pipeline,
                mesh: self.quad_mesh,
                model: ModelInstanceData {
                    subtexture: glyph_data.subtexture,
                    tint: opts.color,
                    transform: Transform {
                        position: glyph_data.bounds.pos.extend(0.0) * transform.scale
                            + transform.position,
                        scale: glyph_data.bounds.dim.extend(1.0) * transform.scale,
                        ..transform
                    },
                    ..Default::default()
                },
            });
        }
    }
}
