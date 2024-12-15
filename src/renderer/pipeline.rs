use std::{marker::PhantomData, sync::Arc};

use itertools::Itertools;
use slotmap::Key;
use wgpu::{VertexAttribute, VertexBufferLayout};

use crate::geom::VertexData;

use super::{state::BindingType, InstanceData, RenderState, TextureBuilder};

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
    extra_bindings: Vec<BindingType>,
    extra_bind_group_layouts: Vec<&'a wgpu::BindGroupLayout>,
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

    pub const DEFAULT_BINDINGS: [BindingType; 3] = [
        BindingType::Texture {
            format: TextureBuilder::DEFAULT_FORMAT,
        }, // material texture
        BindingType::Uniform, // global
        BindingType::Uniform, // view/projection
    ];

    pub fn new(state: &'a mut RenderState) -> Self {
        Self {
            state,
            label: None,
            color_target_states: vec![Some(wgpu::ColorTargetState {
                format: TextureBuilder::DEFAULT_RENDER_FORMAT,
                blend: Some(Self::DEFAULT_BLEND),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            extra_bindings: vec![],
            extra_bind_group_layouts: vec![],
            key: None,
            cull_mode: Some(wgpu::Face::Back),
            depth_stencil_state: Some(wgpu::DepthStencilState {
                format: TextureBuilder::DEFAULT_DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: Default::default(),
                bias: Default::default(),
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

    pub fn with_extra_bindings(self, extra_bindings: Vec<BindingType>) -> Self {
        Self {
            extra_bindings,
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

    pub fn with_extra_bind_group_layouts(
        self,
        extra_bind_group_layouts: Vec<&'a wgpu::BindGroupLayout>,
    ) -> Self {
        Self {
            extra_bind_group_layouts,
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

    pub fn build<V: VertexData, I: InstanceData>(
        self,
        device: &wgpu::Device,
        shader: &wgpu::ShaderModule,
    ) -> PipelineRef<V, I> {
        let bind_group_layouts_vec = Self::DEFAULT_BINDINGS
            .iter()
            .chain(self.extra_bindings.iter())
            .map(|t| self.state.bind_group_layout(device, *t))
            .collect_vec();
        let refs = if self.extra_bind_group_layouts.len() == 0 {
            bind_group_layouts_vec.iter().map(Arc::as_ref).collect_vec()
        } else {
            bind_group_layouts_vec
                .iter()
                .map(Arc::as_ref)
                .chain(self.extra_bind_group_layouts.into_iter())
                .collect_vec()
        };
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(&format!(
                "{} Layout",
                self.label.unwrap_or("Default Pipeline")
            )),
            bind_group_layouts: &refs,
            push_constant_ranges: &[],
        });
        let vv = V::vertex_layout();
        let ii = I::vertex_layout();
        let start_location = vv
            .attributes
            .last()
            .map(|a| a.shader_location + 1)
            .unwrap_or_default();
        let mut vertex_buffers = vec![vv];
        let offset_attributes = ii
            .attributes
            .iter()
            .map(|a| VertexAttribute {
                shader_location: start_location + a.shader_location,
                ..a.clone()
            })
            .collect_vec();
        if offset_attributes.len() > 0 {
            let ii = VertexBufferLayout {
                attributes: &offset_attributes,
                ..ii
            };
            vertex_buffers.push(ii);
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
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: self.depth_stencil_state,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            // If the pipeline will be used with a multiview render pass, this
            // indicates how many array layers the attachments will have.
            multiview: None,
            cache: None,
        });
        self.state.add_pipeline(self.key, pipeline)
    }
}
