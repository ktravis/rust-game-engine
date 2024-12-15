use std::sync::Arc;

use wgpu::{include_wgsl, TextureUsages};

use crate::geom::{ModelVertexData, Point};

use super::{
    instance::InstanceRenderData, state::ViewProjectionUniforms, Display,
    InstanceDataWithNormalMatrix, PipelineBuilder, PipelineRef, RenderState, RenderTarget, Texture,
    TextureBuilder, TextureRef,
};

pub struct GeometryPass {
    pipeline: PipelineRef<ModelVertexData, InstanceDataWithNormalMatrix>,
    // g_position: Texture,
    // g_normal: Texture,
    // g_albedo_specular: Texture,
    pub g_position: TextureRef,
    pub g_normal: TextureRef,
    pub g_albedo_specular: TextureRef,
    depth_target: Texture,
    bind_group: Arc<wgpu::BindGroup>,
    bind_group_layout: wgpu::BindGroupLayout,
}

impl GeometryPass {
    pub fn new(state: &mut RenderState, display: &Display, size: Point<u32>) -> Self {
        let g_position = TextureBuilder::render_target()
            .with_label("g_position")
            .with_format(wgpu::TextureFormat::Rgba16Float)
            .with_usage(TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING)
            .build(display.device(), size);
        let g_normal = TextureBuilder::render_target()
            .with_label("g_normal")
            .with_usage(TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING)
            .with_format(wgpu::TextureFormat::Rgba16Float)
            .build(display.device(), size);
        let g_albedo_specular = TextureBuilder::render_target()
            .with_label("g_albedo_specular")
            .with_usage(TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING)
            .build(display.device(), size);
        let depth_target = TextureBuilder::depth()
            .with_address_mode(wgpu::AddressMode::ClampToBorder)
            .with_border_color(wgpu::SamplerBorderColor::OpaqueWhite)
            .with_usage(
                wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            )
            .build(display.device(), size);
        let pipeline = state
            .pipeline_builder()
            .with_label("Geometry Pass Pipeline")
            .with_color_target_states(vec![
                Some(wgpu::ColorTargetState {
                    format: g_position.format(),
                    blend: Some(PipelineBuilder::DEFAULT_BLEND),
                    write_mask: wgpu::ColorWrites::ALL,
                }),
                Some(wgpu::ColorTargetState {
                    format: g_normal.format(),
                    blend: Some(PipelineBuilder::DEFAULT_BLEND),
                    write_mask: wgpu::ColorWrites::ALL,
                }),
                Some(wgpu::ColorTargetState {
                    format: g_albedo_specular.format(),
                    blend: Some(PipelineBuilder::DEFAULT_BLEND),
                    write_mask: wgpu::ColorWrites::ALL,
                }),
            ])
            .build(
                display.device(),
                &display
                    .device()
                    .create_shader_module(include_wgsl!("../../res/shaders/geometry.wgsl")),
            );
        let bind_group_layout =
            display
                .device()
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[
                        // g_position
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                            count: None,
                        },
                        // g_normal
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 3,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                            count: None,
                        },
                        // g_albedo_specular
                        wgpu::BindGroupLayoutEntry {
                            binding: 4,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 5,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                            count: None,
                        },
                        // g_depth
                        wgpu::BindGroupLayoutEntry {
                            binding: 6,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                sample_type: wgpu::TextureSampleType::Depth,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 7,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                            count: None,
                        },
                    ],
                    label: Some("geometry bind group layout"),
                });
        let bind_group = display
            .device()
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("lighting bind group"),
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&g_position.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&g_position.sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(&g_normal.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::Sampler(&g_normal.sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: wgpu::BindingResource::TextureView(&g_albedo_specular.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 5,
                        resource: wgpu::BindingResource::Sampler(&g_albedo_specular.sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 6,
                        resource: wgpu::BindingResource::TextureView(&depth_target.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 7,
                        resource: wgpu::BindingResource::Sampler(&depth_target.sampler),
                    },
                ],
            })
            .into();
        let g_position = state.load_texture(display, g_position);
        let g_normal = state.load_texture(display, g_normal);
        let g_albedo_specular = state.load_texture(display, g_albedo_specular);
        Self {
            pipeline,
            g_position,
            g_normal,
            g_albedo_specular,
            depth_target,
            bind_group,
            bind_group_layout,
        }
    }

    pub fn bind_group(&self) -> &Arc<wgpu::BindGroup> {
        &self.bind_group
    }

    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    // pub fn shadow_map_texture(&self) -> &Texture {
    //     &self.shadow_map
    // }

    pub fn run(
        &mut self,
        state: &mut RenderState,
        display: &Display,
        view_projection: &ViewProjectionUniforms,
        scene: &[InstanceRenderData<ModelVertexData, InstanceDataWithNormalMatrix>],
    ) {
        state
            .render_pass(
                &display,
                "Geometry Pass",
                &[
                    // RenderTarget::TextureView(&self.g_position.view),
                    // RenderTarget::TextureView(&self.g_normal.view),
                    // RenderTarget::TextureView(&self.g_albedo_specular.view),
                    RenderTarget::TextureRef(self.g_position),
                    RenderTarget::TextureRef(self.g_normal),
                    RenderTarget::TextureRef(self.g_albedo_specular),
                ],
                Some(RenderTarget::TextureView(&self.depth_target.view)),
                view_projection,
                |r| {
                    for render_data in scene {
                        r.draw_instance(&InstanceRenderData {
                            pipeline: Some(self.pipeline),
                            ..*render_data
                        });
                    }
                },
            )
            .submit();
    }
}
