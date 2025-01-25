// Vertex shader

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

struct GlobalUniforms {
    time: f32,
    screen_size: vec2<f32>,
}

@group(1) @binding(0)
var<uniform> global_uniforms: GlobalUniforms;

struct ViewProjectionUniforms {
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
    camera_pos: vec3<f32>,
    inverse_view: mat4x4<f32>,
}

@group(2) @binding(0)
var<uniform> view_proj_uniforms: ViewProjectionUniforms;

@group(3) @binding(0)
var depth_buffer: texture_depth_2d;
@group(3) @binding(1)
var depth_buffer_sampler: sampler;

struct Kernel {
    items: array<vec4<f32>, 64>,
    count: u32,
    radius: f32,
    bias: f32,
    noise_texture_scale: vec2<f32>,

    aspect_ratio: f32,
    tan_half_fov: f32,
    inverse_proj: mat4x4<f32>,
}

@group(4) @binding(0)
var<uniform> kernel: Kernel;

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


// Fragment shader

@group(5) @binding(0)
var ssao_noise: texture_2d<f32>;
@group(5) @binding(1)
var ssao_noise_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) f32 {
    var view_pos = reconstructPosition(in.tex_coords);
    var view_space_normal = normalFromDepth(view_pos, in.tex_coords);

    var random_vec = textureSample(ssao_noise, ssao_noise_sampler, kernel.noise_texture_scale * in.tex_coords.xy).xyz;

    let tangent = normalize(random_vec - view_space_normal * dot(random_vec, view_space_normal));
    let bitangent = cross(view_space_normal, tangent);
    let TBN = mat3x3<f32>(tangent, bitangent, view_space_normal);

    var occlusion = 0.0;
    for (var i = 0; i < i32(kernel.count); i += 1) {
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
    occlusion = 1.0 - (occlusion / f32(kernel.count));
    occlusion = pow(occlusion, 2.0);
    return occlusion;
}

