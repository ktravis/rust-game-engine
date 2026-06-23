use wgpu::TextureUsages;

use crate::{
    define_bind_group,
    geom::{ModelVertexData, Point},
    renderer::{
        bindings::{
            create_uniform_bind_group, BindGroup, Bindable, NonFilteringSampler, TextureView,
            UniformBindGroup,
        },
        shader_type::{create_shader, GlobalUniforms},
    },
};

use super::{
    instance::InstanceRenderData, state::ViewProjectionUniforms, Display,
    InstanceDataWithNormalMatrix, PipelineBuilder, PipelineRef, RenderState, RenderTarget, Texture,
    TextureBuilder,
};

const GEOMETRY_SHADER: &'static str = crate::wgsl!(
    r#"
@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@group(1) @binding(0)
var<uniform> global_uniforms: GlobalUniforms;

@group(2) @binding(0)
var<uniform> view_proj_uniforms: ViewProjectionUniforms;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) view_pos: vec4<f32>,
    @location(2) view_space_normal: vec3<f32>,
    @location(3) tint_color: vec4<f32>,
}

@vertex
fn vs_main(
    vertex: ModelVertexData,
    instance: InstanceDataWithNormalMatrix,
) -> VertexOutput {
    let model_transform = mat4x4<f32>(
        instance.transform_1,
        instance.transform_2,
        instance.transform_3,
        instance.transform_4,
    );
    let normal_matrix = mat3x3<f32>(
        instance.normal_matrix_1,
        instance.normal_matrix_2,
        instance.normal_matrix_3,
    );
    var out: VertexOutput;
    out.tex_coords = instance.subtexture_offset + instance.subtexture_scale * vertex.tex_coords;
    let model_view = (view_proj_uniforms.view * model_transform);
    let model_view_pos = model_view * vertex.position;
    out.clip_position = view_proj_uniforms.projection * model_view_pos;
    out.view_space_normal = normalize(normal_matrix * vertex.normal);
    out.view_pos = model_view_pos;
    out.tint_color = instance.tint;
    return out;
}

struct FragmentOutput {
    @location(0)
    g_position: vec4<f32>,
    @location(1)
    g_normal: vec4<f32>,
    @location(2)
    g_albedo_spec: vec4<f32>,
}

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    var out: FragmentOutput;
    out.g_position = in.view_pos;
    out.g_normal = vec4(normalize(in.view_space_normal), 0.0);
    out.g_albedo_spec = in.tint_color * textureSample(t_diffuse, s_diffuse, in.tex_coords);
    return out;
}
"#
);

define_bind_group! {
    pub GeometryBuffers {
        position_view: TextureView<f32, 2, false>,
        position_sampler: NonFilteringSampler,
        normal_view: TextureView<f32, 2, false>,
        normal_sampler: NonFilteringSampler,
        albedo_spec_view: TextureView<f32, 2, false>,
        albedo_spec_sampler: NonFilteringSampler,
    }
}

pub struct GeometryPass {
    pipeline: PipelineRef<ModelVertexData, InstanceDataWithNormalMatrix>,
    // pub g_position: Texture,
    // pub g_normal: Texture,
    // pub g_albedo_specular: Texture,
    depth_target: Texture,
    bind_group: BindGroup<GeometryBuffers>,
    view_proj_bind_group: UniformBindGroup<ViewProjectionUniforms>,
}

impl GeometryPass {
    pub fn new(state: &mut RenderState, display: &Display, size: Point<u32>) -> Self {
        let g_position = TextureBuilder::render_target()
            .with_label("g_position")
            .with_format(wgpu::TextureFormat::Rgba32Float)
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
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                }),
                Some(wgpu::ColorTargetState {
                    format: g_normal.format(),
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                }),
                Some(wgpu::ColorTargetState {
                    format: g_albedo_specular.format(),
                    blend: Some(PipelineBuilder::DEFAULT_BLEND),
                    write_mask: wgpu::ColorWrites::ALL,
                }),
            ])
            .with_depth_stencil_state(Some(wgpu::DepthStencilState {
                format: depth_target.format(),
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::LessEqual),
                stencil: Default::default(),
                bias: Default::default(),
            }))
            .build(
                display.device(),
                &create_shader::<
                    (GlobalUniforms, ViewProjectionUniforms),
                    (ModelVertexData, InstanceDataWithNormalMatrix),
                >(display, "geometry", GEOMETRY_SHADER.to_string()),
            );
        let bind_group = BindGroup::new(
            display.device(),
            GeometryBuffers {
                position_view: g_position.view.into(),
                position_sampler: g_position.sampler.into(),
                normal_view: g_normal.view.into(),
                normal_sampler: g_normal.sampler.into(),
                albedo_spec_view: g_albedo_specular.view.into(),
                albedo_spec_sampler: g_albedo_specular.sampler.into(),
            },
        );
        let view_proj_bind_group =
            create_uniform_bind_group(display.device(), ViewProjectionUniforms::default());
        // let g_position = state.load_texture(display, g_position);
        // let g_normal = state.load_texture(display, g_normal);
        // let g_albedo_specular = state.load_texture(display, g_albedo_specular);
        Self {
            pipeline,
            depth_target,
            bind_group,
            view_proj_bind_group,
        }
    }

    pub fn bind_group(&self) -> &BindGroup<GeometryBuffers> {
        &self.bind_group
    }

    pub fn run(
        &mut self,
        state: &mut RenderState,
        display: &Display,
        view_projection: &ViewProjectionUniforms,
        scene: &[InstanceRenderData<ModelVertexData, InstanceDataWithNormalMatrix>],
    ) {
        self.view_proj_bind_group
            .update(display.queue(), *view_projection);
        state
            .render_pass(
                &display,
                "Geometry Pass",
                &[
                    RenderTarget::TextureView(&self.bind_group.position_view.raw()),
                    RenderTarget::TextureView(&self.bind_group.normal_view.raw()),
                    RenderTarget::TextureView(&self.bind_group.albedo_spec_view.raw()),
                ],
                Some(RenderTarget::TextureView(&self.depth_target.view)),
                &self.view_proj_bind_group,
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
