use std::{
    fmt::{Debug, Formatter},
    marker::PhantomData,
    num::NonZeroU32,
    ops::{Deref, DerefMut},
};

use bytemuck::Zeroable;
use glam::{Mat4, Vec3, Vec4};
use shadertype_derive::shader_uniform_data;
use wgpu::util::DeviceExt;

use crate::{
    camera::Camera,
    renderer::{
        shader_type::{ShaderBuiltin, ShaderDefinedType, ShaderNamedType, ShaderUniformData},
        Texture,
    },
};

// TODO: pipeline layout is the thing that needs to stay the same between draw calls to different
// pipelines in the same render pass
// TODO: eliminate pipeline builder and just do something like PipelineDescriptor{ field1:
// Some(blah), ..pipeline::basic_descriptor() }
//
// [ ] make definition() recursive
// [x] move "builtin" bind groups out of renderer
// [x] define bindgroup as a generic type wrapping something "bindable"
// [ ] enumerate all bindable types
// [ ] pass.draw_instanced(mesh, commonOptions) -> batcher
// [ ] when drawing, each pipeline/pass should accept anything that can be `Into` its required data
// type (instance data etc)

pub fn create_uniform_bind_group<U: ShaderNamedType + ShaderUniformData>(
    device: &wgpu::Device,
    uniform: U,
) -> UniformBindGroup<U> {
    let resource = UniformBuffer::new(device, uniform);
    BindGroup::<UniformBuffer<U>>::new(device, resource)
}

#[derive(Copy, Clone, Debug, bytemuck::Zeroable, bytemuck::Pod)]
#[shader_uniform_data]
#[repr(C)]
pub struct ViewProjectionUniforms {
    pub view: Mat4,
    pub projection: Mat4,
    pub camera_pos: Vec3,
    #[skip]
    pub _pad_camera_pos: [u8; 4u32 as usize],
    pub inverse_view: Mat4,
}

impl ViewProjectionUniforms {
    pub fn for_camera(camera: &Camera) -> Self {
        let view = camera.view_matrix();
        assert!(
            (view.inverse() * view * Vec4::ONE - Vec4::ONE)
                .abs()
                .length_squared()
                < 0.000001
        );
        Self {
            view,
            inverse_view: view.inverse(),
            projection: camera.perspective_matrix(),
            camera_pos: camera.position(),
            ..Default::default()
        }
    }
}

impl Default for ViewProjectionUniforms {
    fn default() -> Self {
        Self {
            view: Default::default(),
            projection: Default::default(),
            camera_pos: Default::default(),
            inverse_view: Default::default(),
            ..Zeroable::zeroed()
        }
    }
}

fn binding_annotations(group_index: u32, binding_offset: &mut u32) -> String {
    let s = format!("@group({group_index}) @binding({})", *binding_offset);
    *binding_offset += 1;
    s
}

pub struct TextureView<S = f32, const D: u8 = 2, const F: bool = true> {
    inner: wgpu::TextureView,
    _marker: PhantomData<S>,
}

impl<S: ShaderNamedType, const D: u8, const F: bool> ShaderBuiltin for TextureView<S, D, F> {}

impl<S: ShaderNamedType, const D: u8, const F: bool> ShaderNamedType for TextureView<S, D, F> {
    fn wgsl_name() -> String {
        format!(
            "texture_{}d<{}>",
            match D {
                1 | 2 | 3 => D,
                _ => panic!("invalid texture view dimension: {}", D),
            },
            S::wgsl_name(),
        )
    }
}

impl<S, const D: u8, const F: bool> TextureView<S, D, F> {
    pub fn raw(&self) -> &wgpu::TextureView {
        &self.inner
    }
}

