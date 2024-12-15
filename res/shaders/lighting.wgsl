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
    @location(2) tint_color: vec4<f32>,
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

// TODO: remove
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

struct Light {
    position: vec4<f32>,
    color: vec4<f32>,
    view_proj: mat4x4<f32>,
}

struct LightsUniform {
    items: array<Light, 8>,
    count: u32,
    shadow_bias_minimum: f32,
    shadow_bias_factor: f32,
    shadow_blur_half_kernel_size: i32,
}

@group(4) @binding(0)
var<uniform> lights: LightsUniform;
@group(4) @binding(1)
var shadow_map: texture_depth_2d_array;
@group(4) @binding(2)
var shadow_map_sampler: sampler_comparison;

const AMBIENT_LIGHT_FACTOR = vec3(0.5, 0.5, 0.5);

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let view_pos = textureSample(g_position, g_position_sampler, in.tex_coords);
    let view_space_normal = textureSample(g_normal, g_normal_sampler, in.tex_coords);
    let albedo_spec = in.tint_color * textureSample(g_albedo_spec, g_albedo_spec_sampler, in.tex_coords);

    // let view_pos = view_proj_uniforms.view * world_pos;

    var specular = vec4(0.0, 0.0, 0.0, 0.0);
    var diffuse = vec4(0.0, 0.0, 0.0, 0.0);
    var total_light = vec4(clamp(textureSample(t_diffuse, s_diffuse, in.tex_coords).r, 0.0, 1.0));
    // let ao = textureSample(t_diffuse, s_diffuse, in.tex_coords).r;
    // var total_light = vec4(0.8) * (1.0 - ao);

    for (var i = 0u; i < lights.count; i++) {
        let light_color = lights.items[i].color;
        // let ambient_color = light_color * (1.0 - ao);

        let light_pos_w = lights.items[i].position;
        let view_dir_v = normalize(view_pos);
        let light_pos_v = view_proj_uniforms.view * light_pos_w;
        let light_dir_v = normalize((light_pos_v - view_pos).xyz);

        // let shadow_pos = lights.items[i].view_proj * world_pos;
        let shadow_pos = lights.items[i].view_proj * view_proj_uniforms.inverse_view * view_pos;

        var shadow = 1.0;
        // let flip_correction = vec2<f32>(0.5, -0.5);
        // let proj_correction = 1.0 / shadow_pos.w;
        // if shadow_pos.z * proj_correction > 1.0 {
        //     shadow = 1.0;
        // } else {
        //     let light_coords = shadow_pos.xy * flip_correction * proj_correction + vec2<f32>(0.5, 0.5);
        //     let bias = max(lights.shadow_bias_factor * (1.0 - dot(view_space_normal.xyz, light_dir_v)), lights.shadow_bias_minimum);
        //     // shadow = textureSampleCompare(shadow_map, shadow_map_sampler, light_coords, i, shadow_pos.z * proj_correction - bias);
        //
        //     let dim = textureDimensions(shadow_map);
        //     let texelSize = 1.0 / vec2(f32(dim.x), f32(dim.y));
        //     var weight = 0.0;
        //     for (var x = -lights.shadow_blur_half_kernel_size; x <= lights.shadow_blur_half_kernel_size; x++) {
        //         for (var y = -lights.shadow_blur_half_kernel_size; y <= lights.shadow_blur_half_kernel_size; y++) {
        //             shadow += textureSampleCompare(shadow_map, shadow_map_sampler, light_coords + vec2(f32(x), f32(y)) * texelSize, i, shadow_pos.z * proj_correction - bias);
        //             weight += 1.0;
        //         }
        //     }
        //     shadow /= weight;
        // }

        let half_dir_v = normalize(view_dir_v.xyz + light_dir_v);
        let reflect_dir_v = reflect(light_dir_v, view_space_normal.xyz);
        var specular_factor = clamp(dot(view_space_normal.xyz, half_dir_v), 0.0, 1.0);
        specular_factor = pow(specular_factor, 32.0);
        if specular_factor > 0.0 {
            let specular_intensity = 0.6;
            total_light += shadow * specular_intensity * specular_factor * light_color;
        }
        let d = clamp(dot(view_space_normal.xyz, light_dir_v), 0.0, 1.0);
        total_light += shadow * d * light_color;
        // total_light += ambient_color;
    }
    // return vec4(ao * (AMBIENT_LIGHT_FACTOR + diffuse) * albedo_spec.xyz + specular, albedo_spec.w);
    return vec4(albedo_spec.xyz * total_light.rgb, albedo_spec.w);
}



