use bytemuck::Zeroable;
use glam::{vec2, vec4, Vec2, Vec4};
use shadertype_derive::shader_uniform_type;

use crate::{
    camera::Camera,
    geom::{BasicVertexData, Point},
    renderer::{
        bindings::{
            create_uniform_bind_group, texture_bgl_entries, BindGroup, DepthBuffer,
            UnfilteredMaterialGroup, UniformBindGroup, UNIFORM_BGL_ENTRY,
        },
        shader_type::{create_shader, GlobalUniforms},
        ssao::{BlurUniforms, SSAO_BLUR_SHADER},
        state::BoundTexture,
    },
};

use super::{bindings::ViewProjectionUniforms, Display, PipelineRef, RenderState, TextureBuilder};

mod ssao_shader {
    use crate::renderer::{
        bindings::{DepthBuffer, MaterialGroup, ViewProjectionUniforms},
        shader_type::GlobalUniforms,
    };

    pub(super) struct BindGroups {
        // group 0
        diffuse_material: MaterialGroup,
        // group 1
        global_uniforms: GlobalUniforms,
        view_proj_uniforms: ViewProjectionUniforms,
        depth_buffer: DepthBuffer,
    }
}

const SSAO_FROM_DEPTH_SHADER: &'static str = crate::wgsl!(
    r#"
@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@group(1) @binding(0)
var<uniform> global_uniforms: GlobalUniforms;

@group(2) @binding(0)
var<uniform> view_proj_uniforms: ViewProjectionUniforms;

@group(3) @binding(0)
var depth_buffer: texture_depth_2d;
@group(3) @binding(1)
var depth_buffer_sampler: sampler;

@group(4) @binding(0)
var<uniform> kernel: SSAOKernel;

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


fn reconstructPosition(coords: vec2<f32>) -> vec3<f32> {
    let x = coords.x * 2.0 - 1.0;
    let y = (1.0 - coords.y) * 2.0 - 1.0;
    let z = textureSample(depth_buffer, depth_buffer_sampler, coords);
    let position_s = vec4(x, y, z, 1.0);
    let position_v = kernel.inverse_proj * position_s;
    return position_v.xyz / position_v.w;
}

fn normalFromDepth(center: vec3<f32>, coords: vec2<f32>) -> vec3<f32> {
    let above = reconstructPosition(coords + vec2<f32>(0.0, 1.0) / global_uniforms.screen_size);
    let below = reconstructPosition(coords + vec2<f32>(0.0, -1.0) / global_uniforms.screen_size);
    var y1 = above;
    var y2 = center;
    if abs(below.z - center.z) < abs(above.z - center.z) {
        y1 = center;
        y2 = below;
    }

    let left = reconstructPosition(coords + vec2<f32>(-1.0, 0.0) / global_uniforms.screen_size);
    let right = reconstructPosition(coords + vec2<f32>(1.0, 0.0) / global_uniforms.screen_size);
    var x1 = left;
    var x2 = center;
    if abs(right.z - center.z) < abs(left.z - center.z) {
        x1 = center;
        x2 = right;
    }

    return normalize(cross(x2 - x1, y2 - y1));
}

@group(5) @binding(0)
var ssao_noise: texture_2d<f32>;
@group(5) @binding(1)
var ssao_noise_sampler: sampler;

const KERNEL_SIZE: u32 = 64;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) f32 {
    var view_pos = reconstructPosition(in.tex_coords);
    var view_space_normal = normalFromDepth(view_pos, in.tex_coords);

    var random_vec = textureSample(ssao_noise, ssao_noise_sampler, kernel.noise_texture_scale * in.tex_coords.xy).xyz;

    let tangent = normalize(random_vec - view_space_normal * dot(random_vec, view_space_normal));
    let bitangent = cross(view_space_normal, tangent);
    let TBN = mat3x3<f32>(tangent, bitangent, view_space_normal);

    var occlusion = 0.0;
    for (var i = 0; i < i32(KERNEL_SIZE); i += 1) {
        var sample = view_pos.xyz + kernel.radius * TBN * kernel.items[i].xyz;
        var offset = view_proj_uniforms.projection * vec4<f32>(sample, 1.0);
        // perspective scale for projected offset
        offset.x /= offset.w;
        offset.y /= offset.w;
        // map to [0.0, 1.0] range
        offset.x = offset.x * 0.5 + 0.5;
        offset.y = offset.y * 0.5 + 0.5;
        // invert y
        offset.y = 1.0 - offset.y;

        var sample_depth = reconstructPosition(offset.xy).z;

        var range_check = smoothstep(0.0f, 1.0f, kernel.radius / abs(view_pos.z - sample_depth));
        if sample_depth >= sample.z + kernel.bias {
            occlusion += range_check * range_check;
        }
    }
    occlusion = 1.0 - (occlusion / f32(KERNEL_SIZE));
    occlusion = pow(occlusion, 2.0);
    return occlusion;
}
"#
);

#[derive(Debug, PartialEq)]
#[shader_uniform_type]
pub struct SSAOKernel {
    pub items: [glam::f32::Vec4; 64u32 as usize],
    pub radius: f32,
    pub bias: f32,
    pub noise_texture_scale: glam::f32::Vec2,
    pub aspect_ratio: f32,
    pub tan_half_fov: f32,
    #[skip]
    pub _pad_tan_half_fov: [u8; 8u32 as usize],
    pub inverse_proj: glam::f32::Mat4,
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

