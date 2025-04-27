@export
struct VertexInput {
    @location(0) position: vec4<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
}

@export
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
