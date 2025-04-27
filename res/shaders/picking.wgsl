@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

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
    @location(2) normal: vec3<f32>,
}

struct InstanceInput {
    @location(3) model_1: vec4<f32>,
    @location(4) model_2: vec4<f32>,
    @location(5) model_3: vec4<f32>,
    @location(6) model_4: vec4<f32>,
    @location(7) id_color: vec2<u32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(1) world_pos: vec4<f32>,
    @location(2) id_color: vec2<u32>,
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
    let model = model_transform * vertex.position;
    let model_view = view_proj_uniforms.view * model;
    out.clip_position = view_proj_uniforms.projection * model_view;
    out.world_pos = model;
    out.id_color = instance.id_color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<u32> {
    return vec4(in.id_color.x & 0xff, (in.id_color.x >> 8) & 0xff, (in.id_color.x >> 16) & 0xff, in.id_color.x >> 24);
}
