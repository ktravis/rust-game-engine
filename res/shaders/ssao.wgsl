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
    inverse_view: mat4x4<f32>,
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

struct Kernel {
    items: array<vec4<f32>, 64>,
    count: u32,
    radius: f32,
    bias: f32,
    noise_texture_scale: vec2<f32>,
}

@group(4) @binding(0)
var<uniform> kernel: Kernel;

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

        // if abs(view_space_pos.z - sample_depth) < kernel.radius {
        //     if sample_depth <= sample.z {
        //         occlusion += 1.0;
        //     }
        //     // occlusion += step(sample_depth, sample.z);
        // }

        // if sample_depth == 1.0 {
        //     continue;
        // }
        // let range_check = smoothstep(0.0, 1.0, kernel.radius / abs(view_space_pos.z - sample_depth));

        // var depth_check = 0.0;
        // if (abs(Pos.z - sampleDepth) < gSampleRad) {
        //     AO += step(sampleDepth,samplePos.z);
        // }
        // if sample_depth >= sample.z + kernel.bias {
        //     depth_check = 1.0;
        // }
        // occlusion += depth_check * range_check;
    }
    occlusion = 1.0 - (occlusion / f32(kernel.count));
    // occlusion = pow(occlusion, 2.0);
    // return vec4(occlusion, occlusion, occlusion, 1.0);
    return occlusion;
}

