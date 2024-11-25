use std::ops::{Deref, Range};
use std::sync::Arc;

use glam::{vec3, Mat4, Quat};

use crate::color::Color;
use crate::font::FontAtlas;
use crate::geom::{BasicVertexData, Rect, VertexData};
use crate::sprite::Sprite;
use crate::transform::{Transform, Transform2D};

use super::mesh::RawMeshRef;
use super::state::{BindGroupAllocator, BindingSlot, RenderPass, ViewProjectionUniforms};
use super::text::TextDisplayOptions;
use super::{Display, InstanceData, MeshRef, PipelineRef, RawPipelineRef, RenderData, TextureRef};

use super::BasicInstanceData;

#[derive(Debug)]
pub struct InstanceRenderData<V = BasicVertexData, I = BasicInstanceData> {
    pub mesh: MeshRef<V>,
    pub instance: I,
    pub texture: Option<TextureRef>,
    pub pipeline: Option<PipelineRef<V, I>>,
}

impl<V, I> Deref for InstanceRenderData<V, I> {
    type Target = I;

    fn deref(&self) -> &Self::Target {
        &self.instance
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
            InstancedDrawOp::Draw(instances) => {
                render_pass.draw_active_mesh_instanced(instances.clone())
            }
            InstancedDrawOp::SetPipeline(pipeline) => {
                render_pass.set_active_pipeline_raw(*pipeline)
            }
            InstancedDrawOp::SetTexture(texture) => render_pass.bind_texture(*texture),
            InstancedDrawOp::SetMesh(mesh) => render_pass.set_active_mesh_raw(*mesh),
            InstancedDrawOp::SetBindGroup(index, bind_group) => {
                render_pass.set_bind_group(*index, bind_group, &[])
            }
        };
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
            size: (initial_size * std::mem::size_of::<BasicInstanceData>()) as u64,
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
    quad_mesh: MeshRef<BasicVertexData>,
    default_pipeline: RawPipelineRef,

    active_texture: Option<TextureRef>,
    active_mesh: Option<RawMeshRef>,
    active_pipeline: Option<RawPipelineRef>,
    current_draw_range: Range<u32>,
}

impl<'r, 'alloc> InstanceRenderer<'r, 'alloc> {
    const DEFAULT_BIND_GROUP_COUNT: u32 = 3;

    pub(super) fn new(
        bind_group_allocator: &'r BindGroupAllocator<'alloc, ViewProjectionUniforms>,
        instance_storage: &'r mut InstanceStorage,
        quad_mesh: MeshRef<BasicVertexData>,
        default_pipeline: RawPipelineRef,
    ) -> Self {
        Self {
            bind_group_allocator,
            instance_storage,
            quad_mesh,
            default_pipeline,
            active_mesh: None,
            active_texture: None,
            active_pipeline: None,
            current_draw_range: 0..0,
        }
    }

    fn flush_draw_calls(&mut self) {
        if self.current_draw_range.is_empty() {
            return;
        }
        if self.active_pipeline.is_none() {
            self.active_pipeline = Some(self.default_pipeline);
            self.instance_storage
                .ops
                .push(InstancedDrawOp::SetPipeline(self.active_pipeline));
        }
        self.instance_storage
            .ops
            .push(InstancedDrawOp::Draw(self.current_draw_range.clone()));
        self.current_draw_range.start = self.current_draw_range.end
    }

    pub fn set_bind_group(&mut self, index: u32, bind_group: Arc<wgpu::BindGroup>) {
        self.flush_draw_calls();
        self.instance_storage
            .ops
            .push(InstancedDrawOp::SetBindGroup(index, bind_group));
    }

    pub fn set_view_projection(&mut self, view_projection: &ViewProjectionUniforms) {
        self.flush_draw_calls();
        self.instance_storage
            .ops
            .push(InstancedDrawOp::SetBindGroup(
                RenderPass::VIEW_PROJECTION_UNIFORMS_BIND_GROUP_INDEX,
                self.bind_group_allocator.get(view_projection),
            ));
    }

