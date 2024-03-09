use std::ops::{Deref, Range};
use std::sync::Arc;

use glam::{vec3, Mat4, Quat};

use crate::font::FontAtlas;
use crate::sprite::Sprite;
use crate::transform::Transform;

use super::state::{BindGroupAllocator, RenderPass, ViewProjectionUniforms};
use super::text::TextDisplayOptions;
use super::{Display, MeshRef, PipelineRef, RawInstanceData, RenderData, TextureRef};

use super::ModelInstanceData;

#[derive(Debug)]
pub struct InstanceRenderData<T> {
    pub mesh: MeshRef,
    pub model: ModelInstanceData<T>,
    pub texture: Option<TextureRef>,
    pub pipeline: Option<PipelineRef>,
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
pub enum InstancedDrawOp {
    Draw(Range<u32>),
    SetPipeline(Option<PipelineRef>),
    SetTexture(Option<TextureRef>),
    SetMesh(MeshRef),
    SetBindGroup(u32, Arc<wgpu::BindGroup>),
}

impl InstancedDrawOp {
    pub fn apply<'pass, 'r: 'pass>(&'r self, render_pass: &mut RenderPass<'pass>) {
        match self {
            InstancedDrawOp::Draw(instances) => render_pass.draw_active_mesh(instances.clone()),
            InstancedDrawOp::SetPipeline(pipeline) => render_pass.set_active_pipeline(*pipeline),
            InstancedDrawOp::SetTexture(texture) => render_pass.bind_texture(*texture),
            InstancedDrawOp::SetMesh(mesh) => render_pass.set_active_mesh(*mesh),
            InstancedDrawOp::SetBindGroup(index, bind_group) => {
                render_pass.set_bind_group(*index, bind_group, &[])
            }
        }
    }
}

pub struct InstanceStorage {
    ops: Vec<InstancedDrawOp>,
    raw_instances: Vec<RawInstanceData>,
    instance_buffer: wgpu::Buffer,
}

impl InstanceStorage {
    pub fn new(display: &Display, initial_size: usize) -> Self {
        let instance_buffer = display.device().create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (initial_size * std::mem::size_of::<RawInstanceData>()) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
            mapped_at_creation: false,
        });
        Self {
            ops: vec![],
            raw_instances: vec![],
            instance_buffer,
        }
    }

    pub fn render_to<'a>(&'a self, render_pass: &mut RenderPass<'a>) {
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        self.ops.iter().for_each(|op| op.apply(render_pass));
    }

    pub fn clear(&mut self) {
        self.ops.clear();
        self.raw_instances.clear();
    }

    fn update_buffer(&mut self, display: &Display) {
        let req_size = (self.raw_instances.len() * std::mem::size_of::<RawInstanceData>())
            as wgpu::BufferAddress;
        if self.instance_buffer.size() < req_size {
            self.instance_buffer.destroy();
            self.instance_buffer = display.device().create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: req_size * 2,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
                mapped_at_creation: false,
            });
        }
        display.queue().write_buffer(
            &self.instance_buffer,
            0,
            bytemuck::cast_slice(&self.raw_instances),
        );
    }
}

pub struct InstanceRenderer<'r, 'alloc> {
    bind_group_allocator: &'r BindGroupAllocator<'alloc>,
    instance_storage: &'r mut InstanceStorage,

    active_texture: Option<TextureRef>,
    active_mesh: Option<MeshRef>,
    active_pipeline: Option<PipelineRef>,
    current_count: u32,
    range_start: u32,
}

impl<'r, 'alloc> InstanceRenderer<'r, 'alloc> {
    pub fn new(
        bind_group_allocator: &'r BindGroupAllocator<'alloc>,
        instance_storage: &'r mut InstanceStorage,
    ) -> Self {
        Self {
            bind_group_allocator,
            instance_storage,
            active_mesh: None,
            active_texture: None,
            active_pipeline: None,
            current_count: 0,
            range_start: 0,
        }
    }

    pub fn set_view_projection(&mut self, view_projection: &ViewProjectionUniforms) {
        if self.current_count > 0 {
            self.instance_storage.ops.push(InstancedDrawOp::Draw(
                self.range_start..self.range_start + self.current_count,
            ));
            self.range_start += self.current_count;
            self.current_count = 0;
        }
        self.instance_storage
            .ops
            .push(InstancedDrawOp::SetBindGroup(
                RenderPass::VIEW_PROJECTION_UNIFORMS_BIND_GROUP_INDEX,
                self.bind_group_allocator.get(view_projection),
            ));
    }

    #[inline]
    pub fn draw_instance(&mut self, instance: &InstanceRenderData<impl Transform>) {
        if instance.pipeline != self.active_pipeline {
            if self.current_count > 0 {
                self.instance_storage.ops.push(InstancedDrawOp::Draw(
                    self.range_start..self.range_start + self.current_count,
                ));
                self.range_start += self.current_count;
                self.current_count = 0;
            }
            self.instance_storage
                .ops
                .push(InstancedDrawOp::SetPipeline(instance.pipeline));
            self.active_pipeline = instance.pipeline;
        }
        if instance.mesh != self.active_mesh.unwrap_or_default() {
            if self.current_count > 0 {
                self.instance_storage.ops.push(InstancedDrawOp::Draw(
                    self.range_start..self.range_start + self.current_count,
                ));
                self.range_start += self.current_count;
                self.current_count = 0;
            }
            self.instance_storage
                .ops
                .push(InstancedDrawOp::SetMesh(instance.mesh));
            self.active_mesh = Some(instance.mesh);
        }
        if instance.texture != self.active_texture {
            if self.current_count > 0 {
                self.instance_storage.ops.push(InstancedDrawOp::Draw(
                    self.range_start..self.range_start + self.current_count,
                ));
                self.range_start += self.current_count;
                self.current_count = 0;
            }
            self.instance_storage
                .ops
                .push(InstancedDrawOp::SetTexture(instance.texture));
            self.active_texture = instance.texture;
        }
        self.current_count += 1;
        self.instance_storage.raw_instances.push(instance.as_raw());
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

    pub fn commit(self, display: &Display) {
        if self.current_count > 0 {
            self.instance_storage.ops.push(InstancedDrawOp::Draw(
                self.range_start..self.range_start + self.current_count,
            ));
        }
        self.instance_storage.update_buffer(display);
    }
}
