use std::sync::Arc;

use glam::{Mat4, Vec3, Vec4, Vec4Swizzles};
use wgpu::include_wgsl;

use crate::{
    camera::Frustum,
    geom::{ModelVertexData, Point},
};

use super::{
    instance::InstanceRenderData,
    lighting::{Light, LightsUniform},
    state::ViewProjectionUniforms,
    Display, InstanceDataWithNormalMatrix, PipelineRef, RenderState, RenderTarget, Texture,
    TextureBuilder, TextureRef, UniformBuffer, UniformData,
};

pub const MAX_LIGHTS: usize = 8;

pub struct ShadowMappingPass {
    shadow_map_pipeline: PipelineRef<ModelVertexData, InstanceDataWithNormalMatrix>,
    shadow_map: Texture,
    bind_group: Arc<wgpu::BindGroup>,
    bind_group_layout: wgpu::BindGroupLayout,
    shadow_map_target_views: [wgpu::TextureView; MAX_LIGHTS],
    pub shadow_map_debug_textures: [TextureRef; MAX_LIGHTS],
    pub shadow_mapping_uniform: UniformBuffer<LightsUniform>,
    pub depth_bias_state: wgpu::DepthBiasState,
    last_depth_bias_state: wgpu::DepthBiasState,
}

impl ShadowMappingPass {
    pub fn new(state: &mut RenderState, display: &Display) -> Self {
        let shadow_mapping_uniform = UniformBuffer::new(display.device(), LightsUniform::default());
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
            .with_filter_mode(wgpu::FilterMode::Linear)
            .with_usage(
                wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            )
            .build(display.device(), Point::new(2048, 2048));
        let bind_group = display
            .device()
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("lighting bind group"),
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: shadow_mapping_uniform.buffer().as_entire_binding(),
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
        let shadow_map_debug_textures = std::array::from_fn(|i| {
            state.load_texture(
                display,
                TextureBuilder::render_target()
                    .with_label(&format!("shadow map debug {}", i))
                    .build(display.device(), Point::new(2048, 2048)),
            )
        });
        Self {
            shadow_map_pipeline: Default::default(),
            shadow_map,
            bind_group,
            bind_group_layout,
            shadow_map_target_views,
            shadow_mapping_uniform,
            depth_bias_state: wgpu::DepthBiasState {
                constant: 1,
                slope_scale: 0.025,
                clamp: 1.0,
            },
            last_depth_bias_state: Default::default(),
            shadow_map_debug_textures,
        }
    }

    fn build_shadow_map_pipeline(&mut self, state: &mut RenderState, display: &Display) {
        self.shadow_map_pipeline = state
            .pipeline_builder()
            .with_label("Shadow Map Render Pipeline")
            .with_key(self.shadow_map_pipeline)
            .with_cull_mode(Some(wgpu::Face::Front))
            .with_color_target_states(vec![Some(wgpu::ColorTargetState {
                format: TextureBuilder::DEFAULT_RENDER_FORMAT,
                blend: None,
                write_mask: wgpu::ColorWrites::all(),
            })])
            .with_depth_stencil_state(Some(wgpu::DepthStencilState {
                format: TextureBuilder::DEFAULT_DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: Default::default(),
                bias: self.depth_bias_state,
            }))
            .build(
                display.device(),
                &display
                    .device()
                    .create_shader_module(include_wgsl!("../../res/shaders/shadow_map.wgsl")),
            );
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
        view_frustum: &Frustum,
        lights: &[Light],
        scene: &[InstanceRenderData<ModelVertexData, InstanceDataWithNormalMatrix>],
    ) {
        if self.depth_bias_state != self.last_depth_bias_state {
            self.build_shadow_map_pipeline(state, display);
            self.last_depth_bias_state = self.depth_bias_state;
        }
        self.shadow_mapping_uniform
            .update_with(display.queue(), |u| {
                u.lights = lights.to_vec();
                u.view_frustum = *view_frustum;
            });

        let mut bufs = vec![];
        for (i, light) in lights.iter().enumerate() {
            let buf = state
                .render_pass(
                    &display,
                    "Shadow Mapping Pass",
                    &[RenderTarget::TextureRef(self.shadow_map_debug_textures[i])],
                    Some(RenderTarget::TextureView(&self.shadow_map_target_views[i])),
                    &light.view_proj_uniforms(&view_frustum),
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
