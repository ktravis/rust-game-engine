// Vertex shader

struct GlobalUniforms {
    time: f32,
}

@group(1) @binding(0)
var<uniform> global_uniforms: GlobalUniforms;

struct ViewProjectionUniforms {
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
}

@group(2) @binding(0)
var<uniform> view_proj_uniforms: ViewProjectionUniforms;

struct VertexInput {
    @location(0) position: vec4<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) screen_pos: vec2<f32>,
}

@vertex
fn vs_main(
    vertex: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = vertex.tex_coords;
    let model = vertex.position;
    let model_view = view_proj_uniforms.view * model;
    out.clip_position = view_proj_uniforms.projection * model_view;
    out.screen_pos = model_view.xy;
    return out;
}

// Fragment shader

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, in.tex_coords);
}