impl<const D: u8, const F: bool> Bindable for TextureView<f32, D, F> {
    fn entries(&self) -> Vec<wgpu::BindGroupEntry<'_>> {
        vec![wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::TextureView(&self.inner),
        }]
    }

    fn layout_entries() -> Vec<wgpu::BindGroupLayoutEntry> {
        let view_dimension = match D {
            1 => wgpu::TextureViewDimension::D1,
            2 => wgpu::TextureViewDimension::D2,
            3 => wgpu::TextureViewDimension::D3,
            _ => panic!("invalid texture view dimension: {}", D),
        };
        vec![
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension,
                    sample_type: wgpu::TextureSampleType::Float { filterable: F },
                },
                count: None,
            },
            // wgpu::BindGroupLayoutEntry {
            //     binding: 1,
            //     visibility: wgpu::ShaderStages::all(),
            //     // TODO: technically need to allow non-filtering sampler even if F here
            //     ty: wgpu::BindingType::Sampler(if F {
            //         wgpu::SamplerBindingType::Filtering
            //     } else {
            //         wgpu::SamplerBindingType::NonFiltering
            //     }),
            //     count: None,
            // },
        ]
    }

    fn shader_bindings(name: &str, group_index: u32, binding_offset: &mut u32) -> String {
        format!(
            "{} var {}: {};",
            binding_annotations(group_index, binding_offset),
            name,
            <Self as ShaderNamedType>::wgsl_name()
        )
    }
}

impl<const D: u8, const F: bool> Bindable for TextureView<u32, D, F> {
    fn entries(&self) -> Vec<wgpu::BindGroupEntry<'_>> {
        vec![wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::TextureView(&self.inner),
        }]
    }

    fn layout_entries() -> Vec<wgpu::BindGroupLayoutEntry> {
        let view_dimension = match D {
            1 => wgpu::TextureViewDimension::D1,
            2 => wgpu::TextureViewDimension::D2,
            3 => wgpu::TextureViewDimension::D3,
            _ => panic!("invalid texture view dimension: {}", D),
        };
        assert!(!F, "Uint texture sample type is not filterable");
        vec![wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                multisampled: false,
                view_dimension,
                sample_type: wgpu::TextureSampleType::Uint,
            },
            count: None,
        }]
    }

    fn shader_bindings(name: &str, group_index: u32, binding_offset: &mut u32) -> String {
        format!(
            "{} var {}: {};",
            binding_annotations(group_index, binding_offset),
            name,
            <Self as ShaderNamedType>::wgsl_name()
        )
    }
}

impl<const D: u8, const F: bool> Bindable for TextureView<i32, D, F> {
    fn entries(&self) -> Vec<wgpu::BindGroupEntry<'_>> {
        vec![wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::TextureView(&self.inner),
        }]
    }

    fn layout_entries() -> Vec<wgpu::BindGroupLayoutEntry> {
        let view_dimension = match D {
            1 => wgpu::TextureViewDimension::D1,
            2 => wgpu::TextureViewDimension::D2,
            3 => wgpu::TextureViewDimension::D3,
            _ => panic!("invalid texture view dimension: {}", D),
        };
        assert!(!F, "Sint texture sample type is not filterable");
        vec![wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                multisampled: false,
                view_dimension,
                sample_type: wgpu::TextureSampleType::Sint,
            },
            count: None,
        }]
    }

    fn shader_bindings(name: &str, group_index: u32, binding_offset: &mut u32) -> String {
        format!(
            "{} var {}: {};",
            binding_annotations(group_index, binding_offset),
            name,
            <Self as ShaderNamedType>::wgsl_name()
        )
    }
}

impl<S, const D: u8, const F: bool> From<wgpu::TextureView> for TextureView<S, D, F> {
    fn from(inner: wgpu::TextureView) -> Self {
        Self {
            inner,
            _marker: PhantomData,
        }
    }
}

#[derive(Clone)]
pub struct DepthTextureView<const D: u8 = 2> {
    inner: wgpu::TextureView,
}

impl<const D: u8> ShaderBuiltin for DepthTextureView<D> {}

impl<const D: u8> ShaderNamedType for DepthTextureView<D> {
    fn wgsl_name() -> String {
        format!(
            "texture_depth_{}d",
            match D {
                1 | 2 | 3 => D,
                _ => panic!("invalid texture view dimension: {}", D),
            },
        )
    }
}

impl<const D: u8> Bindable for DepthTextureView<D> {
    fn entries(&self) -> Vec<wgpu::BindGroupEntry<'_>> {
        vec![wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::TextureView(&self.inner),
        }]
    }

    fn layout_entries() -> Vec<wgpu::BindGroupLayoutEntry> {
        let view_dimension = match D {
            1 => wgpu::TextureViewDimension::D1,
            2 => wgpu::TextureViewDimension::D2,
            3 => wgpu::TextureViewDimension::D3,
            _ => panic!("invalid texture view dimension: {}", D),
        };
        vec![wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                multisampled: false,
                view_dimension,
                sample_type: wgpu::TextureSampleType::Depth,
            },
            count: None,
        }]
    }

    fn shader_bindings(name: &str, group_index: u32, binding_offset: &mut u32) -> String {
        format!(
            "{} var {}: {};",
            binding_annotations(group_index, binding_offset),
            name,
            <Self as ShaderNamedType>::wgsl_name()
        )
    }
}

