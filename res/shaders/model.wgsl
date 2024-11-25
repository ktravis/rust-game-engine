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

struct Light {
    position: vec4<f32>,
    color: vec4<f32>,
    view_proj: mat4x4<f32>,
}

struct LightsUniform {
    items: array<Light, 8>,
    count: u32,
}

@group(3) @binding(0)
var<uniform> lights: LightsUniform;

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
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) world_pos: vec4<f32>,
    @location(2) tint_color: vec4<f32>,
    @location(3) view_space_normal: vec3<f32>,
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
    let model = model_transform * vertex.position;
    let model_view = view_proj_uniforms.view * model;
    out.clip_position = view_proj_uniforms.projection * model_view;
    out.tint_color = instance.tint;
    out.view_space_normal = normalize(view_proj_uniforms.view * model_transform * vec4(vertex.normal, 0.0)).xyz;
    out.world_pos = model;
    return out;
}

// Fragment shader

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@group(3) @binding(1)
var shadow_map: texture_depth_2d_array;
@group(3) @binding(2)
var shadow_map_sampler: sampler_comparison;

const AMBIENT_LIGHT_FACTOR = vec3(0.1, 0.1, 0.1);

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let c = in.tint_color * textureSample(t_diffuse, s_diffuse, in.tex_coords);
    let view_pos = view_proj_uniforms.view * in.world_pos;

    var specular = vec3(0.0, 0.0, 0.0);
    var diffuse = vec3(0.0, 0.0, 0.0);
    for (var i = 0u; i < lights.count; i++) {
        let light_pos_w = lights.items[i].position;
        let view_dir_v = normalize(view_proj_uniforms.view * vec4(view_proj_uniforms.camera_pos - in.world_pos.xyz, 0.0));
        let light_pos_v = view_proj_uniforms.view * light_pos_w;
        let light_dir_v = normalize((light_pos_v - view_pos).xyz);

        let shadow_pos = lights.items[i].view_proj * in.world_pos;

        var shadow = 1.0;
        let flip_correction = vec2<f32>(0.5, -0.5);
        let proj_correction = 1.0 / shadow_pos.w;
        if shadow_pos.z * proj_correction > 1.0 {
            shadow = 1.0;
        } else {
            let light_coords = shadow_pos.xy * flip_correction * proj_correction + vec2<f32>(0.5, 0.5);
            let bias = max(0.05 * (1.0 - dot(in.view_space_normal, light_dir_v)), 0.005);
            // shadow = textureSampleCompare(shadow_map, shadow_map_sampler, light_coords, i, shadow_pos.z * proj_correction - bias);

            let dim = textureDimensions(shadow_map);
            let texelSize = 1.0 / vec2(f32(dim.x), f32(dim.y));
            for (var x = -1; x <= 1; x++) {
                for (var y = -1; y <= 1; y++) {
                    shadow += textureSampleCompare(shadow_map, shadow_map_sampler, light_coords + vec2(f32(x), f32(y)) * texelSize, i, shadow_pos.z * proj_correction - bias); 
                }    
            }
            shadow /= 9.0;
        }

        let half_dir_v = normalize(view_dir_v.xyz + light_dir_v);
        let reflect_dir_v = reflect(light_dir_v, in.view_space_normal.xyz);
        let specular_factor = clamp(dot(in.view_space_normal.xyz, half_dir_v), 0.0, 1.0);
        if (specular_factor > 0.0) {
            let s = pow(specular_factor, 32.0);
            specular += shadow * 0.6 * s * lights.items[i].color.xyz;
        }
        let d = clamp(dot(in.view_space_normal, light_dir_v), 0.0, 1.0);
        diffuse += shadow * d * lights.items[i].color.xyz;
    }
    return vec4((AMBIENT_LIGHT_FACTOR + diffuse) * c.xyz + specular, c.w);
}