    pub fn bind<T: BindingSlot>(&mut self, binding_slot: T) {
        self.flush_draw_calls();
        self.instance_storage
            .ops
            .push(InstancedDrawOp::SetBindGroup(
                Self::DEFAULT_BIND_GROUP_COUNT + binding_slot.slot(),
                binding_slot.value().clone(),
            ));
    }

    #[inline]
    pub fn draw_instance<V: VertexData, I: InstanceData>(
        &mut self,
        instance: &InstanceRenderData<V, I>,
    ) {
        let pipeline = instance.pipeline.map(|p| p.raw());
        if pipeline != self.active_pipeline {
            self.flush_draw_calls();
            self.instance_storage
                .ops
                .push(InstancedDrawOp::SetPipeline(pipeline));
            self.active_pipeline = pipeline;
        }
        let mesh = instance.mesh.raw();
        if mesh != self.active_mesh.unwrap_or_default() {
            self.flush_draw_calls();
            self.instance_storage
                .ops
                .push(InstancedDrawOp::SetMesh(mesh));
            self.active_mesh = Some(mesh);
        }
        if instance.texture != self.active_texture {
            self.flush_draw_calls();
            self.instance_storage
                .ops
                .push(InstancedDrawOp::SetTexture(instance.texture));
            self.active_texture = instance.texture;
        }
        self.current_draw_range.end += 1;
        self.instance_storage
            .raw_instance_bytes
            // .extend_from_slice(bytemuck::cast_slice(&[instance.as_raw()]));
            .extend_from_slice(bytemuck::bytes_of(&instance.instance));
    }

    // I is more constrained here because we want to be able to set `subtexture`. Maybe we could
    // come up with a TexturedInstanceData trait?
    #[inline]
    pub fn draw_sprite<V: VertexData>(
        &mut self,
        sprite: &Sprite,
        render_data: &RenderData<V, BasicInstanceData>,
        frame: usize,
        transform: impl Transform,
    ) {
        let frame = &sprite.frames[frame];
        let scale = vec3(sprite.size.x as f32, sprite.size.y as f32, 1.0);
        let origin = sprite.pivot.unwrap_or_default().as_vec2();
        let transform = transform.as_mat4()
            * Mat4::from_scale_rotation_translation(scale, Quat::IDENTITY, -origin.extend(0.0));
        self.draw_instance(&render_data.for_instance(BasicInstanceData {
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
        render_data: &RenderData<V, BasicInstanceData>,
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
            self.draw_instance(&render_data.for_instance(BasicInstanceData {
                subtexture: glyph_data.subtexture,
                tint: opts.color,
                transform,
                ..Default::default()
            }));
        }
    }

    #[inline]
    pub fn draw_quad(&mut self, texture: impl Into<Option<TextureRef>>, transform: impl Transform) {
        self.draw_quad_ex(texture.into(), transform, Color::WHITE, Rect::default())
    }

    #[inline]
    pub fn draw_quad_ex(
        &mut self,
        texture: Option<TextureRef>,
        transform: impl Transform,
        tint: Color,
        subtexture: Rect,
    ) {
        let transform = transform.as_mat4();
        self.draw_instance(&InstanceRenderData {
            mesh: self.quad_mesh,
            instance: BasicInstanceData {
                transform,
                tint,
                subtexture,
            },
            texture,
            pipeline: None,
        });
    }

    #[inline]
    pub fn draw_rect(&mut self, texture: impl Into<Option<TextureRef>>, rect: Rect) {
        self.draw_quad(
            texture,
            Transform2D {
                position: rect.pos,
                scale: rect.dim,
                rotation_rad: 0.0,
            },
        );
    }

    pub fn commit(mut self, display: &Display) {
        self.flush_draw_calls();
        self.instance_storage.update_buffer(display);
    }
}