impl<const D: u8> DepthTextureView<D> {
    pub fn raw(&self) -> &wgpu::TextureView {
        &self.inner
    }
}

impl<const D: u8> From<wgpu::TextureView> for DepthTextureView<D> {
    fn from(inner: wgpu::TextureView) -> Self {
        debug_assert!(inner.texture().format().has_depth_aspect());
        Self { inner }
    }
}

#[derive(Clone)]
pub struct DepthTextureArrayView {
    inner: wgpu::TextureView,
}

impl ShaderNamedType for DepthTextureArrayView {
    fn wgsl_name() -> String {
        "texture_depth_2d_array".into()
    }
}

impl ShaderBuiltin for DepthTextureArrayView {}

impl Bindable for DepthTextureArrayView {
    fn entries(&self) -> Vec<wgpu::BindGroupEntry<'_>> {
        vec![wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::TextureView(&self.inner),
        }]
    }

    fn layout_entries() -> Vec<wgpu::BindGroupLayoutEntry> {
        vec![wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                multisampled: false,
                view_dimension: wgpu::TextureViewDimension::D2Array,
                sample_type: wgpu::TextureSampleType::Depth,
            },
            count: None,
        }]
    }

    fn shader_bindings(name: &str, group_index: u32, binding_offset: &mut u32) -> String {
        format!(
            "{} var {}: {};",
            binding_annotations(group_index, binding_offset),
            name,
            <Self as ShaderNamedType>::wgsl_name()
        )
    }
}

impl DepthTextureArrayView {
    pub fn raw(&self) -> &wgpu::TextureView {
        &self.inner
    }
}

impl From<wgpu::TextureView> for DepthTextureArrayView {
    fn from(inner: wgpu::TextureView) -> Self {
        debug_assert!(inner.texture().format().has_depth_aspect());
        Self { inner }
    }
}

#[derive(Clone)]
pub struct TextureSampler<const F: bool = true> {
    inner: wgpu::Sampler,
}

impl<const F: bool> ShaderBuiltin for TextureSampler<F> {}

impl<const F: bool> ShaderNamedType for TextureSampler<F> {
    fn wgsl_name() -> String {
        "sampler".into()
    }
}

impl<const F: bool> TextureSampler<F> {
    pub fn raw(&self) -> &wgpu::Sampler {
        &self.inner
    }
}

pub type FilteringSampler = TextureSampler<true>;
pub type NonFilteringSampler = TextureSampler<false>;

pub const fn non_comparison_sampler(filtering: bool) -> wgpu::SamplerBindingType {
    if filtering {
        wgpu::SamplerBindingType::Filtering
    } else {
        wgpu::SamplerBindingType::NonFiltering
    }
}

impl<const F: bool> Bindable for TextureSampler<F> {
    fn entries(&self) -> Vec<wgpu::BindGroupEntry<'_>> {
        vec![wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::Sampler(&self.inner),
        }]
    }

    fn layout_entries() -> Vec<wgpu::BindGroupLayoutEntry> {
        vec![wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(non_comparison_sampler(F)),
            count: None,
        }]
    }

    fn shader_bindings(name: &str, group_index: u32, binding_offset: &mut u32) -> String {
        format!(
            "{} var {}: {};",
            binding_annotations(group_index, binding_offset),
            name,
            <Self as ShaderNamedType>::wgsl_name()
        )
    }
}

impl<const F: bool> From<wgpu::Sampler> for TextureSampler<F> {
    fn from(inner: wgpu::Sampler) -> Self {
        Self { inner }
    }
}

#[derive(Clone)]
pub struct ComparisonSampler {
    inner: wgpu::Sampler,
}

impl ShaderBuiltin for ComparisonSampler {}

impl ShaderNamedType for ComparisonSampler {
    fn wgsl_name() -> String {
        "sampler_comparison".into()
    }
}

