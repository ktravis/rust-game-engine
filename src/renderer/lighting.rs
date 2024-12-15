use std::sync::Arc;

use glam::{Mat4, Vec3, Vec4, Vec4Swizzles};
use wgpu::include_wgsl;

use crate::{
    geom::{ModelVertexData, Point},
    transform::{Transform, Transform3D},
};

use super::{
    instance::InstanceRenderData, state::ViewProjectionUniforms, Display,
    InstanceDataWithNormalMatrix, PipelineRef, RenderState, RenderTarget, Texture, TextureBuilder,
    UniformBuffer, UniformData,
};

pub const MAX_LIGHTS: usize = 8;

#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
struct LightRaw {
    position: Vec4,
    color: Vec4,
    view_proj: Mat4,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Light {
    pub position: Vec4,
    pub color: Vec4,
    pub view: Mat4,
    pub proj: Mat4,
}

impl From<Light> for LightRaw {
    fn from(value: Light) -> Self {
        LightRaw {
            position: value.position,
            color: value.color,
            view_proj: value.proj * value.view,
        }
    }
}

#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct LightingUniformsRaw {
    items: [LightRaw; MAX_LIGHTS],
    count: u32,
    shadow_bias_minimum: f32,
    shadow_bias_factor: f32,
    shadow_blur_half_kernel_size: i32,
}

#[derive(Clone, Debug)]
pub struct LightingUniforms {
    lights: Vec<Light>,
    shadow_bias_minimum: f32,
    shadow_bias_factor: f32,
    shadow_blur_half_kernel_size: i32,
}

impl Default for LightingUniforms {
    fn default() -> Self {
        Self {
            lights: vec![],
            shadow_bias_minimum: 0.005,
            shadow_bias_factor: 0.005,
            shadow_blur_half_kernel_size: 3,
        }
    }
}

impl UniformData for LightingUniforms {
    type Raw = LightingUniformsRaw;

    fn raw(&self) -> Self::Raw {
        LightingUniformsRaw {
            items: std::array::from_fn(|i| {
                if i < self.lights.len() {
                    self.lights[i].into()
                } else {
                    Default::default()
                }
            }),
            count: self.lights.len() as _,
            shadow_bias_minimum: self.shadow_bias_minimum,
            shadow_bias_factor: self.shadow_bias_factor,
            shadow_blur_half_kernel_size: self.shadow_blur_half_kernel_size,
        }
    }
}

pub struct ShadowMappingPass {
    shadow_map_pipeline: PipelineRef<ModelVertexData, InstanceDataWithNormalMatrix>,
    shadow_map: Texture,
    bind_group: Arc<wgpu::BindGroup>,
    bind_group_layout: wgpu::BindGroupLayout,
    shadow_map_target_views: [wgpu::TextureView; MAX_LIGHTS],
    lights_uniform: UniformBuffer<LightingUniforms>,
}

impl ShadowMappingPass {
    pub fn new(state: &mut RenderState, display: &Display) -> Self {
        let shadow_map_pipeline = state
            .pipeline_builder()
            .with_label("Shadow Map Render Pipeline")
            .with_color_target_states(vec![])
            .build(
                display.device(),
                &display
                    .device()
                    .create_shader_module(include_wgsl!("../../res/shaders/shadow_map.wgsl")),
            );
        let lights_uniform = UniformBuffer::new(display.device(), LightingUniforms::default());
        let bind_group_layout =
            display
                .device()
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D2Array,
                                sample_type: wgpu::TextureSampleType::Depth,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                            count: None,
                        },
                    ],
                    label: Some("lighting bind group layout"),
                });
        let shadow_map = TextureBuilder::depth()
            .with_layers(MAX_LIGHTS as u32)
            .with_address_mode(wgpu::AddressMode::ClampToBorder)
            .with_border_color(wgpu::SamplerBorderColor::OpaqueWhite)
            .with_usage(
                wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            )
            .build(display.device(), Point::new(1024, 1024));
        let bind_group = display
            .device()
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("lighting bind group"),
                layout: &bind_group_layout,
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
            })
            .into();
        let shadow_map_target_views = std::array::from_fn(|i| {
            shadow_map
                .texture
                .create_view(&wgpu::TextureViewDescriptor {
                    label: Some("shadow"),
                    format: None,
                    dimension: Some(wgpu::TextureViewDimension::D2),
                    aspect: wgpu::TextureAspect::All,
                    base_mip_level: 0,
                    mip_level_count: None,
                    base_array_layer: i as u32,
                    array_layer_count: Some(1),
                })
        });
        Self {
            shadow_map_pipeline,
            shadow_map,
            bind_group,
            bind_group_layout,
            shadow_map_target_views,
            lights_uniform,
        }
    }

    pub fn bind_group(&self) -> &Arc<wgpu::BindGroup> {
        &self.bind_group
    }

    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    pub fn shadow_map_texture(&self) -> &Texture {
        &self.shadow_map
    }

    pub fn run(
        &mut self,
        state: &mut RenderState,
        display: &Display,
        lights: &[Light],
        scene: &[InstanceRenderData<ModelVertexData, InstanceDataWithNormalMatrix>],
    ) {
        self.lights_uniform.update(
            display.queue(),
            LightingUniforms {
                lights: lights.to_vec(),
                ..Default::default()
            },
        );

        let mut bufs = vec![];
        for (i, light) in lights.iter().enumerate() {
            let buf = state
                .render_pass(
                    &display,
                    "Shadow Pass",
                    &[],
                    Some(RenderTarget::TextureView(&self.shadow_map_target_views[i])),
                    &ViewProjectionUniforms {
                        view: light.view,
                        projection: light.proj,
                        pos: light.position.xyz(),
                        inverse_view: light.view.inverse(),
                    },
                    |r| {
                        for render_data in scene {
                            r.draw_instance(&InstanceRenderData {
                                pipeline: Some(self.shadow_map_pipeline),
                                ..*render_data
                            });
                        }
                    },
                )
                .command_buffer();
            bufs.push(buf);
        }
        display.queue().submit(bufs);
    }
}