    fn new(noise_texture_scale: Vec2, camera: &Camera) -> Self {
        let items = Self::generate_items();
        let aspect_ratio = camera.aspect_ratio();
        let tan_half_fov = (camera.fov_radians() / 2.0).tan();
        Self {
            items,
            radius: Self::DEFAULT_RADIUS,
            bias: Self::DEFAULT_BIAS,
            noise_texture_scale,
            aspect_ratio,
            tan_half_fov,
            inverse_proj: camera.perspective_matrix().inverse(),
            ..Zeroable::zeroed()
        }
    }
}

pub struct SSAOPass {
    pipeline: PipelineRef<BasicVertexData>,
    output_texture: BoundTexture,
    kernel: UniformBindGroup<SSAOKernel>,
    noise_texture: BindGroup<UnfilteredMaterialGroup>,
    blur_enabled: bool,
    blur_uniforms: UniformBindGroup<BlurUniforms>,
    blur_pipeline: PipelineRef<BasicVertexData>,
    // blur_temp_buffer: TextureRef,
    blur_temp_buffer: BoundTexture,
    scene_depth_buffer: BindGroup<DepthBuffer>,
    buffer_size: Point<u32>,
    view_proj_bind_group: UniformBindGroup<ViewProjectionUniforms>,
}

impl SSAOPass {
    pub const OCCLUSION_MAP_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R16Float;

    const NOISE_SCALE: usize = 4;

    pub fn new(
        state: &mut RenderState,
        display: &Display,
        size: Point<u32>,
        depth_target: DepthBuffer,
        camera: &Camera,
    ) -> Self {
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
        let noise_texture = BindGroup::new(
            display.device(),
            UnfilteredMaterialGroup {
                view: noise_texture.view.into(),
                sampler: noise_texture.sampler.into(),
            },
        );
        let kernel = create_uniform_bind_group(
            display.device(),
            SSAOKernel::new(size.as_vec2() / Self::NOISE_SCALE as f32, camera),
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
        let output_texture = TextureBuilder::render_target()
            .with_label("ssao")
            .with_format(Self::OCCLUSION_MAP_FORMAT)
            .with_filter_mode(wgpu::FilterMode::Linear)
            .with_usage(
                wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_DST
                    | wgpu::TextureUsages::RENDER_ATTACHMENT,
            )
            .build(display.device(), size);
        let output_texture = state.bind_texture(&display, output_texture);

        let scene_depth_buffer = BindGroup::new(display.device(), depth_target);
        let pipeline = state
            .pipeline_builder()
            .with_label("SSAO Pipeline")
            .with_bind_group_layouts(vec![
                &main_texture_bgl,
                &global_uniform_bgl,
                &view_proj_uniform_bgl,
                scene_depth_buffer.layout(),
                kernel.layout(),
                noise_texture.layout(),
            ])
            .with_color_target_states(vec![Some(wgpu::ColorTargetState {
                blend: None,
                format: Self::OCCLUSION_MAP_FORMAT,
                write_mask: wgpu::ColorWrites::ALL,
            })])
            .with_depth_stencil_state(None)
            .build(
                display.device(),
                &create_shader::<
                    (GlobalUniforms, ViewProjectionUniforms, SSAOKernel),
                    BasicVertexData,
                >(
                    display,
                    "ssao_from_depth",
                    SSAO_FROM_DEPTH_SHADER.to_string(),
                ),
            );
        let blur_pipeline = state
            .pipeline_builder()
            .with_label("SSAO Blur Pipeline")
            .with_color_target_states(vec![Some(wgpu::ColorTargetState {
                blend: None,
                format: Self::OCCLUSION_MAP_FORMAT,
                write_mask: wgpu::ColorWrites::ALL,
            })])
            .with_bind_group_layouts(vec![
                &main_texture_bgl,
                &global_uniform_bgl,
                &view_proj_uniform_bgl,
                scene_depth_buffer.layout(),
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
                .with_format(Self::OCCLUSION_MAP_FORMAT)
                .with_filter_mode(wgpu::FilterMode::Linear)
                .with_usage(
                    wgpu::TextureUsages::TEXTURE_BINDING
                        | wgpu::TextureUsages::COPY_DST
                        | wgpu::TextureUsages::RENDER_ATTACHMENT,
                )
                .build(display.device(), size),
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
            scene_depth_buffer,
            buffer_size: size,
            view_proj_bind_group,
        }
    }

    pub fn run(
        &mut self,
        state: &mut RenderState,
        display: &Display,
        view_projection: &ViewProjectionUniforms,
    ) -> &BoundTexture {
        let mut u = **self.kernel;
        u.inverse_proj = view_projection.projection.inverse();
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
                    r.set_bind_group(3, self.scene_depth_buffer.bind_group(), &[]);
                    r.set_bind_group(4, self.kernel.bind_group(), &[]);
                    r.set_bind_group(5, self.noise_texture.bind_group(), &[]);
                    r.set_pipeline(self.pipeline);
                    r.draw_mesh(quad);
                },
            )
            .submit();
        if self.blur_enabled {
            self.blur_uniforms.update_with(display.queue(), |s| {
                s.step = vec2(1.0 / self.buffer_size.x as f32, 0.0);
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
                        r.set_bind_group(3, self.scene_depth_buffer.bind_group(), &[]);
                        r.set_bind_group(4, self.blur_uniforms.bind_group(), &[]);
                        r.set_pipeline(self.blur_pipeline);
                        r.draw_mesh(quad);
                    },
                )
                .submit();
            self.blur_uniforms.update_with(display.queue(), |s| {
                s.step = vec2(0.0, 1.0 / self.buffer_size.y as f32);
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
                        r.set_bind_group(3, self.scene_depth_buffer.bind_group(), &[]);
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