impl Bindable for ComparisonSampler {
    fn entries(&self) -> Vec<wgpu::BindGroupEntry<'_>> {
        vec![wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::Sampler(&self.inner),
        }]
    }

    fn layout_entries() -> Vec<wgpu::BindGroupLayoutEntry> {
        vec![wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
            count: None,
        }]
    }

    fn shader_bindings(name: &str, group_index: u32, binding_offset: &mut u32) -> String {
        format!(
            "{} var {}: {};",
            binding_annotations(group_index, binding_offset),
            name,
            <Self as ShaderNamedType>::wgsl_name()
        )
    }
}

impl From<wgpu::Sampler> for ComparisonSampler {
    fn from(inner: wgpu::Sampler) -> Self {
        Self { inner }
    }
}

pub fn texture_bgl_entries(format: wgpu::TextureFormat) -> Vec<wgpu::BindGroupLayoutEntry> {
    let sample_type = format
        .sample_type(Some(wgpu::TextureAspect::All), None)
        .expect(&format!(
            "non-sampleable texture format {:?} used in binding",
            format
        ));
    vec![
        wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                multisampled: false,
                view_dimension: wgpu::TextureViewDimension::D2,
                sample_type,
            },
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: 1,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(if format.has_depth_aspect() {
                wgpu::SamplerBindingType::Comparison
            } else {
                match sample_type {
                    wgpu::TextureSampleType::Float { filterable: false } => {
                        wgpu::SamplerBindingType::NonFiltering
                    }
                    _ => wgpu::SamplerBindingType::Filtering,
                }
            }),
            count: None,
        },
    ]
}

pub trait Bindable /*: Sized*/ {
    fn entries(&self) -> Vec<wgpu::BindGroupEntry<'_>>;

    fn layout_entries() -> Vec<wgpu::BindGroupLayoutEntry>;

    fn create_layout(device: &wgpu::Device, name: &'static str) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(name),
            entries: &Self::layout_entries(),
        })
    }

    fn shader_bindings<'a>(name: &'a str, group_index: u32, binding_offset: &mut u32) -> String;

    fn wgsl_var_suffix() -> String {
        "".into()
    }

    // this requires the Sized bound, do we care?
    // fn into_bind_group(self, device: &wgpu::Device, name: &'static str) -> BindGroup<Self> {
    //     let bgl = self.create_layout(device, name);
    //     BindGroup::new(device, &bgl, self)
    // }
}

pub struct BindGroup<T> {
    pub resource: T,
    bind_group: wgpu::BindGroup,
    layout: wgpu::BindGroupLayout,
}

impl<T> BindGroup<T> {
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
    pub fn layout(&self) -> &wgpu::BindGroupLayout {
        &self.layout
    }
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
    pub fn new(device: &wgpu::Device, resource: T) -> Self {
        let layout = T::create_layout(device, std::any::type_name::<T>());
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("bind group ({})", std::any::type_name::<T>())),
            layout: &layout,
            entries: &resource.entries(),
        });
        Self {
            resource,
            bind_group,
            layout,
        }
    }

    pub fn with_layout(device: &wgpu::Device, layout: wgpu::BindGroupLayout, resource: T) -> Self {
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("bind group ({})", std::any::type_name::<T>())),
            layout: &layout,
            entries: &resource.entries(),
        });
        Self {
            resource,
            bind_group,
            layout,
        }
    }
}

impl BindGroup<Texture> {
    pub fn for_texture(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        resource: Texture,
    ) -> Self {
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("bind group (texture)")),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&resource.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&resource.sampler),
                },
            ],
        });
        Self {
            resource,
            bind_group,
            layout: layout.clone(),
        }
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

