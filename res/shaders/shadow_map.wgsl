struct GlobalUniforms {
    time: f32,
    screen_size: vec2<f32>,
}

struct ViewProjectionUniforms {
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
    camera_pos: vec3<f32>,
    inverse_view: mat4x4<f32>,
}

struct VertexInput {
    @location(0) position: vec4<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
}

struct InstanceInput {
    @location(3) uv_scale: vec2<f32>,
    @location(4) uv_offset: vec2<f32>,
    @location(5) tint: vec4<f32>,
    @location(6) model_1x: vec4<f32>,
    @location(7) model_2x: vec4<f32>,
    @location(8) model_3x: vec4<f32>,
    @location(9) model_4x: vec4<f32>,
    @location(10) normal_1x: vec4<f32>,
    @location(11) normal_2x: vec4<f32>,
    @location(12) normal_3x: vec4<f32>,
    @location(13) normal_4x: vec4<f32>,
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@group(1) @binding(0)
var<uniform> global_uniforms: GlobalUniforms;

@group(2) @binding(0)
var<uniform> view_proj_uniforms: ViewProjectionUniforms;

@vertex
fn vs_main(
    vertex: VertexInput,
    instance: InstanceInput,
) -> @builtin(position) vec4<f32> {
    let model_transform = mat4x4<f32>(
        instance.model_1x,
        instance.model_2x,
        instance.model_3x,
        instance.model_4x,
    );
    let model = model_transform * vertex.position;
    let model_view = view_proj_uniforms.view * model;
    return view_proj_uniforms.projection * model_view;
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    return vec4(position.z, position.z, position.z, 1.0);
}

