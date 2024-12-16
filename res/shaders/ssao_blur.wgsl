// Vertex shader

struct GlobalUniforms {
    time: f32,
}

@group(1) @binding(0)
var<uniform> global_uniforms: GlobalUniforms;

struct ViewProjectionUniforms {
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
    camera_pos: vec3<f32>,
}

@group(2) @binding(0)
var<uniform> view_proj_uniforms: ViewProjectionUniforms;

struct VertexInput {
    @location(0) position: vec4<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct InstanceInput {
    @location(2) uv_scale: vec2<f32>,
    @location(3) uv_offset: vec2<f32>,
    @location(4) tint: vec4<f32>,
    @location(5) model_1: vec4<f32>,
    @location(6) model_2: vec4<f32>,
    @location(7) model_3: vec4<f32>,
    @location(8) model_4: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) tint_color: vec4<f32>,
}

@vertex
fn vs_main(
    vertex: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    let model_transform = mat4x4<f32>(
        instance.model_1,
        instance.model_2,
        instance.model_3,
        instance.model_4,
    );
    var out: VertexOutput;
    out.tex_coords = instance.uv_offset + instance.uv_scale * vertex.tex_coords;
    var model = vertex.position;
    model.x = model.x * 2.0 - 1.0;
    model.y = model.y * 2.0 - 1.0;
    let model_view = model;
    out.clip_position = model_view;
    out.tint_color = instance.tint;
    return out;
}

// Fragment shader

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

struct BlurUniforms {
    half_kernel_size: i32,
    sharpness: f32,
    step: vec2<f32>,
}

@group(4) @binding(0)
var<uniform> blur_settings: BlurUniforms;

@group(3) @binding(0)
var g_position: texture_2d<f32>;
@group(3) @binding(1)
var g_position_sampler: sampler;

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
    var center_depth = textureSample(g_position, g_position_sampler, in.tex_coords).z;
    var weight = 1.0;

    for (var i = 1; i <= blur_settings.half_kernel_size * 2; i++) {
        let r = f32(i);
        let uv = in.tex_coords + r * blur_settings.step;
        let sample_color = textureSample(t_diffuse, s_diffuse, uv).r;
        let sample_depth = textureSample(g_position, g_position_sampler, in.tex_coords).z;
        let w = blur_weight(r, center_depth, sample_depth);
        weight += w;
        result += sample_color * w;
    }

    for (var i = 1; i <= blur_settings.half_kernel_size * 2; i++) {
        let r = f32(i);
        let uv = in.tex_coords - r * blur_settings.step;
        let sample_color = textureSample(t_diffuse, s_diffuse, uv).r;
        let sample_depth = textureSample(g_position, g_position_sampler, uv).z;
        let w = blur_weight(r, center_depth, sample_depth);
        weight += w;
        result += sample_color * w;
    }

    result /= weight;
    return result;
}
