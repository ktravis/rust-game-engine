#import global.wgsl::{GlobalUniforms, ViewProjectionUniforms}
#import inputs.wgsl::{VertexInput, InstanceInput}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@group(1) @binding(0)
var<uniform> global_uniforms: GlobalUniforms;

@group(2) @binding(0)
var<uniform> view_proj_uniforms: ViewProjectionUniforms;

@vertex
fn vs_main(
    vertex: VertexInput,
    instance: InstanceInput,
) -> @builtin(position) vec4<f32> {
    let model_transform = mat4x4<f32>(
        instance.model_1x,
        instance.model_2x,
        instance.model_3x,
        instance.model_4x,
    );
    let model = model_transform * vertex.position;
    let model_view = view_proj_uniforms.view * model;
    return view_proj_uniforms.projection * model_view;
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    return vec4(position.z, position.z, position.z, 1.0);
}

