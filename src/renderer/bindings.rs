use std::{
    fmt::{Debug, Formatter},
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use bytemuck::Zeroable;
use glam::{Mat4, Vec3, Vec4};
use shadertype_derive::shader_uniform_type;
use wgpu::{TextureFormat, TextureSampleType};

use crate::{
    camera::Camera,
    renderer::{shader_type::ShaderUniformType, Texture},
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

pub const UNIFORM_BGL_ENTRY: wgpu::BindGroupLayoutEntry = wgpu::BindGroupLayoutEntry {
    binding: 0,
    visibility: wgpu::ShaderStages::all(),
    ty: wgpu::BindingType::Buffer {
        ty: wgpu::BufferBindingType::Uniform,
        has_dynamic_offset: false,
        min_binding_size: None,
    },
    count: None,
};

pub fn create_uniform_bind_group<U: UniformData>(
    device: &wgpu::Device,
    uniform: U,
) -> UniformBindGroup<U> {
    let resource = UniformBuffer::new(device, uniform);
    <BindGroup<UniformBuffer<U>>>::new(device, resource)
}

#[shader_uniform_type]
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

pub struct TextureView<S = f32, const D: u8 = 2, const F: bool = true> {
    inner: wgpu::TextureView,
    _marker: PhantomData<S>,
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
                    sample_type: TextureSampleType::Float { filterable: F },
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
                sample_type: TextureSampleType::Uint,
            },
            count: None,
        }]
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
                sample_type: TextureSampleType::Sint,
            },
            count: None,
        }]
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
                sample_type: TextureSampleType::Depth,
            },
            count: None,
        }]
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
                sample_type: TextureSampleType::Depth,
            },
            count: None,
        }]
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

impl<const F: bool> TextureSampler<F> {
    pub fn raw(&self) -> &wgpu::Sampler {
        &self.inner
    }
}

pub type FilteringSampler = TextureSampler<true>;
pub type NonFilteringSampler = TextureSampler<false>;

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
            ty: wgpu::BindingType::Sampler(if F {
                wgpu::SamplerBindingType::Filtering
            } else {
                wgpu::SamplerBindingType::NonFiltering
            }),
            count: None,
        }]
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
}

impl From<wgpu::Sampler> for ComparisonSampler {
    fn from(inner: wgpu::Sampler) -> Self {
        Self { inner }
    }
}

pub fn texture_bgl_entries(format: TextureFormat) -> Vec<wgpu::BindGroupLayoutEntry> {
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

pub trait UniformData {
    type Raw: bytemuck::Pod + bytemuck::Zeroable;
    fn raw(&self) -> Self::Raw;
}

impl<U> UniformData for U
where
    U: ShaderUniformType,
{
    type Raw = <Self as ShaderUniformType>::Raw;
    fn raw(&self) -> Self::Raw {
        *<Self as ShaderUniformType>::raw(self)
    }
}

impl ShaderUniformType for Mat4 {
    type Raw = Self;

    fn definition() -> String {
        "mat4x4<f32>".to_string()
    }

    fn raw<'a>(&'a self) -> &'a Self::Raw {
        self
    }
}

// TODO: make this into a proc macro etc
#[macro_export]
macro_rules! define_bind_group {
    ($sv:vis $name:ident {
        $($fv:vis $f:ident: $t:ty,)*
    }) => (
        $sv struct $name {
            $($fv $f: $t,)*
        }

        impl Bindable for $name {
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

pub struct UniformBuffer<U: UniformData> {
    uniform: U,
    buffer: wgpu::Buffer,
}

impl<U: UniformData> Deref for UniformBuffer<U> {
    type Target = U;

    fn deref(&self) -> &Self::Target {
        &self.uniform
    }
}

impl<U: UniformData> DerefMut for UniformBuffer<U> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.uniform
    }
}

impl<U: UniformData> UniformBuffer<U> {
    pub fn new(device: &wgpu::Device, uniform: U) -> Self {
        let n = std::mem::size_of::<U::Raw>();
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&format!("Uniform Buffer ({})", std::any::type_name::<U>())),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            // pad to 16 bytes, which will be required in the shader
            size: n.next_multiple_of(16) as u64,
            mapped_at_creation: false,
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

impl<U: UniformData> Bindable for UniformBuffer<U> {
    fn entries(&self) -> Vec<wgpu::BindGroupEntry<'_>> {
        vec![wgpu::BindGroupEntry {
            binding: 0,
            resource: self.buffer.as_entire_binding(),
        }]
    }

    fn layout_entries() -> Vec<wgpu::BindGroupLayoutEntry> {
        vec![UNIFORM_BGL_ENTRY]
    }
}

pub type UniformBindGroup<T> = BindGroup<UniformBuffer<T>>;

/*
    @group(3) @binding(0)
    var<uniform> lights: LightingUniformsRaw;
    @group(3) @binding(1)
    var shadow_map: texture_depth_2d_array;
    @group(3) @binding(2)
    var shadow_map_sampler: sampler_comparison;

    &wgpu::BindGroupLayoutDescriptor {
        label: Some("group3"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0u32,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1u32,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2Array,
                    sample_type: wgpu::TextureSampleType::Depth,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2u32,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                count: None,
            },
        ],
    }

    &wgpu::BindGroupDescriptor {
        label: Some("lighting bind group"),
        layout: &lights_uniform_bgl,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: lights_uniform.buffer().as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&shadow_map.view),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Sampler(&shadow_map.sampler),
            },
        ],
    }
*/
