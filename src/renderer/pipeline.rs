use std::marker::PhantomData;

use itertools::Itertools;
use slotmap::Key;

use crate::renderer::shader_type::VertexInput;

use super::{RenderState, TextureBuilder};

slotmap::new_key_type! {
    pub(super) struct RawPipelineRef;
}

#[derive(Copy, Clone, Debug, Default)]
pub struct PipelineRef<V, I> {
    raw: RawPipelineRef,
    _marker: PhantomData<(V, I)>,
}

impl<V, I> PipelineRef<V, I> {
    pub(super) fn raw(self) -> RawPipelineRef {
        self.raw
    }

    pub fn is_null(&self) -> bool {
        self.raw.is_null()
    }
}

impl<V, I> From<RawPipelineRef> for PipelineRef<V, I> {
    fn from(raw: RawPipelineRef) -> Self {
        PipelineRef {
            raw,
            _marker: PhantomData,
        }
    }
}

pub struct PipelineBuilder<'a> {
    state: &'a mut RenderState,
    label: Option<&'a str>,
    color_target_states: Vec<Option<wgpu::ColorTargetState>>,
    bind_group_layouts: Vec<&'a wgpu::BindGroupLayout>,
    key: Option<RawPipelineRef>,
    cull_mode: Option<wgpu::Face>,
    depth_stencil_state: Option<wgpu::DepthStencilState>,
}

impl<'a> PipelineBuilder<'a> {
    pub const DEFAULT_BLEND: wgpu::BlendState = wgpu::BlendState {
        color: wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::SrcAlpha,
            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
            operation: wgpu::BlendOperation::Add,
        },
        alpha: wgpu::BlendComponent::OVER,
    };

    pub fn new(state: &'a mut RenderState) -> Self {
        Self {
            state,
            label: None,
            color_target_states: vec![Some(wgpu::ColorTargetState {
                format: TextureBuilder::DEFAULT_RENDER_FORMAT,
                blend: Some(Self::DEFAULT_BLEND),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            bind_group_layouts: vec![],
            key: None,
            cull_mode: Some(wgpu::Face::Back),
            depth_stencil_state: Some(wgpu::DepthStencilState {
                format: TextureBuilder::DEFAULT_DEPTH_FORMAT,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::LessEqual),
                stencil: Default::default(),
                bias: wgpu::DepthBiasState {
                    clamp: 1.0,
                    ..Default::default()
                },
            }),
        }
    }

    pub fn with_key<V, I>(self, key: PipelineRef<V, I>) -> Self {
        let key = if key.is_null() { None } else { Some(key.raw()) };
        Self { key, ..self }
    }

    pub fn with_label(self, label: &'a str) -> Self {
        Self {
            label: Some(label),
            ..self
        }
    }

    pub fn with_color_target_states(
        self,
        color_target_states: Vec<Option<wgpu::ColorTargetState>>,
    ) -> Self {
        Self {
            color_target_states,
            ..self
        }
    }

    pub fn for_color_target_format(self, format: wgpu::TextureFormat) -> Self {
        self.with_color_target_states(vec![Some(wgpu::ColorTargetState {
            format,
            blend: Some(Self::DEFAULT_BLEND),
            write_mask: wgpu::ColorWrites::ALL,
        })])
    }

    pub fn with_bind_group_layouts(
        self,
        bind_group_layouts: Vec<&'a wgpu::BindGroupLayout>,
    ) -> Self {
        Self {
            bind_group_layouts,
            ..self
        }
    }

    pub fn with_depth_stencil_state(
        self,
        depth_stencil_state: Option<wgpu::DepthStencilState>,
    ) -> Self {
        Self {
            depth_stencil_state,
            ..self
        }
    }

    pub fn with_cull_mode(self, cull_mode: Option<wgpu::Face>) -> Self {
        Self { cull_mode, ..self }
    }

    pub fn build<V: VertexInput, I: VertexInput>(
        self,
        device: &wgpu::Device,
        shader: &wgpu::ShaderModule,
    ) -> PipelineRef<V, I> {
        let refs = self
            .bind_group_layouts
            .into_iter()
            .map(Option::Some)
            .collect_vec();
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(&format!(
                "{} Layout",
                self.label.unwrap_or("Default Pipeline")
            )),
            bind_group_layouts: &refs,
            immediate_size: 0,
        });
        let vv = V::vertex_buffer_layout(wgpu::VertexStepMode::Vertex, 0);
        let ii = I::vertex_buffer_layout(wgpu::VertexStepMode::Instance, V::next_offset());
        let mut vertex_buffers = vec![vv.to_wgpu()];
        if ii.attributes.len() > 0 {
            vertex_buffers.push(ii.to_wgpu());
        }
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: self.label,
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: Some("vs_main"),
                buffers: &vertex_buffers,
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: Some("fs_main"),
                targets: &self.color_target_states,
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Cw,
                cull_mode: self.cull_mode,
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: true,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: self.depth_stencil_state,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        });
        self.state.add_pipeline(self.key, pipeline)
    }
}
