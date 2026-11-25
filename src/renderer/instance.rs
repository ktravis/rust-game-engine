use std::ops::{Deref, Range};

use glam::{Mat3, Mat4};
use shadertype_derive::{ShaderNamedType, VertexInput};

use crate::color::Color;
use crate::geom::Rect;
use crate::renderer::shader_type::VertexInput;
use crate::renderer::state::MeshRef;
use crate::transform::Transform;

use super::Display;

#[derive(Clone, Debug, Default)]
pub struct ObjectData<T: Transform> {
    pub mesh: MeshRef,
    pub texture: Option<u32>,
    pub casts_shadow: bool,

    pub subtexture: Option<Rect>,
    pub tint: Color,
    pub transform: T,
    pub normal_matrix: Mat3,
    pub material: u32,
}

#[derive(Debug)]
pub struct InstanceRenderData<I = BasicInstanceData> {
    pub mesh: MeshRef,
    pub instance: I,
    pub texture: Option<u32>,
    pub casts_shadow: bool,
}

impl<I> Deref for InstanceRenderData<I> {
    type Target = I;

    fn deref(&self) -> &Self::Target {
        &self.instance
    }
}

// TODO: rename "PerInstanceData"

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, ShaderNamedType, VertexInput)]
#[step_mode(instance)] // TODO: this should be #[vertex_input(step_mode = instance)]
pub struct BasicInstanceData {
    pub subtexture: Rect,
    pub tint: Color,
    pub transform: Mat4,
}

impl BasicInstanceData {
    #[inline]
    pub fn transform(transform: impl Transform) -> Self {
        Self {
            transform: transform.as_mat4(),
            ..Default::default()
        }
    }
}

impl<T: Transform> From<&ObjectData<T>> for BasicInstanceData {
    fn from(value: &ObjectData<T>) -> Self {
        Self {
            subtexture: value.subtexture.unwrap_or_default(),
            tint: value.tint,
            transform: value.transform.as_mat4(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, ShaderNamedType, VertexInput)]
#[step_mode(instance)] // TODO: this should be #[vertex_input(step_mode = instance)]
pub struct InstanceDataWithNormalMatrix {
    pub subtexture: Rect,
    pub tint: Color,
    pub transform: Mat4,
    pub normal_matrix: Mat3,
    pub material: u32,
    pub texture: u32,
}

impl InstanceDataWithNormalMatrix {
    pub fn from_basic(other: BasicInstanceData, view_matrix: Mat4) -> Self {
        Self {
            transform: other.transform,
            tint: other.tint,
            subtexture: other.subtexture,
            // TODO: this is the big performance killer, we need to not do this every frame
            normal_matrix: Mat3::from_mat4(view_matrix * other.transform)
                .inverse()
                .transpose(),
            material: 0,
            texture: 0,
        }
    }
}

impl<T: Transform> From<&ObjectData<T>> for InstanceDataWithNormalMatrix {
    fn from(value: &ObjectData<T>) -> Self {
        Self {
            subtexture: value.subtexture.unwrap_or_default(),
            tint: value.tint,
            transform: value.transform.as_mat4(),
            normal_matrix: value.normal_matrix,
            material: value.material,
            texture: value.texture.unwrap_or_default(),
        }
    }
}

pub struct InstanceStorage {
    pub draw_range: Range<u32>,
    raw_instance_bytes: Vec<u8>,
    instance_buffer: Option<wgpu::Buffer>,
    destroy_list: Vec<wgpu::Buffer>,
}

impl InstanceStorage {
    pub fn new(display: &Display, initial_size: usize) -> Self {
        let instance_buffer = display.device().create_buffer(&wgpu::BufferDescriptor {
            label: Some("instance storage"),
            size: (initial_size * std::mem::size_of::<BasicInstanceData>()) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
            mapped_at_creation: false,
        });
        Self {
            raw_instance_bytes: vec![],
            instance_buffer: Some(instance_buffer),
            destroy_list: vec![],
            draw_range: 0..0,
        }
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        self.instance_buffer.as_ref().unwrap()
    }

    pub fn add<I: VertexInput>(&mut self, instance: &I) {
        self.draw_range.end += 1;
        self.raw_instance_bytes
            .extend_from_slice(bytemuck::bytes_of(instance));
    }

    pub fn clear(&mut self) {
        self.draw_range = 0..0;
        self.raw_instance_bytes.clear();
        for b in self.destroy_list.drain(..) {
            b.destroy();
        }
    }

    pub fn update_buffer(&mut self, display: &Display) -> &wgpu::Buffer {
        let req_size = self.raw_instance_bytes.len() as wgpu::BufferAddress;
        if self.buffer().size() < req_size {
            self.destroy_list.push(self.instance_buffer.take().unwrap());
            self.instance_buffer = Some(display.device().create_buffer(&wgpu::BufferDescriptor {
                label: Some("instance storage (resized)"),
                size: req_size * 2,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
                mapped_at_creation: false,
            }));
        }
        let buf = self.buffer();
        if req_size > 0 {
            let mut w = display
                .queue()
                .write_buffer_with(buf, 0, std::num::NonZeroU64::new(req_size).unwrap())
                .unwrap();
            w.copy_from_slice(&self.raw_instance_bytes);
        }
        buf
    }
}
