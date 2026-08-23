use bytemuck::Zeroable;
use glam::{vec2, vec4, Vec2, Vec4};
use shadertype_derive::shader_uniform_type;

use crate::{
    geom::{BasicVertexData, Point},
    renderer::{
        bindings::{
            create_uniform_bind_group, texture_bgl_entries, BindGroup, Bindable,
            UnfilteredMaterialGroup, UniformBindGroup, UNIFORM_BGL_ENTRY,
        },
        geometry::GeometryBuffers,
        instance::BasicInstanceData,
        shader_type::{create_shader, GlobalUniforms},
        state::BoundTexture,
    },
};

use super::{bindings::ViewProjectionUniforms, Display, PipelineRef, RenderState, TextureBuilder};

const SSAO_SHADER: &'static str = crate::wgsl!(
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
}

var<immediate> model_data: ModelData;

@vertex
fn vs_main(
    vertex: BasicVertexData,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = vertex.tex_coords;
    var model = vertex.position;
    model.x = model.x * 2.0 - 1.0;
    model.y = model.y * 2.0 - 1.0;
    let model_view = model;
    out.clip_position = model_view;
    return out;
}

@group(3) @binding(0)
var g_position: texture_2d<f32>;
@group(3) @binding(1)
var g_position_sampler: sampler;
@group(3) @binding(2)
var g_normal: texture_2d<f32>;
@group(3) @binding(3)
var g_normal_sampler: sampler;
@group(3) @binding(4)
var g_albedo_spec: texture_2d<f32>;
@group(3) @binding(5)
var g_albedo_spec_sampler: sampler;

@group(4) @binding(0)
var<uniform> kernel: SSAOKernel;

@group(5) @binding(0)
var ssao_noise: texture_2d<f32>;
@group(5) @binding(1)
var ssao_noise_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) f32 {
    var view_space_pos = textureSample(g_position, g_position_sampler, in.tex_coords.xy);
    var view_space_normal = normalize(textureSample(g_normal, g_normal_sampler, in.tex_coords.xy).xyz);

    var random_vec = textureSample(ssao_noise, ssao_noise_sampler, kernel.noise_texture_scale * in.tex_coords.xy).xyz;

    let tangent = normalize(random_vec - view_space_normal * dot(random_vec, view_space_normal));
    let bitangent = cross(view_space_normal, tangent);
    let TBN = mat3x3<f32>(tangent, bitangent, view_space_normal);

    var occlusion = 0.0;
    for (var i = 0; i < i32(kernel.count); i += 1) {
        var sample = view_space_pos.xyz + kernel.radius * TBN * kernel.items[i].xyz;
        var offset = view_proj_uniforms.projection * vec4<f32>(sample, 1.0);
        // perspective scale for projected offset
        offset.x /= offset.w;
        offset.y /= offset.w;
        // map to [0.0, 1.0] range
        offset.x = offset.x * 0.5 + 0.5;
        offset.y = offset.y * 0.5 + 0.5;
        // invert y
        offset.y = 1.0 - offset.y;

        var sample_depth = textureSample(g_position, g_position_sampler, offset.xy).z;

        var range_check = smoothstep(0.0f, 1.0f, kernel.radius / abs(view_space_pos.z - sample_depth));
        if sample_depth >= sample.z + kernel.bias {
            occlusion += range_check;
        }
    }
    occlusion = 1.0 - (occlusion / f32(kernel.count));
    occlusion = pow(occlusion, 2.0);
    return occlusion;
}"#
);

pub(crate) const SSAO_BLUR_SHADER: &'static str = crate::wgsl!(
    r#"
@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@group(1) @binding(0)
var<uniform> global_uniforms: GlobalUniforms;

// @group(2) @binding(0)
// var<uniform> view_proj_uniforms: ViewProjectionUniforms;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@vertex
fn vs_main(
    vertex: BasicVertexData,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = vertex.tex_coords;
    var model = vertex.position;
    model.x = model.x * 2.0 - 1.0;
    model.y = model.y * 2.0 - 1.0;
    let model_view = model;
    out.clip_position = model_view;
    return out;
}

@group(4) @binding(0)
var<uniform> blur_settings: BlurUniforms;

@group(3) @binding(0)
var depth_buffer: texture_depth_2d;
@group(3) @binding(1)
var depth_buffer_sampler: sampler;