// TODO: make this into a proc macro etc
#[macro_export]
macro_rules! define_bind_group {
    ($sv:vis $name:ident $(<$($generics:tt),*>)? {
        $($fv:vis $f:ident: $t:ty,)*
    }) => (
        $sv struct $name $(<$($generics),*>)? {
            $($fv $f: $t,)*
        }

        impl $(<$($generics),*>)? Bindable for $name $(<$($generics),*>)? {
            fn entries(&self) -> Vec<wgpu::BindGroupEntry<'_>> {
                let ev = [
                    $(self.$f.entries(),)*
                ];
                let mut x = vec![];
                for entries in ev.into_iter() {
                    let offset = x.len() as u32;
                    for entry in entries {
                        x.push(wgpu::BindGroupEntry{
                            binding: entry.binding + offset,
                            resource: entry.resource,
                        });
                    }
                }
                x
            }

            fn layout_entries() -> Vec<wgpu::BindGroupLayoutEntry> {
                let ev = [
                    $(<$t as Bindable>::layout_entries(),)*
                ];
                let mut x = vec![];
                for entries in ev.into_iter() {
                    let offset = x.len() as u32;
                    for entry in entries {
                        x.push(wgpu::BindGroupLayoutEntry{
                            binding: entry.binding + offset,
                            ..entry
                        });
                    }
                }
                x
            }
            fn shader_bindings(name: &str, group_index: u32, binding_offset: &mut u32) -> String {
                [
                    $({
                        <$t as ::rust_game_engine::renderer::bindings::Bindable>::shader_bindings(
                            &format!("{name}_{}", stringify!($f)),
                            group_index,
                            binding_offset,
                        )
                    },)*
                ].join("\n")
            }
        }

        impl $(<$($generics),*>)? ::rust_game_engine::renderer::shader_type::ShaderDefinedType for $name $(<$($generics),*>)? {
            fn definitions() -> Vec<String> {
                [
                    $(<$t as ::rust_game_engine::renderer::shader_type::ShaderDefinedType>::definitions() ,)*
                ].into_iter().flatten().collect::<Vec<_>>()
            }
        }
    )
}

define_bind_group! {
    pub MaterialGroup {
        pub view: TextureView,
        pub sampler: TextureSampler,
    }
}

define_bind_group! {
    pub UnfilteredMaterialGroup {
        pub view: TextureView<f32, 2, false>,
        pub sampler: TextureSampler<false>,
    }
}

define_bind_group! {
    pub DepthBuffer {
        // #[suffix("")] -> depth_bufffer
        pub view: DepthTextureView<2>,
        // -> depth_bufffer_sampler
        pub sampler: TextureSampler<true>,
    }
}

// impl<U> UniformData for U
// where
//     U: Sized + bytemuck::Pod + bytemuck::Zeroable + !ShaderUniformType,
// {
//     type Raw = Self;
//     fn raw(&self) -> Self::Raw {
//         return *self;
//     }
// }

pub struct UniformBuffer<U: ShaderNamedType + ShaderUniformData> {
    uniform: U,
    buffer: wgpu::Buffer,
}

impl<U: ShaderNamedType + ShaderUniformData + ShaderDefinedType> ShaderDefinedType
    for UniformBuffer<U>
{
    fn definitions() -> Vec<String> {
        U::definitions()
    }
}

impl<U: ShaderNamedType + ShaderUniformData> ShaderNamedType for UniformBuffer<U> {
    fn wgsl_name() -> String {
        U::wgsl_name()
    }
}

impl<U: ShaderNamedType + ShaderUniformData> Deref for UniformBuffer<U> {
    type Target = U;

    fn deref(&self) -> &Self::Target {
        &self.uniform
    }
}

impl<U: ShaderNamedType + ShaderUniformData> DerefMut for UniformBuffer<U> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.uniform
    }
}

impl<U: ShaderNamedType + ShaderUniformData> UniformBuffer<U> {
    pub fn new(device: &wgpu::Device, uniform: U) -> Self {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("Uniform Buffer ({})", std::any::type_name::<U>())),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            // pad to 16 bytes, which will be required in the shader
            contents: bytemuck::bytes_of(&uniform.raw()),
        });
        Self { uniform, buffer }
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    pub fn update_with(&mut self, queue: &wgpu::Queue, modifier: impl FnOnce(&mut U)) {
        modifier(&mut self.uniform);
        queue.write_buffer(&self.buffer, 0, bytemuck::bytes_of(&self.uniform.raw()));
    }

    pub fn update(&mut self, queue: &wgpu::Queue, uniform: U) {
        self.update_with(queue, |u| *u = uniform);
    }
}

impl<U: ShaderNamedType + ShaderUniformData> Bindable for UniformBuffer<U> {
    fn entries(&self) -> Vec<wgpu::BindGroupEntry<'_>> {
        vec![wgpu::BindGroupEntry {
            binding: 0,
            resource: self.buffer.as_entire_binding(),
        }]
    }

    fn layout_entries() -> Vec<wgpu::BindGroupLayoutEntry> {
        vec![wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::all(),
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }]
    }

    fn shader_bindings(name: &str, group_index: u32, binding_offset: &mut u32) -> String {
        format!(
            "{} var{} {}: {};",
            binding_annotations(group_index, binding_offset),
            Self::wgsl_var_suffix(),
            name,
            <U as ShaderNamedType>::wgsl_name()
        )
    }

    fn wgsl_var_suffix() -> String {
        "<uniform>".into()
    }
}

