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
}

@group(3) @binding(0)
var<uniform> blur_settings: BlurUniforms;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) f32 {
    let texelSize = vec2<f32>(1.0, 1.0) / vec2<f32>(textureDimensions(t_diffuse, 0));
    var result = 0.0;
    var weight = 0.0;
    for (var x = -blur_settings.half_kernel_size; x <= blur_settings.half_kernel_size; x += 1) {
        for (var y = -blur_settings.half_kernel_size; y <= blur_settings.half_kernel_size; y += 1) {
            let offset = texelSize * vec2(f32(x), f32(y));
            result += textureSample(t_diffuse, s_diffuse, in.tex_coords + offset).r;
            weight += 1.0;
        }
    }
    return result / weight;
}
