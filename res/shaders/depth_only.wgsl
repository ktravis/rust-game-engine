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

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
}

@vertex
fn vs_main(
    vertex: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    let model_transform = mat4x4<f32>(
        instance.model_1x,
        instance.model_2x,
        instance.model_3x,
        instance.model_4x,
    );
    let normal_matrix = mat4x4<f32>(
        instance.normal_1x,
        instance.normal_2x,
        instance.normal_3x,
        instance.normal_4x,
    );
    let model_view = (view_proj_uniforms.view * model_transform);
    let model_view_pos = model_view * vertex.position;
    var out: VertexOutput;
    out.clip_position = view_proj_uniforms.projection * model_view_pos;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) {
}
