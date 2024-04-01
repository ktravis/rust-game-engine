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

struct LightsUniform {
    positions: array<vec4<f32>, 16>,
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
    @location(1) world_pos: vec3<f32>,
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
    out.view_space_normal = (view_proj_uniforms.view * model_transform * vec4(vertex.normal, 0.0)).xyz;
    out.world_pos = model.xyz;
    return out;
}

// Fragment shader

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

const AMBIENT_LIGHT_FACTOR = 0.1;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let c = in.tint_color * textureSample(t_diffuse, s_diffuse, in.tex_coords);

    var specular = 0.0;
    var diffuse = 0.0;
    for (var i = 0; i < i32(lights.count); i++) {
        let light_pos_w = lights.positions[i];
        let view_dir_v = view_proj_uniforms.view * vec4(normalize(view_proj_uniforms.camera_pos - in.world_pos), 0.0);
        let light_pos_v = view_proj_uniforms.view * vec4(light_pos_w.xyz, 0.0);//vec4(20.0 * cos(global_uniforms.time), 10.0, 20.0 * sin(global_uniforms.time), 0.0);
        let light_dir_v = normalize(light_pos_v.xyz - in.view_space_normal.xyz);
        let half_dir_v = normalize(view_dir_v.xyz + light_dir_v);
        let reflect_dir_v = reflect(-light_dir_v, in.view_space_normal.xyz);

        specular += pow(max(dot(view_dir_v.xyz, reflect_dir_v), 0.0), 32.0);
        diffuse += max(dot(in.view_space_normal, half_dir_v), 0.0);
    }
    let light_factor = AMBIENT_LIGHT_FACTOR + diffuse + specular;
    return vec4(light_factor * c.xyz, c.w);
}
