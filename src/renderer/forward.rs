use std::ops::Deref;

use wgpu::{include_wgsl, TextureUsages};

use crate::geom::{ModelVertexData, Point};

use super::{
    instance::InstanceRenderData,
    ssao_from_depth,
    state::{BoundTexture, ViewProjectionUniforms},
    Display, InstanceDataWithNormalMatrix, PipelineBuilder, PipelineRef, RenderState, RenderTarget,
    Texture, TextureBuilder, TextureRef, UniformBindGroup,
};

pub struct ForwardGeometryPass {
    pipeline: PipelineRef<ModelVertexData, InstanceDataWithNormalMatrix>,
    depth_only_pipeline: PipelineRef<ModelVertexData, InstanceDataWithNormalMatrix>,
    pub color_target: TextureRef,
    pub depth_target: Texture,
    depth_target_bind_group: wgpu::BindGroup,
    depth_target_bgl: wgpu::BindGroupLayout,
}

impl ForwardGeometryPass {
    pub fn new(
        state: &mut RenderState,
        display: &Display,
        size: Point<u32>,
        lights_uniform_bgl: &wgpu::BindGroupLayout,
    ) -> Self {
        let color_target = TextureBuilder::render_target()
            .with_label("color_target")
            .with_usage(TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING)
            .build(display.device(), size);
        let depth_target = TextureBuilder::depth()
            .with_address_mode(wgpu::AddressMode::ClampToBorder)
            .with_border_color(wgpu::SamplerBorderColor::OpaqueWhite)
            .with_compare_func(None)
            .with_usage(
                wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            )
            .build(display.device(), size);
        let depth_stencil_state = Some(wgpu::DepthStencilState {
            format: depth_target.format(),
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::LessEqual,
            stencil: Default::default(),
            bias: Default::default(),
        });
        let depth_only_pipeline = state
            .pipeline_builder()
            .with_label("Forward Rendering (Depth Prepass)")
            .with_color_target_states(vec![])
            .with_depth_stencil_state(depth_stencil_state.clone())
            .build(
                display.device(),
                &display
                    .device()
                    .create_shader_module(include_wgsl!("../../res/shaders/depth_only.wgsl")),
            );
        let texture_bgl = state.bind_group_layout(
            display.device(),
            super::state::BindingType::Texture {
                format: ssao_from_depth::SSAOPass::OCCLUSION_MAP_FORMAT,
            },
        );
        let pipeline = state
            .pipeline_builder()
            .with_label("Forward Rendering")
            .with_color_target_states(vec![Some(wgpu::ColorTargetState {
                format: color_target.format(),
                blend: Some(PipelineBuilder::DEFAULT_BLEND),
                write_mask: wgpu::ColorWrites::ALL,
            })])
            .with_depth_stencil_state(depth_stencil_state)
            .with_extra_bind_group_layouts(vec![&lights_uniform_bgl, texture_bgl.deref()])
            .build(
                display.device(),
                &display
                    .device()
                    .create_shader_module(include_wgsl!("../../res/shaders/forward.wgsl")),
            );
        let color_target = state.load_texture(display, color_target);
        let depth_target_bgl =
            display
                .device()
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("forward pass depth target bgl"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                sample_type: wgpu::TextureSampleType::Depth,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                });
        let depth_target_bind_group =
            display
                .device()
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("forward pass depth target"),
                    layout: &depth_target_bgl,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&depth_target.view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&depth_target.sampler),
                        },
                    ],
                });
        Self {
            pipeline,
            depth_only_pipeline,
            color_target,
            depth_target,
            depth_target_bgl,
            depth_target_bind_group,
        }
    }

    pub fn depth_buffer_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.depth_target_bgl
    }

    pub fn depth_buffer_bind_group(&self) -> &wgpu::BindGroup {
        &self.depth_target_bind_group
    }

    pub fn depth_prepass(
        &mut self,
        state: &mut RenderState,
        display: &Display,
        view_projection: &ViewProjectionUniforms,
        scene: &[InstanceRenderData<ModelVertexData, InstanceDataWithNormalMatrix>],
    ) {
        state
            .render_pass(
                &display,
                "Depth Pre-Pass",
                &[],
                Some(RenderTarget::TextureView(&self.depth_target.view)),
                view_projection,
                |r| {
                    for render_data in scene {
                        r.draw_instance(&InstanceRenderData {
                            pipeline: Some(self.depth_only_pipeline),
                            ..*render_data
                        });
                    }
                },
            )
            .submit();
    }

    pub fn run(
        &mut self,
        state: &mut RenderState,
        display: &Display,
        view_projection: &ViewProjectionUniforms,
        scene: &[InstanceRenderData<ModelVertexData, InstanceDataWithNormalMatrix>],
        lights_uniform_bg: &wgpu::BindGroup,
        occlusion_map: TextureRef,
    ) {
        let t = state.get_texture(occlusion_map).bind_group().clone();
        state
            .render_pass(
                &display,
                "Forward Rendering Pass",
                &[RenderTarget::TextureRef(self.color_target)],
                Some(RenderTarget::TextureView(&self.depth_target.view)),
                view_projection,
                |r| {
                    r.set_bind_group(3, lights_uniform_bg, &[]);
                    r.set_bind_group(4, t.deref(), &[]);
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