fn blur_weight(radius: f32, center_depth: f32, sample_depth: f32) -> f32 {
    let blur_sigma = (f32(blur_settings.half_kernel_size) + 1.0) * 0.5;
    let blur_falloff = 1.0 / (2.0 * blur_sigma * blur_sigma);
    let depth_diff = (sample_depth - center_depth) * blur_settings.sharpness;
    let weight = exp2(-radius * radius * blur_falloff - depth_diff * depth_diff);
    return weight;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) f32 {
    let texelSize = vec2<f32>(1.0, 1.0) / vec2<f32>(textureDimensions(t_diffuse, 0));

    var result = textureSample(t_diffuse, s_diffuse, in.tex_coords).r;
    var center_depth = textureSample(depth_buffer, depth_buffer_sampler, in.tex_coords);
    var weight = 1.0;

    for (var i = 1; i <= blur_settings.half_kernel_size; i++) {
        let r = f32(i);
        let uv = in.tex_coords + r * blur_settings.step;
        let sample_color = textureSample(t_diffuse, s_diffuse, uv).r;
        let sample_depth = textureSample(depth_buffer, depth_buffer_sampler, in.tex_coords);
        let w = blur_weight(r, center_depth, sample_depth);
        weight += w;
        result += sample_color * w;
    }

    for (var i = 1; i <= blur_settings.half_kernel_size; i++) {
        let r = f32(i);
        let uv = in.tex_coords - r * blur_settings.step;
        let sample_color = textureSample(t_diffuse, s_diffuse, uv).r;
        let sample_depth = textureSample(depth_buffer, depth_buffer_sampler, in.tex_coords);
        let w = blur_weight(r, center_depth, sample_depth);
        weight += w;
        result += sample_color * w;
    }

    result /= weight;
    return result;
}"#
);

#[derive(Debug, PartialEq)]
#[shader_uniform_type]
pub struct SSAOKernel {
    pub items: [glam::f32::Vec4; 64u32 as usize],
    pub count: u32,
    pub radius: f32,
    pub bias: f32,
    #[skip]
    pub _pad_bias: [u8; 4u32 as usize],
    pub noise_texture_scale: glam::f32::Vec2,
    #[skip]
    pub _pad: [u8; 8u32 as usize],
}

impl SSAOKernel {
    const SIZE: usize = 64;
    const DEFAULT_RADIUS: f32 = 0.3;
    const DEFAULT_BIAS: f32 = 0.025;

    fn generate_items() -> [Vec4; Self::SIZE] {
        std::array::from_fn(|i| {
            let scale = i as f32 / Self::SIZE as f32;
            let v = rand::random::<f32>()
                * vec4(
                    2.0 * rand::random::<f32>() - 1.0,
                    2.0 * rand::random::<f32>() - 1.0,
                    rand::random::<f32>(),
                    0.0,
                )
                .normalize();
            v * (0.05 + 0.95 * scale * scale)
        })
    }

    fn new(noise_texture_scale: Vec2) -> Self {
        let items = Self::generate_items();
        Self {
            items,
            count: Self::SIZE as u32,
            radius: Self::DEFAULT_RADIUS,
            bias: Self::DEFAULT_BIAS,
            noise_texture_scale,
            ..Zeroable::zeroed()
        }
    }
}

#[shader_uniform_type]
pub struct BlurUniforms {
    pub half_kernel_size: i32,
    pub sharpness: f32,
    pub step: Vec2,
}

impl Default for BlurUniforms {
    fn default() -> Self {
        Self {
            half_kernel_size: 2,
            sharpness: 40.0,
            step: Vec2::ZERO,
        }
    }
}

pub struct SSAOPass {
    pipeline: PipelineRef<(BasicVertexData, BasicInstanceData)>,
    output_texture: BoundTexture,
    kernel: UniformBindGroup<SSAOKernel>,
    noise_texture: BindGroup<UnfilteredMaterialGroup>,
    blur_enabled: bool,
    blur_uniforms: UniformBindGroup<BlurUniforms>,
    blur_pipeline: PipelineRef<(BasicVertexData, BasicInstanceData)>,
    blur_temp_buffer: BoundTexture,
    view_proj_bind_group: UniformBindGroup<ViewProjectionUniforms>,
}

