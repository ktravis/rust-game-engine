use std::ops::{Deref, Range};

use glam::{vec3, Mat4, Quat};
use recycle_vec::VecExt;

use crate::font::FontAtlas;
use crate::sprite::Sprite;
use crate::transform::Transform;

use super::state::RenderPass;
use super::text::TextDisplayOptions;
use super::{MeshRef, PipelineRef, RawInstanceData, RenderData, TextureRef};

use super::ModelInstanceData;

#[derive(Debug)]
pub struct InstanceRenderData<T> {
    pub texture: Option<TextureRef>,
    pub pipeline: PipelineRef,
    pub mesh: MeshRef,
    pub model: ModelInstanceData<T>,
}

impl<T> Deref for InstanceRenderData<T> {
    type Target = ModelInstanceData<T>;

    fn deref(&self) -> &Self::Target {
        &self.model
    }
}

pub trait DrawInstance {
    fn draw_instance<T: Transform>(&mut self, instance: &InstanceRenderData<T>);
}

#[derive(Debug, Clone)]
pub enum InstancedDrawOp<'a> {
    Draw(Range<u32>),
    SetPipeline(PipelineRef),
    SetTexture(Option<TextureRef>),
    SetMesh(MeshRef),
    SetBindGroup(u32, &'a wgpu::BindGroup),
}

impl<'a> InstancedDrawOp<'a> {
    pub fn apply<'pass>(self, render_pass: &mut RenderPass<'pass>)
    where
        'a: 'pass,
    {
        match self {
            InstancedDrawOp::Draw(instances) => render_pass.draw_active_mesh(instances),
            InstancedDrawOp::SetPipeline(pipeline) => render_pass.set_active_pipeline(pipeline),
            InstancedDrawOp::SetTexture(texture) => render_pass.bind_texture(texture),
            InstancedDrawOp::SetMesh(mesh) => render_pass.set_active_mesh(mesh),
            InstancedDrawOp::SetBindGroup(index, bind_group) => {
                render_pass.set_bind_group(index, bind_group, &[])
            }
        }
    }
}

#[derive(Default)]
pub struct InstanceStorage {
    ops: Option<Vec<InstancedDrawOp<'static>>>,
    raw_instances: Option<Vec<RawInstanceData>>,
}

impl InstanceStorage {
    pub fn temp<'a>(&mut self) -> (Vec<InstancedDrawOp<'a>>, Vec<RawInstanceData>) {
        (
            self.ops.take().unwrap_or_default().recycle(),
            self.raw_instances.take().unwrap_or_default().recycle(),
        )
    }

    pub fn reset(
        &mut self,
        ops: Vec<InstancedDrawOp<'static>>,
        raw_instances: Vec<RawInstanceData>,
    ) {
        let _ = self.ops.insert(ops);
        let _ = self.raw_instances.insert(raw_instances);
    }
}

pub struct InstanceRenderer<'op> {
    ops: Vec<InstancedDrawOp<'op>>,
    instances: Vec<RawInstanceData>,

    active_texture: Option<TextureRef>,
    active_mesh: Option<MeshRef>,
    active_pipeline: Option<PipelineRef>,
    current_count: u32,
    range_start: u32,
}

impl<'op> InstanceRenderer<'op> {
    pub fn new(ops: Vec<InstancedDrawOp<'op>>, instances: Vec<RawInstanceData>) -> Self {
        Self {
            ops,
            instances,
            active_mesh: None,
            active_texture: None,
            active_pipeline: None,
            current_count: 0,
            range_start: 0,
        }
    }

    #[inline]
    pub fn draw_instance(&mut self, instance: &InstanceRenderData<impl Transform>) {
        if instance.pipeline != self.active_pipeline.unwrap_or_default() {
            if self.current_count > 0 {
                self.ops.push(InstancedDrawOp::Draw(
                    self.range_start..self.range_start + self.current_count,
                ));
                self.range_start += self.current_count;
                self.current_count = 0;
            }
            self.ops
                .push(InstancedDrawOp::SetPipeline(instance.pipeline));
            self.active_pipeline = Some(instance.pipeline);
        }
        if instance.mesh != self.active_mesh.unwrap_or_default() {
            if self.current_count > 0 {
                self.ops.push(InstancedDrawOp::Draw(
                    self.range_start..self.range_start + self.current_count,
                ));
                self.range_start += self.current_count;
                self.current_count = 0;
            }
            self.ops.push(InstancedDrawOp::SetMesh(instance.mesh));
            self.active_mesh = Some(instance.mesh);
        }
        if instance.texture != self.active_texture {
            if self.current_count > 0 {
                self.ops.push(InstancedDrawOp::Draw(
                    self.range_start..self.range_start + self.current_count,
                ));
                self.range_start += self.current_count;
                self.current_count = 0;
            }
            self.ops.push(InstancedDrawOp::SetTexture(instance.texture));
            self.active_texture = instance.texture;
        }
        self.current_count += 1;
        self.instances.push(instance.as_raw());
    }

    #[inline]
    pub fn draw_sprite(
        &mut self,
        sprite: &Sprite,
        render_data: &RenderData,
        frame: usize,
        transform: impl Transform,
    ) {
        let frame = &sprite.frames[frame];
        let scale = vec3(sprite.size.x as f32, sprite.size.y as f32, 1.0);
        let origin = sprite.pivot.unwrap_or_default().as_vec2();
        let transform = transform.as_mat4()
            * Mat4::from_scale_rotation_translation(scale, Quat::IDENTITY, -origin.extend(0.0));
        self.draw_instance(&render_data.for_model_instance(ModelInstanceData {
            subtexture: frame.region,
            transform,
            ..Default::default()
        }));
    }

    #[inline]
    pub fn draw_text(
        &mut self,
        s: impl AsRef<str>,
        font: &FontAtlas,
        render_data: &RenderData,
        transform: impl Transform,
        opts: TextDisplayOptions,
    ) {
        let m = transform.as_mat4();
        for glyph_data in font.layout_text(s.as_ref(), opts.layout) {
            let transform = m * Mat4::from_scale_rotation_translation(
                glyph_data.bounds.dim.extend(1.0),
                Quat::IDENTITY,
                glyph_data.bounds.pos.extend(0.0),
            );
            self.draw_instance(&render_data.for_model_instance(ModelInstanceData {
                subtexture: glyph_data.subtexture,
                tint: opts.color,
                transform,
                ..Default::default()
            }));
        }
    }

    pub fn commit(mut self) -> (Vec<InstancedDrawOp<'op>>, Vec<RawInstanceData>) {
        if self.current_count > 0 {
            self.ops.push(InstancedDrawOp::Draw(
                self.range_start..self.range_start + self.current_count,
            ));
        }
        (self.ops, self.instances)
    }
}
