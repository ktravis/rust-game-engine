use super::instance::InstanceRenderData;
use super::texture::Texture;
use super::{display, DisplayView, MeshRef, PipelineRef, RenderState, TextureRef};
use crate::{color::*, geom::*, transform::*};
use glam::Mat4;
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
    pub fn for_model_instance<T: Transform>(
        self,
        model: ModelInstanceData<T>,
    ) -> InstanceRenderData<T> {
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
pub struct ModelInstanceData<T = Transform3D> {
    pub subtexture: Rect,
    pub tint: Color,
    pub transform: T,
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
}

impl<T: Transform> ModelInstanceData<T> {
    #[inline]
    pub fn transform(transform: T) -> Self {
        Self {
            transform,
            subtexture: Rect::default(),
            tint: Color::WHITE,
        }
    }

    #[inline]
    pub fn as_raw(&self) -> RawInstanceData {
        let model = self.transform.as_mat4(); //.to_cols_array_2d();
        RawInstanceData {
            uv_scale: [self.subtexture.dim.x, self.subtexture.dim.y],
            uv_offset: [self.subtexture.pos.x, self.subtexture.pos.y],
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
    model: Mat4,
}

impl<T: Transform> From<ModelInstanceData<T>> for RawInstanceData {
    fn from(other: ModelInstanceData<T>) -> Self {
        other.as_raw()
    }
}

impl<T: Default> Default for ModelInstanceData<T> {
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
            model: Mat4::IDENTITY,
        }
    }
}

#[derive(Copy, Clone)]
pub enum RenderTarget<'a> {
    Offscreen(&'a OffscreenFramebuffer),
    Display(&'a DisplayView<'a>),
}

impl<'a> RenderTarget<'a> {
    pub fn size_pixels(self) -> Point<u32> {
        match self {
            RenderTarget::Offscreen(fb) => fb.size,
            RenderTarget::Display(display_view) => display_view.size_pixels(),
        }
    }

    pub fn color_attachment(
        self,
        state: &'a RenderState,
        load_op: wgpu::LoadOp<wgpu::Color>,
    ) -> wgpu::RenderPassColorAttachment {
        let view = match self {
            RenderTarget::Offscreen(fb) => &state.get_texture(fb.color).view,
            RenderTarget::Display(display_view) => &display_view.view,
        };
        wgpu::RenderPassColorAttachment {
            view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: load_op,
                store: wgpu::StoreOp::Store,
            },
        }
    }

    pub fn depth_stencil_attachment(
        self,
        state: &'a RenderState,
        depth_load_op: wgpu::LoadOp<f32>,
        stencil_ops: impl Into<Option<wgpu::Operations<u32>>>,
    ) -> Option<wgpu::RenderPassDepthStencilAttachment> {
        let depth = match self {
            RenderTarget::Offscreen(fb) => fb.depth,
            RenderTarget::Display(display_view) => {
                return display_view.depth_stencil_attachment(depth_load_op, stencil_ops);
            }
        };
        depth.map(|tex| wgpu::RenderPassDepthStencilAttachment {
            view: &state.get_texture(tex).view,
            depth_ops: Some(wgpu::Operations {
                load: depth_load_op,
                store: wgpu::StoreOp::Store,
            }),
            stencil_ops: stencil_ops.into(),
        })
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
    pub color: TextureRef,
    pub depth: Option<TextureRef>,
    pub(super) size: Point<u32>,
}

// impl RenderTarget for OffscreenFramebuffer {
//     fn size_pixels(&self) -> Point<u32> {
//         self.size
//     }
//
//     fn color_attachment<'state>(
//         &self,
//         state: &'state RenderState,
//         load_op: wgpu::LoadOp<wgpu::Color>,
//     ) -> wgpu::RenderPassColorAttachment<'state> {
//         wgpu::RenderPassColorAttachment {
//             view: &state.get_texture(self.color).view,
//             resolve_target: None,
//             ops: wgpu::Operations {
//                 load: load_op,
//                 store: wgpu::StoreOp::Store,
//             },
//         }
//     }
//
//     fn depth_stencil_attachment<'state>(
//         &self,
//         state: &'state RenderState,
//         depth_load_op: wgpu::LoadOp<f32>,
//         stencil_ops: impl Into<Option<wgpu::Operations<u32>>>,
//     ) -> Option<wgpu::RenderPassDepthStencilAttachment<'state>> {
//         self.depth
//             .map(|tex| wgpu::RenderPassDepthStencilAttachment {
//                 view: &state.get_texture(tex).view,
//                 depth_ops: Some(wgpu::Operations {
//                     load: depth_load_op,
//                     store: wgpu::StoreOp::Store,
//                 }),
//                 stencil_ops: stencil_ops.into(),
//             })
//     }
// }

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
