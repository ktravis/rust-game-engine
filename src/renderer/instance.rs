use std::ops::{Deref, Range};
use std::sync::Arc;

use glam::{vec3, Mat4, Quat};

use crate::font::FontAtlas;
use crate::geom::{ModelVertexData, VertexData};
use crate::sprite::Sprite;
use crate::transform::Transform;

use super::mesh::RawMeshRef;
use super::state::{BindGroupAllocator, RawPipelineRef, RenderPass, ViewProjectionUniforms};
use super::text::TextDisplayOptions;
use super::{Display, InstanceData, MeshRef, PipelineRef, RenderData, TextureRef};

use super::ModelInstanceData;

#[derive(Debug)]
pub struct InstanceRenderData<V = ModelVertexData, I = ModelInstanceData> {
    pub mesh: MeshRef<V>,
    pub model: I,
    pub texture: Option<TextureRef>,
    pub pipeline: Option<PipelineRef<V, I>>,
}

impl<V, I> Deref for InstanceRenderData<V, I> {
    type Target = I;

    fn deref(&self) -> &Self::Target {
        &self.model
    }
}

#[derive(Debug, Clone)]
enum InstancedDrawOp {
    Draw(Range<u32>),
    SetPipeline(Option<RawPipelineRef>),
    SetTexture(Option<TextureRef>),
    SetMesh(RawMeshRef),
    SetBindGroup(u32, Arc<wgpu::BindGroup>),
}

impl InstancedDrawOp {
    pub fn apply<'pass, 'r: 'pass>(&'r self, render_pass: &mut RenderPass<'pass>) {
        match self {
            InstancedDrawOp::Draw(instances) => render_pass.draw_active_mesh(instances.clone()),
            InstancedDrawOp::SetPipeline(pipeline) => {
                render_pass.set_active_pipeline_raw(*pipeline)
            }
            InstancedDrawOp::SetTexture(texture) => render_pass.bind_texture(*texture),
            InstancedDrawOp::SetMesh(mesh) => render_pass.set_active_mesh_raw(*mesh),
            InstancedDrawOp::SetBindGroup(index, bind_group) => {
                render_pass.set_bind_group(*index, bind_group, &[])
            }
        }
    }
}

pub struct InstanceStorage {
    ops: Vec<InstancedDrawOp>,
    raw_instance_bytes: Vec<u8>,
    instance_buffer: wgpu::Buffer,
}

impl InstanceStorage {
    pub fn new(display: &Display, initial_size: usize) -> Self {
        let instance_buffer = display.device().create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (initial_size * std::mem::size_of::<ModelInstanceData>()) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
            mapped_at_creation: false,
        });
        Self {
            ops: vec![],
            raw_instance_bytes: vec![],
            instance_buffer,
        }
    }

    pub fn render_to<'a>(&'a self, render_pass: &mut RenderPass<'a>) {
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        self.ops.iter().for_each(|op| op.apply(render_pass));
    }

    pub fn clear(&mut self) {
        self.ops.clear();
        self.raw_instance_bytes.clear();
    }

    fn update_buffer(&mut self, display: &Display) {
        let req_size = self.raw_instance_bytes.len() as wgpu::BufferAddress;
        if self.instance_buffer.size() < req_size {
            self.instance_buffer.destroy();
            self.instance_buffer = display.device().create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: req_size * 2,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
                mapped_at_creation: false,
            });
        }
        display
            .queue()
            .write_buffer(&self.instance_buffer, 0, &self.raw_instance_bytes);
    }
}

pub struct InstanceRenderer<'r, 'alloc> {
    bind_group_allocator: &'r BindGroupAllocator<'alloc, ViewProjectionUniforms>,
    instance_storage: &'r mut InstanceStorage,

    active_texture: Option<TextureRef>,
    active_mesh: Option<RawMeshRef>,
    active_pipeline: Option<RawPipelineRef>,
    current_count: u32,
    range_start: u32,
}

impl<'r, 'alloc> InstanceRenderer<'r, 'alloc> {
    pub fn new(
        bind_group_allocator: &'r BindGroupAllocator<'alloc, ViewProjectionUniforms>,
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
    pub fn draw_instance<V: VertexData, I: InstanceData>(
        &mut self,
        instance: &InstanceRenderData<V, I>,
    ) {
        let pipeline = instance.pipeline.map(|p| p.raw());
        if pipeline != self.active_pipeline {
            if self.current_count > 0 {
                self.instance_storage.ops.push(InstancedDrawOp::Draw(
                    self.range_start..self.range_start + self.current_count,
                ));
                self.range_start += self.current_count;
                self.current_count = 0;
            }
            self.instance_storage
                .ops
                .push(InstancedDrawOp::SetPipeline(pipeline));
            self.active_pipeline = pipeline;
        }
        let mesh = instance.mesh.raw();
        if mesh != self.active_mesh.unwrap_or_default() {
            if self.current_count > 0 {
                self.instance_storage.ops.push(InstancedDrawOp::Draw(
                    self.range_start..self.range_start + self.current_count,
                ));
                self.range_start += self.current_count;
                self.current_count = 0;
            }
            self.instance_storage
                .ops
                .push(InstancedDrawOp::SetMesh(mesh));
            self.active_mesh = Some(mesh);
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
        self.instance_storage
            .raw_instance_bytes
            // .extend_from_slice(bytemuck::cast_slice(&[instance.as_raw()]));
            .extend_from_slice(bytemuck::bytes_of(&instance.model));
    }

    // I is more constrained here because we want to be able to set `subtexture`. Maybe we could
    // come up with a TexturedInstanceData trait?
    #[inline]
    pub fn draw_sprite<V: VertexData>(
        &mut self,
        sprite: &Sprite,
        render_data: &RenderData<V, ModelInstanceData>,
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
    pub fn draw_text<V: VertexData>(
        &mut self,
        s: impl AsRef<str>,
        font: &FontAtlas,
        render_data: &RenderData<V, ModelInstanceData>,
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
