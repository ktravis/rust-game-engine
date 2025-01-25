// Vertex shader

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
    @location(2) view_space_normal: vec3<f32>,
    @location(3) tint_color: vec4<f32>,
    @location(4) world_pos: vec4<f32>,
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
    out.view_space_normal = (normal_matrix * vec4(vertex.normal, 1.0)).xyz;
    out.view_pos = model_view_pos;
    out.tint_color = instance.tint;
    out.world_pos = model_transform * vertex.position;
    return out;
}

// Fragment shader

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

struct Light {
    direction: vec3<f32>, // spot + directional
    kind: u32,
    color: vec4<f32>,
    view_proj: mat4x4<f32>,
    position: vec3<f32>, // spot + point
    radius: f32, // spot
    reach: f32, // spot
}

struct LightsUniform {
    items: array<Light, 8>,
    count: u32,
    shadow_bias_minimum: f32,
    shadow_bias_factor: f32,
    shadow_blur_half_kernel_size: i32,
    ambient_color: vec4<f32>,
}

@group(3) @binding(0)
var<uniform> lights: LightsUniform;
@group(3) @binding(1)
var shadow_map: texture_depth_2d_array;
@group(3) @binding(2)
var shadow_map_sampler: sampler_comparison;

@group(4) @binding(0)
var occlusion_map: texture_2d<f32>;
@group(4) @binding(1)
var occlusion_map_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let view_pos = in.view_pos;
    let view_space_normal = in.view_space_normal;
    let albedo_spec = in.tint_color * textureSample(t_diffuse, s_diffuse, in.tex_coords);

    let texelSize = 1.0 / f32(textureDimensions(shadow_map).x);

    var total_light = vec3(0.0, 0.0, 0.0);

    // Material properties
    // TODO: this ignores tint alpha
    let MaterialAmbientColor = lights.ambient_color.rgb;
    let MaterialDiffuseColor = albedo_spec.rgb;
    let MaterialSpecularColor = vec3(0.4, 0.4, 0.4);

    // Normal of the computed fragment, in camera space
    let n = normalize(in.view_space_normal);
    let ao = textureSample(occlusion_map, occlusion_map_sampler, in.clip_position.xy / global_uniforms.screen_size).r;

    total_light += ao * MaterialAmbientColor * MaterialDiffuseColor;

    for (var i = 0u; i < lights.count; i++) {
        var visibility = 1.0;

        if lights.items[i].kind == 1 { // spot light
            let light_to_fragment = in.world_pos.xyz - lights.items[i].position;
            let light_dist_sqr = dot(light_to_fragment, light_to_fragment);
            let spot_factor = dot(normalize(light_to_fragment), lights.items[i].direction);
            let reach_sqr = pow(lights.items[i].reach, 2.0);
            if spot_factor > lights.items[i].radius && light_dist_sqr < reach_sqr {
                visibility = (1.0 - (1.0 - spot_factor) * 1.0 / (1.0 - lights.items[i].radius));
            } else {
                visibility = 0.0;
            }
        }

        let LightColor = lights.items[i].color.rgb;
        let LightPower = lights.items[i].color.a;
        // Direction of the light (from the fragment to the light)
        let l = normalize((view_proj_uniforms.view * vec4(lights.items[i].position, 0.0)).xyz);
        // Cosine of the angle between the normal and the light direction, 
        // clamped above 0
        //  - light is at the vertical of the triangle -> 1
        //  - light is perpendicular to the triangle -> 0
        //  - light is behind the triangle -> 0
        let cosTheta = clamp(dot(n, l), 0.0, 1.0);

        // Eye vector (towards the camera)
        let E = normalize(-in.view_pos.xyz);
        // Direction in which the triangle reflects the light
        let R = reflect(-l, n);
        // Cosine of the angle between the Eye vector and the Reflect vector,
        // clamped to 0
        //  - Looking into the reflection -> 1
        //  - Looking elsewhere -> < 1
        let cosAlpha = clamp(dot(E, R), 0.0, 1.0);

        // var bias = lights.shadow_bias_factor * tan(acos(cosTheta));
        // bias = clamp(bias, 0.0, 0.01);
        var bias = 0.0;

        let shadow_pos = lights.items[i].view_proj * in.world_pos;

        let flip_correction = vec2<f32>(0.5, -0.5);
        let proj_correction = 1.0 / shadow_pos.w;

        let ShadowCoord = shadow_pos.xy * flip_correction * proj_correction + vec2<f32>(0.5, 0.5);
        var occlusion = 0.0;

        var weight = 0.0;
        for (var x = -lights.shadow_blur_half_kernel_size; x <= lights.shadow_blur_half_kernel_size; x++) {
            for (var y = -lights.shadow_blur_half_kernel_size; y <= lights.shadow_blur_half_kernel_size; y++) {
                occlusion += 1.0 - textureSampleCompare(shadow_map, shadow_map_sampler, ShadowCoord.xy + vec2(f32(x), f32(y)) * texelSize, i, (shadow_pos.z - bias) / shadow_pos.w);
                weight += 1.0;
            }
        }
        occlusion /= weight;
        // occlusion += ao;

        visibility = clamp(visibility - occlusion, 0.0, 1.0);

        total_light += // Ambient : simulates indirect lighting
                // ao * MaterialAmbientColor + // Diffuse : "color" of the object
                visibility * MaterialDiffuseColor * LightColor * LightPower * cosTheta + // Specular : reflective highlight, like a mirror
                visibility * MaterialSpecularColor * LightColor * LightPower * pow(cosAlpha, 5.0);
    }

    return vec4(total_light.xyz, albedo_spec.w);
}
