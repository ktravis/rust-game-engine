use std::ops::Deref;

use wgpu::TextureUsages;

use crate::geom::{ModelVertexData, Point};

use super::{
    instance::InstanceRenderData,
    lighting::LightsUniform,
    shaders::{self, forward as shader},
    ssao_from_depth,
    state::{BindingType, ViewProjectionUniforms},
    Display, InstanceDataWithNormalMatrix, PipelineBuilder, PipelineRef, RenderState, RenderTarget,
    Texture, TextureBuilder, TextureRef, UniformBuffer,
};

pub struct ForwardGeometryPass {
    pipeline: PipelineRef<ModelVertexData, InstanceDataWithNormalMatrix>,
    depth_only_pipeline: PipelineRef<ModelVertexData, InstanceDataWithNormalMatrix>,
    pub color_target: TextureRef,
    pub depth_target: Texture,
    pub lights_uniform: UniformBuffer<LightsUniform>,
    lights_bind_group: wgpu::BindGroup,
}

impl ForwardGeometryPass {
    pub fn new(
        state: &mut RenderState,
        display: &Display,
        size: Point<u32>,
        shadow_map: &Texture,
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
                    .create_shader_module(shaders::depth_only::DESCRIPTOR.clone()),
            );
        let texture_bgl = state.bind_group_layout(
            display.device(),
            BindingType::Texture {
                format: ssao_from_depth::SSAOPass::OCCLUSION_MAP_FORMAT,
            },
        );
        let lights_uniform_bgl = display
            .device()
            .create_bind_group_layout(&shader::globals::group3::layout());
        let lights_uniform = UniformBuffer::new(display.device(), LightsUniform::default());
        let lights_bind_group = display
            .device()
            .create_bind_group(&wgpu::BindGroupDescriptor {
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
            });
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
                    .create_shader_module(shader::DESCRIPTOR.clone()),
            );
        let color_target = state.load_texture(display, color_target);
        Self {
            pipeline,
            depth_only_pipeline,
            color_target,
            depth_target,
            lights_uniform,
            lights_bind_group,
        }
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
                    use shader::globals::*;
                    r.set_bind_group(lights::GROUP, &self.lights_bind_group, &[]);
                    r.set_bind_group(occlusion_map::GROUP, t.deref(), &[]);
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
