use super::instance::InstanceRenderData;
use super::texture::{Texture, TextureBuilder};
use super::{MeshRef, PipelineRef, TextureRef};
use crate::{color::*, geom::*, transform::*};
use glam::Mat4;
use std::borrow::Borrow;
use std::fmt::{Debug, Formatter};
use std::ops::{Deref, DerefMut};
use wgpu::util::DeviceExt;
use wgpu::{vertex_attr_array, VertexAttribute, VertexBufferLayout, VertexStepMode};

#[derive(Copy, Clone)]
pub struct RenderData {
    pub pipeline: PipelineRef,
    pub texture: TextureRef,
    pub mesh: MeshRef,
}

impl RenderData {
    pub fn for_model_instance(self, model: ModelInstanceData) -> InstanceRenderData {
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

pub trait InstanceData: Copy + Default + Sized + VertexLayout {}

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

#[derive(Debug, Clone, Copy)]
pub struct ModelInstanceData {
    pub subtexture: Rect,
    pub tint: Color,
    pub transform: Transform,
}

impl ModelInstanceData {
    const ATTRIBUTES: [VertexAttribute; 7] = vertex_attr_array![
        // uv_scale: vec2<f32>
        2 => Float32x2,
        // uv_offset: vec2<f32>
        3 => Float32x2,
        // tint: vec4<f32>
        4 => Float32x4,
        // model_N: vec4<f32> * 4
        5 => Float32x4,
        6 => Float32x4,
        7 => Float32x4,
        8 => Float32x4,
    ];

    #[inline]
    pub fn as_raw(&self) -> RawInstanceData {
        let model = self.transform.as_matrix().to_cols_array_2d();
        RawInstanceData {
            uv_scale: self.subtexture.dim.into(),
            uv_offset: self.subtexture.pos.into(),
            tint: [self.tint.r, self.tint.g, self.tint.b, self.tint.a],
            model,
        }
    }
}

impl VertexLayout for ModelInstanceData {
    fn vertex_layout() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<RawInstanceData>() as wgpu::BufferAddress,
            step_mode: VertexStepMode::Instance,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RawInstanceData {
    uv_scale: [f32; 2],
    uv_offset: [f32; 2],
    tint: [f32; 4],
    model: [[f32; 4]; 4],
}

impl From<ModelInstanceData> for RawInstanceData {
    fn from(other: ModelInstanceData) -> Self {
        other.as_raw()
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

impl Default for RawInstanceData {
    fn default() -> Self {
        Self {
            uv_scale: Default::default(),
            uv_offset: Default::default(),
            tint: [1.0, 1.0, 1.0, 1.0],
            model: Mat4::IDENTITY.to_cols_array_2d(),
        }
    }
}

pub trait RenderTarget {
    fn size_pixels(&self) -> Point<u32>;

    fn color_attachment(
        &self,
        load_op: wgpu::LoadOp<wgpu::Color>,
    ) -> wgpu::RenderPassColorAttachment;

    fn depth_stencil_attachment(
        &self,
        _depth_load_op: wgpu::LoadOp<f32>,
        _stencil_ops: impl Into<Option<wgpu::Operations<u32>>>,
    ) -> Option<wgpu::RenderPassDepthStencilAttachment> {
        None
    }
}

#[rustfmt::skip]
pub const DEFAULT_TEXTURE_DATA: [u8; 16] = [
    255, 0, 255, 255,
    255, 255, 255, 255,
    255, 255, 255, 255,
    255, 0, 255, 255,
];

#[derive(Debug)]
pub struct OffscreenFramebuffer {
    pub color: BindGroup<Texture>,
    pub depth: Option<Texture>,
}

impl OffscreenFramebuffer {
    pub fn new(
        device: &wgpu::Device,
        bind_group_layout: &wgpu::BindGroupLayout,
        size: Point<u32>,
    ) -> Self {
        let color = TextureBuilder::render_target()
            .with_label("offscreen_color_target")
            .build(device, size);
        let depth = TextureBuilder::depth()
            .with_label("offscreen_depth_target")
            .build(device, size);
        Self {
            color: BindGroup::new(device, bind_group_layout, color),
            depth: Some(depth),
        }
    }
}

impl<T> RenderTarget for T
where
    T: Borrow<OffscreenFramebuffer>,
{
    fn size_pixels(&self) -> Point<u32> {
        self.borrow().color.size_pixels()
    }

    fn color_attachment(
        &self,
        load_op: wgpu::LoadOp<wgpu::Color>,
    ) -> wgpu::RenderPassColorAttachment {
        wgpu::RenderPassColorAttachment {
            view: &self.borrow().color.view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: load_op,
                store: wgpu::StoreOp::Store,
            },
        }
    }

    fn depth_stencil_attachment(
        &self,
        depth_load_op: wgpu::LoadOp<f32>,
        stencil_ops: impl Into<Option<wgpu::Operations<u32>>>,
    ) -> Option<wgpu::RenderPassDepthStencilAttachment> {
        self.borrow()
            .depth
            .as_ref()
            .map(|tex| wgpu::RenderPassDepthStencilAttachment {
                view: &tex.view,
                depth_ops: Some(wgpu::Operations {
                    load: depth_load_op,
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: stencil_ops.into(),
            })
    }
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
    bind_group: wgpu::BindGroup,
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
        let bind_group = resource.bind_group(device, layout);
        Self {
            resource,
            bind_group,
        }
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
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

pub trait UniformData {
    type Raw: bytemuck::Pod + bytemuck::Zeroable;
    fn as_raw(&self) -> Self::Raw;
}

pub struct UniformBuffer<U: UniformData> {
    uniform: U,
    buffer: wgpu::Buffer,
}

impl<U: UniformData> UniformBuffer<U> {
    pub fn new(device: &wgpu::Device, uniform: U) -> Self {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniform.as_raw()]),
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
        queue.write_buffer(&self.buffer, 0, bytemuck::bytes_of(&self.uniform.as_raw()));
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