impl SSAOPass {
    const NOISE_SCALE: usize = 4;
    pub fn new(state: &mut RenderState, display: &Display) -> Self {
        let fb_size = display.size_pixels();
        let noise: [Vec4; Self::NOISE_SCALE * Self::NOISE_SCALE] = std::array::from_fn(|_| {
            vec4(
                2.0 * rand::random::<f32>() - 1.0,
                2.0 * rand::random::<f32>() - 1.0,
                0.0,
                1.0,
            )
        });
        let noise_texture = TextureBuilder::labeled("ssao_noise")
            .with_usage(wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST)
            .with_format(wgpu::TextureFormat::Rgba32Float)
            .with_address_mode(wgpu::AddressMode::Repeat)
            .from_raw_bytes(
                display.device(),
                display.queue(),
                bytemuck::bytes_of(&noise),
                Point::new(Self::NOISE_SCALE as _, Self::NOISE_SCALE as _),
            );

        let noise_texture = UnfilteredMaterialGroup {
            view: noise_texture.view.into(),
            sampler: noise_texture.sampler.into(),
        };
        let noise_texture =
            BindGroup::<UnfilteredMaterialGroup>::new(display.device(), noise_texture);
        let kernel = create_uniform_bind_group(
            display.device(),
            SSAOKernel::new(fb_size.as_vec2() / Self::NOISE_SCALE as f32),
        );
        let output_texture = state.bind_texture(
            &display,
            TextureBuilder::render_target()
                .with_label("ssao")
                .with_format(wgpu::TextureFormat::R16Float)
                .with_filter_mode(wgpu::FilterMode::Linear)
                .with_usage(
                    wgpu::TextureUsages::TEXTURE_BINDING
                        | wgpu::TextureUsages::COPY_DST
                        | wgpu::TextureUsages::RENDER_ATTACHMENT,
                )
                .build(display.device(), fb_size),
        );
        let main_texture_bgl =
            display
                .device()
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("main texture"),
                    entries: &texture_bgl_entries(TextureBuilder::DEFAULT_FORMAT),
                });
        let global_uniform_bgl =
            display
                .device()
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("global uniform bg layout"),
                    entries: &[UNIFORM_BGL_ENTRY],
                });
        let view_proj_uniform_bgl =
            display
                .device()
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("view proj uniform bg layout"),
                    entries: &[UNIFORM_BGL_ENTRY],
                });
        let geometry_buffers_bgl =
            GeometryBuffers::create_layout(display.device(), "geometry buffers");
        let pipeline = state
            .pipeline_builder()
            .with_label("SSAO Pipeline")
            .with_bind_group_layouts(vec![
                &main_texture_bgl,
                &global_uniform_bgl,
                &view_proj_uniform_bgl,
                &geometry_buffers_bgl,
                kernel.layout(),
                noise_texture.layout(),
            ])
            .with_color_target_states(vec![Some(wgpu::ColorTargetState {
                blend: None,
                format: wgpu::TextureFormat::R16Float,
                write_mask: wgpu::ColorWrites::ALL,
            })])
            .with_depth_stencil_state(None)
            .build(
                display.device(),
                &create_shader::<
                    (GlobalUniforms, ViewProjectionUniforms, SSAOKernel),
                    BasicVertexData,
                >(display, "ssao", SSAO_SHADER.to_string()),
            );
        let blur_pipeline = state
            .pipeline_builder()
            .with_label("SSAO Blur Pipeline")
            .with_color_target_states(vec![Some(wgpu::ColorTargetState {
                blend: None,
                format: wgpu::TextureFormat::R16Float,
                write_mask: wgpu::ColorWrites::ALL,
            })])
            .with_bind_group_layouts(vec![
                &main_texture_bgl,
                &global_uniform_bgl,
                &view_proj_uniform_bgl,
                &geometry_buffers_bgl,
                kernel.layout(),
            ])
            .with_depth_stencil_state(None)
            .build(
                display.device(),
                &create_shader::<
                    (GlobalUniforms, ViewProjectionUniforms, BlurUniforms),
                    BasicVertexData,
                >(display, "ssao_blur", SSAO_BLUR_SHADER.to_string()),
            );
        let blur_temp_buffer = state.bind_texture(
            display,
            TextureBuilder::render_target()
                .with_label("blurred_ssao")
                .with_format(wgpu::TextureFormat::R16Float)
                .with_filter_mode(wgpu::FilterMode::Linear)
                .with_usage(
                    wgpu::TextureUsages::TEXTURE_BINDING
                        | wgpu::TextureUsages::COPY_DST
                        | wgpu::TextureUsages::RENDER_ATTACHMENT,
                )
                .build(display.device(), fb_size),
        );

        let blur_uniforms = create_uniform_bind_group(display.device(), Default::default());
        let view_proj_bind_group =
            create_uniform_bind_group(display.device(), ViewProjectionUniforms::default());
        Self {
            pipeline,
            output_texture,
            kernel,
            noise_texture,
            blur_pipeline,
            blur_uniforms,
            blur_enabled: true,
            blur_temp_buffer,
            view_proj_bind_group,
        }
    }

    pub fn run(
        &mut self,
        state: &mut RenderState,
        display: &Display,
        view_projection: &ViewProjectionUniforms,
        geometry_buffers: &BindGroup<GeometryBuffers>,
    ) -> &BoundTexture {
        let u = **self.kernel;
        self.kernel.update(display.queue(), u);
        self.view_proj_bind_group
            .update(display.queue(), *view_projection);
        let quad = state.quad_mesh();
        state
            .render_pass(
                &display,
                "SSAO Pass",
                &[&self.output_texture.resource.view],
                None,
                |r| {
                    let default_texture = r.render_state.get_texture(None).clone();
                    r.set_bind_group(0, &default_texture, &[]);
                    let global_uniforms = r.render_state.global_uniforms.bind_group().clone();
                    r.set_bind_group(1, &global_uniforms, &[]);
                    r.set_bind_group(2, self.view_proj_bind_group.bind_group(), &[]);
                    r.set_bind_group(3, geometry_buffers.bind_group(), &[]);
                    r.set_bind_group(4, self.kernel.bind_group(), &[]);
                    r.set_bind_group(5, self.noise_texture.bind_group(), &[]);
                    r.set_pipeline(self.pipeline);
                    r.draw_mesh(quad);
                },
            )
            .submit();
        if self.blur_enabled {
            self.blur_uniforms.update_with(display.queue(), |s| {
                s.step = vec2(1.0 / display.size_pixels().x as f32, 0.0);
            });
            state
                .render_pass(
                    display,
                    "SSAO Blur Pass - X",
                    &[&self.blur_temp_buffer.resource.view],
                    None,
                    |r| {
                        r.set_bind_group(0, self.output_texture.bind_group(), &[]);
                        let global_uniforms = r.render_state.global_uniforms.bind_group().clone();
                        r.set_bind_group(1, &global_uniforms, &[]);
                        r.set_bind_group(2, self.view_proj_bind_group.bind_group(), &[]);
                        r.set_bind_group(3, geometry_buffers.bind_group(), &[]);
                        r.set_bind_group(4, self.blur_uniforms.bind_group(), &[]);
                        r.set_pipeline(self.blur_pipeline);
                        r.draw_mesh(quad);
                    },
                )
                .submit();
            self.blur_uniforms.update_with(display.queue(), |s| {
                s.step = vec2(0.0, 1.0 / display.size_pixels().y as f32);
            });
            state
                .render_pass(
                    display,
                    "SSAO Blur Pass - Y",
                    &[&self.output_texture.resource.view],
                    None,
                    |r| {
                        r.set_bind_group(0, self.blur_temp_buffer.bind_group(), &[]);
                        let global_uniforms = r.render_state.global_uniforms.bind_group().clone();
                        r.set_bind_group(1, &global_uniforms, &[]);
                        r.set_bind_group(2, self.view_proj_bind_group.bind_group(), &[]);
                        r.set_bind_group(3, geometry_buffers.bind_group(), &[]);
                        r.set_bind_group(4, self.blur_uniforms.bind_group(), &[]);
                        r.set_pipeline(self.blur_pipeline);
                        r.draw_mesh(quad);
                    },
                )
                .submit();
        }
        &self.output_texture
    }

    pub fn debug_ui(&mut self, ui: &mut egui::Ui) {
        ui.add(egui::Slider::new(&mut self.kernel.radius, 0.0..=5.0).text("radius"));
        ui.add(egui::Slider::new(&mut self.kernel.bias, 0.0..=2.0).text("bias"));
        if ui.add(egui::Button::new("Regenerate")).clicked() {
            self.kernel.items = SSAOKernel::generate_items();
        }

        ui.separator();
        ui.label("Blur");
        ui.add(egui::Checkbox::new(&mut self.blur_enabled, "enabled"));
        ui.add(
            egui::Slider::new(&mut self.blur_uniforms.half_kernel_size, 0..=10)
                .text("half kernel size"),
        );
        ui.add(
            egui::Slider::new(&mut self.blur_uniforms.sharpness, 0.0..=100.0)
                .text("edge sharpness"),
        );
    }
}