pub type UniformBindGroup<T> = BindGroup<UniformBuffer<T>>;

pub struct TextureBindingArray<'a, const N: usize, T = TextureView> {
    views: Vec<&'a wgpu::TextureView>,
    samplers: Vec<&'a wgpu::Sampler>,
    // placeholder_texture: Texture,
    // _n: [(); N],
    _marker: PhantomData<T>,
}

impl<'a, const N: usize, T: Bindable + ShaderNamedType> ShaderBuiltin
    for TextureBindingArray<'a, N, T>
{
}

impl<'a, const N: usize, T: Bindable + ShaderNamedType> TextureBindingArray<'a, N, T> {
    pub fn new(textures: &'a [Texture], placeholder_texture: &'a Texture) -> Self {
        assert!(textures.len() <= N);
        let (views, samplers): (Vec<_>, Vec<_>) = textures
            .iter()
            .chain(std::iter::repeat(placeholder_texture))
            .take(N)
            .map(|t| (&t.view, &t.sampler))
            .unzip();
        Self {
            views,
            samplers,
            // placeholder_texture,
            _marker: PhantomData,
        }
    }
}

impl<'a, const N: usize, T: Bindable + ShaderNamedType> Bindable for TextureBindingArray<'a, N, T> {
    fn entries(&self) -> Vec<wgpu::BindGroupEntry<'_>> {
        vec![
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureViewArray(&self.views),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::SamplerArray(&self.samplers),
            },
        ]
    }

    fn layout_entries() -> Vec<wgpu::BindGroupLayoutEntry> {
        vec![
            wgpu::BindGroupLayoutEntry {
                count: NonZeroU32::new(N as _),
                ..T::layout_entries()[0]
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                count: NonZeroU32::new(N as _),
                ..FilteringSampler::layout_entries()[0]
            },
        ]
    }

    fn shader_bindings(name: &str, group_index: u32, binding_offset: &mut u32) -> String {
        let views = format!("@group({group_index}) @binding({binding_offset}) var {name}_views: binding_array<{}, {N}>;", T::wgsl_name());
        *binding_offset += 1;
        let samplers = format!("@group({group_index}) @binding({binding_offset}) var {name}_samplers: binding_array<{}, {N}>;", FilteringSampler::wgsl_name());
        format!("{views}\n{samplers}")
    }
}

#[cfg(test)]
mod test {
    use std::num::NonZeroU32;

    use glam::Mat4;

    use crate::renderer::bindings::UniformBuffer;

    use super::{Bindable, TextureBindingArray};

    #[test]
    fn test_texture_binding_array_layout_entries() {
        assert_eq!(
            TextureBindingArray::<24>::layout_entries(),
            vec![
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: NonZeroU32::new(24),
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering,),
                    count: NonZeroU32::new(24),
                },
            ]
        )
    }

    #[test]
    fn test_texture_binding_array_shader_bindings() {
        const EXPECTED_BINDINGS: &str = r#"
@group(0) @binding(0) var texture_array_views: binding_array<texture_2d<f32>, 24>;
@group(0) @binding(1) var texture_array_samplers: binding_array<sampler, 24>;
        "#;

        assert_eq!(
            TextureBindingArray::<24>::shader_bindings("texture_array", 0, &mut 0).trim_ascii(),
            EXPECTED_BINDINGS.trim_ascii(),
        )
    }

    define_bind_group! {
        pub TestBindGroup<'a> {
            pub globals: UniformBuffer<Mat4>,
            pub materials: TextureBindingArray<'a, 128>,
        }
    }

    #[test]
    fn test_texture_binding_array_in_bind_group_macro() {
        const EXPECTED_BINDINGS: &str = r#"
@group(0) @binding(0) var<uniform> foo_globals: mat4x4<f32>;
@group(0) @binding(1) var foo_materials_views: binding_array<texture_2d<f32>, 128>;
@group(0) @binding(2) var foo_materials_samplers: binding_array<sampler, 128>;
        "#;

        assert_eq!(
            TestBindGroup::shader_bindings("foo", 0, &mut 0).trim_ascii(),
            EXPECTED_BINDINGS.trim_ascii(),
        )
    }
}
