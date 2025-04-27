use crate::geom::{ModelVertexData, Point};

use super::{
    instance::InstanceRenderData, lighting::LightsUniform, shaders, Display,
    InstanceDataWithNormalMatrix, PipelineRef, RenderState, RenderTarget, Texture, TextureBuilder,
    TextureRef,
};

pub const MAX_LIGHTS: usize = 8;

pub struct ShadowMappingPass {
    shadow_map_pipeline: PipelineRef<ModelVertexData, InstanceDataWithNormalMatrix>,
    shadow_map: Texture,
    shadow_map_target_views: [wgpu::TextureView; MAX_LIGHTS],
    pub shadow_map_debug_textures: [TextureRef; MAX_LIGHTS],
    pub depth_bias_state: wgpu::DepthBiasState,
    last_depth_bias_state: wgpu::DepthBiasState,
}

impl ShadowMappingPass {
    pub fn new(state: &mut RenderState, display: &Display) -> Self {
        let shadow_map = TextureBuilder::depth()
            .with_layers(MAX_LIGHTS as u32)
            .with_address_mode(wgpu::AddressMode::ClampToBorder)
            .with_border_color(wgpu::SamplerBorderColor::OpaqueWhite)
            .with_filter_mode(wgpu::FilterMode::Linear)
            .with_usage(
                wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            )
            .build(display.device(), Point::new(2048, 2048));
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
            shadow_map_target_views,
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
                    .create_shader_module(shaders::shadow_map::DESCRIPTOR),
            );
    }

    pub fn shadow_map_texture(&self) -> &Texture {
        &self.shadow_map
    }

    pub fn run(
        &mut self,
        state: &mut RenderState,
        display: &Display,
        lights_uniform: &LightsUniform,
        scene: &[InstanceRenderData<ModelVertexData, InstanceDataWithNormalMatrix>],
    ) {
        if self.depth_bias_state != self.last_depth_bias_state {
            self.build_shadow_map_pipeline(state, display);
            self.last_depth_bias_state = self.depth_bias_state;
        }

        let command_buffers = lights_uniform.lights.iter().enumerate().map(|(i, light)| {
            state
                .render_pass(
                    &display,
                    "Shadow Mapping Pass",
                    &[RenderTarget::TextureRef(self.shadow_map_debug_textures[i])],
                    Some(RenderTarget::TextureView(&self.shadow_map_target_views[i])),
                    &light.view_proj_uniforms(&lights_uniform.view_frustum),
                    |r| {
                        for render_data in scene {
                            r.draw_instance(&InstanceRenderData {
                                pipeline: Some(self.shadow_map_pipeline),
                                ..*render_data
                            });
                        }
                    },
                )
                .command_buffer()
        });
        display.queue().submit(command_buffers);
    }
}
