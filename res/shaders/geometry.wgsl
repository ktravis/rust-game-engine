#import global.wgsl::{GlobalUniforms, ViewProjectionUniforms}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@group(1) @binding(0)
var<uniform> global_uniforms: GlobalUniforms;

@group(2) @binding(0)
var<uniform> view_proj_uniforms: ViewProjectionUniforms;

struct VertexInput {
    @location(0) position: vec4<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
}

struct InstanceInput {
    @location(3) uv_scale: vec2<f32>,
    @location(4) uv_offset: vec2<f32>,
    @location(5) tint: vec4<f32>,
    @location(6) model_1: vec4<f32>,
    @location(7) model_2: vec4<f32>,
    @location(8) model_3: vec4<f32>,
    @location(9) model_4: vec4<f32>,
    @location(10) normal_1: vec4<f32>,
    @location(11) normal_2: vec4<f32>,
    @location(12) normal_3: vec4<f32>,
    @location(13) normal_4: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) view_pos: vec4<f32>,
    @location(2) view_space_normal: vec4<f32>,
    @location(3) tint_color: vec4<f32>,
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
    let normal_matrix = mat4x4<f32>(
        instance.normal_1,
        instance.normal_2,
        instance.normal_3,
        instance.normal_4,
    );
    var out: VertexOutput;
    out.tex_coords = instance.uv_offset + instance.uv_scale * vertex.tex_coords;
    let model_view = (view_proj_uniforms.view * model_transform);
    let model_view_pos = model_view * vertex.position;
    out.clip_position = view_proj_uniforms.projection * model_view_pos;
    out.view_space_normal = (normal_matrix * vec4(vertex.normal, 1.0));
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
    out.g_normal = normalize(in.view_space_normal);
    out.g_albedo_spec = in.tint_color * textureSample(t_diffuse, s_diffuse, in.tex_coords);
    return out;
}
