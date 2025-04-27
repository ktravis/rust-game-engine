@export
struct GlobalUniforms {
    time: f32,
    screen_size: vec2<f32>,
}

@export
struct ViewProjectionUniforms {
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
    camera_pos: vec3<f32>,
    inverse_view: mat4x4<f32>,
}

@export
struct ModelVertexData {
    @location(0) position: vec4<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
}

@fragment
fn main() { }
