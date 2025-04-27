use std::{ops::Deref, sync::Arc};

use crate::geom::BasicVertexData;

use super::{
    instance::InstanceRenderData,
    lighting::{Light, LightsUniform},
    shaders::deferred_lighting as shader,
    state::ViewProjectionUniforms,
    BasicInstanceData, Display, PipelineRef, RenderState, RenderTarget, TextureRef,
    UniformBindGroup,
};

pub struct LightingPass {
    pipeline: PipelineRef<BasicVertexData, BasicInstanceData>,
    lights_uniform: UniformBindGroup<LightsUniform>,
}

impl LightingPass {
    pub fn new(
        state: &mut RenderState,
        display: &Display,
        geometry_pass_bgl: &wgpu::BindGroupLayout,
    ) -> Self {
        let (lights_uniform, uniform_bgl) =
            state.create_uniform_bind_group(display.device(), LightsUniform::default());
        let pipeline = state
            .pipeline_builder()
            .with_label("Lighting Render Pipeline")
            .with_extra_bind_group_layouts(vec![geometry_pass_bgl, &uniform_bgl])
            .with_depth_stencil_state(None)
            .build(
                display.device(),
                &display.device().create_shader_module(shader::DESCRIPTOR),
            );
        Self {
            pipeline,
            lights_uniform,
        }
    }

    pub fn run(
        &mut self,
        state: &mut RenderState,
        display: &Display,
        destination: RenderTarget,
        view_projection: &ViewProjectionUniforms,
        geometry_pass_bg: Arc<wgpu::BindGroup>,
        occlusion_map: TextureRef,
        lights: &[Light],
    ) {
        self.lights_uniform.update(
            display.queue(),
            LightsUniform {
                lights: lights.to_vec(),
                ..Default::default()
            },
        );
        let quad = state.quad_mesh();
        state
            .render_pass(
                display,
                "Lighting Pass",
                &[destination],
                None,
                view_projection,
                |r| {
                    use shader::globals::*;
                    r.set_bind_group(group3::GROUP, geometry_pass_bg.deref(), &[]);
                    r.set_bind_group(lights::GROUP, self.lights_uniform.bind_group().deref(), &[]);
                    r.draw_instance(&InstanceRenderData {
                        mesh: quad,
                        instance: Default::default(),
                        texture: Some(occlusion_map),
                        pipeline: Some(self.pipeline),
                    });
                },
            )
            .submit();
    }
}
