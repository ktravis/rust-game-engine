use super::instance::InstanceRenderData;
use super::{MeshRef, PipelineRef, TextureRef};
use crate::{color::*, geom::*, transform::*};
use glam::Mat4;
use std::fmt::{Debug, Formatter};
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use wgpu::util::DeviceExt;
use wgpu::{vertex_attr_array, VertexAttribute, VertexBufferLayout, VertexStepMode};

#[derive(Copy, Clone)]
pub struct RenderData<V, I> {
    pub pipeline: Option<PipelineRef<V, I>>,
    pub texture: TextureRef,
    pub mesh: MeshRef<V>,
}

impl<V, I: InstanceData> RenderData<V, I> {
    pub fn for_model_instance(self, model: I) -> InstanceRenderData<V, I> {
        InstanceRenderData {
            texture: Some(self.texture),
            pipeline: self.pipeline,
            mesh: self.mesh,
            model,
        }
    }
}

pub trait VertexLayout {
    fn vertex_layout() -> VertexBufferLayout<'static>;
}

pub trait InstanceData: Copy + Default + Sized + VertexLayout + bytemuck::Pod {}

impl VertexLayout for () {
    fn vertex_layout() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: 0,
            step_mode: VertexStepMode::Instance,
            attributes: &[],
        }
    }
}

impl InstanceData for () {}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelInstanceData {
    pub subtexture: Rect,
    pub tint: Color,
    pub transform: Mat4,
}

impl InstanceData for ModelInstanceData {}

impl ModelInstanceData {
    const ATTRIBUTES: [VertexAttribute; 7] = vertex_attr_array![
        // uv_scale: vec2<f32>
        0 => Float32x2,
        // uv_offset: vec2<f32>
        1 => Float32x2,
        // tint: vec4<f32>
        2 => Float32x4,
        // model_N: vec4<f32> * 4
        3 => Float32x4,
        4 => Float32x4,
        5 => Float32x4,
        6 => Float32x4,
    ];
}

impl ModelInstanceData {
    #[inline]
    pub fn transform(transform: impl Transform) -> Self {
        Self {
            transform: transform.as_mat4(),
            subtexture: Rect::default(),
            tint: Color::WHITE,
        }
    }
}

impl VertexLayout for ModelInstanceData {
    fn vertex_layout() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: VertexStepMode::Instance,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

impl Default for ModelInstanceData {
    fn default() -> Self {
        Self {
            transform: Default::default(),
            tint: Color::WHITE,
            subtexture: Rect::new(0., 0., 1., 1.),
        }
    }
}

#[rustfmt::skip]
pub const DEFAULT_TEXTURE_DATA: [u8; 16] = [
    // 255, 0, 255, 255,
    255, 255, 255, 255,
    255, 255, 255, 255,
    255, 255, 255, 255,
    255, 255, 255, 255,
    // 255, 0, 255, 255,
];

#[derive(Debug)]
pub struct OffscreenFramebuffer {
    pub color: TextureRef,
    pub depth: Option<TextureRef>,
    pub(super) size: Point<u32>,
}

pub trait VertexLayouts {
    fn vertex_layouts() -> Vec<VertexBufferLayout<'static>>;
}

impl<V: VertexLayout> VertexLayouts for V {
    fn vertex_layouts() -> Vec<VertexBufferLayout<'static>> {
        vec![V::vertex_layout()]
    }
}

macro_rules! impl_vertex_layouts_tuple {
    ($($V:ident),*) => {
        impl<$($V: VertexLayout),*> VertexLayouts for ($($V),*) {
            fn vertex_layouts() -> Vec<VertexBufferLayout<'static>> {
                vec![$($V::vertex_layout()),*]
            }
        }
    };
}

impl_vertex_layouts_tuple!(V1, V2);
impl_vertex_layouts_tuple!(V1, V2, V3);
impl_vertex_layouts_tuple!(V1, V2, V3, V4);
impl_vertex_layouts_tuple!(V1, V2, V3, V4, V5);
impl_vertex_layouts_tuple!(V1, V2, V3, V4, V5, V6);

pub trait Bindable {
    fn bind_group(&self, device: &wgpu::Device, layout: &wgpu::BindGroupLayout) -> wgpu::BindGroup;
}

pub struct BindGroup<T: Bindable> {
    resource: T,
    bind_group: Arc<wgpu::BindGroup>,
}

impl<T: Bindable + Debug> Debug for BindGroup<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BindGroup")
            .field("resource", &self.resource)
            .field("bind_group", &self.bind_group)
            .finish()
    }
}

impl<T: Bindable> BindGroup<T> {
    pub fn new(device: &wgpu::Device, layout: &wgpu::BindGroupLayout, resource: T) -> Self {
        let bind_group = Arc::new(resource.bind_group(device, layout));
        Self {
            resource,
            bind_group,
        }
    }

    pub fn bind_group(&self) -> &Arc<wgpu::BindGroup> {
        &self.bind_group
    }
}

impl<T: Bindable> Deref for BindGroup<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.resource
    }
}

impl<T: Bindable> DerefMut for BindGroup<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.resource
    }
}

pub trait UniformData: bytemuck::Pod + bytemuck::Zeroable {}

pub struct UniformBuffer<U: UniformData> {
    uniform: U,
    buffer: wgpu::Buffer,
}

impl<U: UniformData> UniformBuffer<U> {
    pub fn new(device: &wgpu::Device, uniform: U) -> Self {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        Self { uniform, buffer }
    }

    pub fn uniform(&self) -> &U {
        &self.uniform
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    pub fn update_with(&mut self, queue: &wgpu::Queue, modifier: impl FnOnce(&mut U)) {
        modifier(&mut self.uniform);
        queue.write_buffer(&self.buffer, 0, bytemuck::bytes_of(&self.uniform));
    }

    pub fn update(&mut self, queue: &wgpu::Queue, uniform: U) {
        self.update_with(queue, |u| *u = uniform);
    }

    pub fn bind_group_layout(
        device: &wgpu::Device,
        name: impl AsRef<str>,
        visibility: wgpu::ShaderStages,
    ) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some(name.as_ref()),
        })
    }
}

impl<U: UniformData> Bindable for UniformBuffer<U> {
    fn bind_group(&self, device: &wgpu::Device, layout: &wgpu::BindGroupLayout) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("uniform_bind_group"),
            layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: self.buffer.as_entire_binding(),
            }],
        })
    }
}
