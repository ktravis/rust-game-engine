use itertools::Itertools;
use wgpu::TextureUsages;

use crate::{
    define_bind_group,
    geom::{ModelVertexData, Point},
    renderer::{
        bindings::{
            create_uniform_bind_group, BindGroup, Bindable, ComparisonSampler,
            DepthTextureArrayView, DepthTextureView, MaterialGroup, TextureSampler,
            UniformBindGroup, UniformBuffer,
        },
        instance::{InstanceDataWithNormalMatrix, InstanceStorage},
        lighting::{LightRaw, LightingUniformsRaw},
        shader_type::{create_shader, GlobalUniforms},
        state::BoundTexture,
    },
};

use super::{
    bindings::ViewProjectionUniforms, instance::InstanceRenderData, lighting::LightsUniform,
    Display, Pipeline, PipelineBuilder, RenderState, TextureBuilder,
};

const DEPTH_ONLY_SHADER: &'static str = crate::wgsl!(
    r#"
@group(0) @binding(0)
var<uniform> view_proj_uniforms: ViewProjectionUniforms;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
}

@vertex
fn vs_main(
    vertex: ModelVertexData,
    instance: InstanceDataWithNormalMatrix,
) -> VertexOutput {
    let model_transform = mat4x4<f32>(
        instance.transform_1,
        instance.transform_2,
        instance.transform_3,
        instance.transform_4,
    );
    let model_view = (view_proj_uniforms.view * model_transform);
    let model_view_pos = model_view * vertex.position;
    var out: VertexOutput;
    out.clip_position = view_proj_uniforms.projection * model_view_pos;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) { }"#
);

const FORWARD_LIGHTING_SHADER: &'static str = crate::wgsl!(
    r#"
@group(0) @binding(0)
var texture_array: binding_array<texture_2d<f32>, 256>;
@group(0) @binding(1)
var sampler_array: binding_array<sampler, 256>;

@group(1) @binding(0)
var<uniform> global_uniforms: GlobalUniforms;

@group(2) @binding(0)
var<uniform> view_proj_uniforms: ViewProjectionUniforms;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) view_pos: vec4<f32>,
    @location(2) view_space_normal: vec3<f32>,
    @location(3) tint_color: vec4<f32>,
    @location(4) world_pos: vec4<f32>,
    @location(5) material: u32,
    @location(6) texture: u32,
}

@vertex
fn vs_main(
    vertex: ModelVertexData,
    instance: InstanceDataWithNormalMatrix,
) -> VertexOutput {
    let model_transform = mat4x4<f32>(
        instance.transform_1,
        instance.transform_2,
        instance.transform_3,
        instance.transform_4,
    );
    let normal_matrix = mat3x3<f32>(
        instance.normal_matrix_1,
        instance.normal_matrix_2,
        instance.normal_matrix_3,
    );
    var out: VertexOutput;
    out.tex_coords = instance.subtexture_offset + instance.subtexture_scale * vertex.tex_coords;
    let model_view = (view_proj_uniforms.view * model_transform);
    let model_view_pos = model_view * vertex.position;
    out.clip_position = view_proj_uniforms.projection * model_view_pos;
    out.view_space_normal = normalize(normal_matrix * vertex.normal);
    out.view_pos = model_view_pos;
    out.tint_color = instance.tint;
    out.world_pos = model_transform * vertex.position;
    out.material = instance.material;
    out.texture = instance.texture;
    return out;
}

@group(3) @binding(0)
var<uniform> lights: LightingUniformsRaw;
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
    let albedo_spec = in.tint_color * textureSample(texture_array[in.texture], sampler_array[in.texture], in.tex_coords);

    let texelSize = 1.0 / f32(textureDimensions(shadow_map).x);

    var total_light = vec3(0.0, 0.0, 0.0);

    // Material properties
    // TODO: this ignores tint alpha
    let MaterialAmbientColor = lights.ambient_color.rgb;
    let MaterialDiffuseColor = albedo_spec.rgb;
    let MaterialSpecularColor = vec3(0.4, 0.4, 0.4);

    // Normal of the computed fragment, in camera space
    let n = normalize(in.view_space_normal);

    let eyeCoords = in.view_pos.xyz;

    // unlit
    if (in.material == 1) {
        return vec4(MaterialAmbientColor * MaterialDiffuseColor, albedo_spec.a);
    }
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
        let light_pos_view = (view_proj_uniforms.view * vec4(lights.items[i].position, 1.0)).xyz - eyeCoords;
        let l = normalize(light_pos_view);

        // Cosine of the angle between the normal and the light direction, 
        // clamped above 0
        //  - light is at the vertical of the triangle -> 1
        //  - light is perpendicular to the triangle -> 0
        //  - light is behind the triangle -> 0
        let cosTheta = clamp(dot(n, l), 0.0, 1.0);

        // Eye vector (towards the camera)
        let E = normalize(-eyeCoords);
        // Direction in which the triangle reflects the light
        let R = reflect(-l, n);
        // Cosine of the angle between the Eye vector and the Reflect vector,
        // clamped to 0
        //  - Looking into the reflection -> 1
        //  - Looking elsewhere -> < 1
        let cosAlpha = clamp(dot(E, R), 0.0, 1.0);

        var bias = lights.shadow_bias_factor * tan(acos(cosTheta));
        // bias = clamp(bias, 0.0, 0.01);
        // var bias = 0.0001;

        let shadow_pos = lights.items[i].view_proj * in.world_pos;

        let flip_correction = vec2<f32>(0.5, -0.5);
        let proj_correction = 1.0 / shadow_pos.w;

        let ShadowCoord = shadow_pos.xy * flip_correction * proj_correction + vec2<f32>(0.5, 0.5);
        var occlusion = 0.0;

        var weight = 0.0;
        for (var x = -lights.shadow_blur_half_kernel_size; x <= lights.shadow_blur_half_kernel_size; x++) {
            for (var y = -lights.shadow_blur_half_kernel_size; y <= lights.shadow_blur_half_kernel_size; y++) {
                let s = textureSampleCompare(shadow_map, shadow_map_sampler, ShadowCoord.xy + vec2(f32(x), f32(y)) * texelSize, i, (shadow_pos.z) / shadow_pos.w - bias);
                occlusion += 1.0 - s;
                weight += 1.0;
            }
        }
        occlusion /= weight;

        visibility = clamp(visibility - occlusion, 0.0, 1.0);

        total_light +=
                visibility * MaterialDiffuseColor * LightColor * LightPower * cosTheta +
                visibility * MaterialSpecularColor * LightColor * LightPower * pow(cosAlpha, 5.0);
    }

    return vec4(total_light.xyz, albedo_spec.a);
}"#
);

define_bind_group! {
    pub LightingGroup {
        pub lights: UniformBuffer<LightsUniform>,
        pub shadow_maps_view: DepthTextureArrayView,
        pub shadow_maps_sampler: ComparisonSampler,
    }
}

pub struct ForwardGeometryPass {
    pipeline: Pipeline<(ModelVertexData, InstanceDataWithNormalMatrix)>,
    depth_only_pipeline: Pipeline<(ModelVertexData, InstanceDataWithNormalMatrix)>,
    pub color_target: BoundTexture,
    pub depth_target_view: DepthTextureView,
    pub depth_target_sampler: TextureSampler,
    pub lighting_group: BindGroup<LightingGroup>,
    view_proj_bind_group: UniformBindGroup<ViewProjectionUniforms>,

    // staging_belt: wgpu::util::StagingBelt,
    // instance_buffer: wgpu::Buffer,
    instance_storage: InstanceStorage,
}

impl ForwardGeometryPass {
    pub fn new(
        state: &mut RenderState,
        display: &Display,
        size: Point<u32>,
        shadow_maps_view: wgpu::TextureView,
        shadow_maps_sampler: wgpu::Sampler,
    ) -> Self {
        let color_target = TextureBuilder::render_target()
            .with_label("color_target")
            .with_usage(TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING)
            .build(display.device(), size);
        let depth_target = TextureBuilder::depth()
            .with_address_mode(wgpu::AddressMode::ClampToBorder)
            .with_border_color(wgpu::SamplerBorderColor::OpaqueWhite)
            .with_compare_func(None)
            .with_usage(
                wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            )
            .build(display.device(), size);
        let depth_stencil_state = Some(wgpu::DepthStencilState {
            format: depth_target.format(),
            depth_write_enabled: Some(true),
            depth_compare: Some(wgpu::CompareFunction::LessEqual),
            stencil: Default::default(),
            bias: Default::default(),
        });
        let global_uniform_bgl =
            UniformBuffer::<GlobalUniforms>::create_layout(display.device(), "global uniforms");
        let view_proj_uniform_bgl = UniformBuffer::<ViewProjectionUniforms>::create_layout(
            display.device(),
            "view proj uniform bg layout",
        );
        let depth_only_pipeline = PipelineBuilder::new("Forward Rendering (Depth Prepass)")
            .with_color_target_states(vec![])
            .with_depth_stencil_state(depth_stencil_state.clone())
            .with_bind_group_layouts(vec![&view_proj_uniform_bgl])
            .build(
                display.device(),
                &create_shader::<
                    (GlobalUniforms, ViewProjectionUniforms),
                    (ModelVertexData, InstanceDataWithNormalMatrix),
                >(display, "depth_only", DEPTH_ONLY_SHADER.to_string()),
            );
        let occlusion_map_layout = MaterialGroup::create_layout(display.device(), "occlusion map");
        let lighting_group = BindGroup::new(
            display.device(),
            LightingGroup {
                lights: UniformBuffer::new(display.device(), LightsUniform::default()),
                shadow_maps_view: shadow_maps_view.into(),
                shadow_maps_sampler: shadow_maps_sampler.into(),
            },
        );
        let view_proj_bind_group =
            create_uniform_bind_group(display.device(), ViewProjectionUniforms::default());
        let pipeline = PipelineBuilder::new("Forward Rendering")
            .with_color_target_states(vec![Some(wgpu::ColorTargetState {
                format: color_target.format(),
                blend: Some(PipelineBuilder::DEFAULT_BLEND),
                write_mask: wgpu::ColorWrites::ALL,
            })])
            .with_depth_stencil_state(depth_stencil_state)
            .with_bind_group_layouts(vec![
                state.texture_array_bgl.as_ref().unwrap(),
                &global_uniform_bgl,
                &view_proj_uniform_bgl,
                lighting_group.layout(),
                &occlusion_map_layout,
            ])
            .build(
                display.device(),
                &create_shader::<
                    (
                        GlobalUniforms,
                        ViewProjectionUniforms,
                        LightRaw,
                        LightingUniformsRaw,
                    ),
                    (ModelVertexData, InstanceDataWithNormalMatrix),
                >(display, "forward", FORWARD_LIGHTING_SHADER.to_string()),
            );
        let color_target = state.bind_texture(display, color_target);
        let instance_storage = InstanceStorage::new(display, 512);
        Self {
            pipeline,
            depth_only_pipeline,
            color_target,
            depth_target_view: depth_target.view.into(),
            depth_target_sampler: depth_target.sampler.into(),
            lighting_group,
            view_proj_bind_group,
            instance_storage,
            // staging_belt,
            // instance_buffer,
        }
    }

    pub fn depth_prepass(
        &mut self,
        state: &mut RenderState,
        display: &Display,
        view_projection: &ViewProjectionUniforms,
        scene: &[InstanceRenderData<ModelVertexData, InstanceDataWithNormalMatrix>],
    ) {
        self.view_proj_bind_group
            .update(display.queue(), *view_projection);
        self.instance_storage.clear();
        for o in scene {
            self.instance_storage.add(&o.instance);
        }
        self.instance_storage.update_buffer(display);
        state
            .render_pass(
                &display,
                "Depth Pre-Pass",
                &[],
                Some(&self.depth_target_view),
                |r| {
                    let mut draw_offset = 0;
                    r.set_pipeline(&self.depth_only_pipeline);
                    r.set_bind_group(0, self.view_proj_bind_group.bind_group(), &[]);
                    r.set_vertex_buffer(1, self.instance_storage.buffer().slice(..));
                    // for (mesh, group) in &scene.iter().group_by(|i| i.mesh) {
                    //     let mut b = r.draw_instanced(mesh, &self.depth_only_pipeline);
                    //     for object in group {
                    //         b.add(object);
                    //     }
                    // }
                    for (mesh, group) in &scene.iter().group_by(|i| i.mesh) {
                        let num_indices = {
                            let m = r.render_state.get_mesh(mesh);
                            r.set_vertex_buffer(0, m.vertex_buffer.slice(..));
                            r.set_index_buffer(m.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                            m.num_indices
                        };
                        let start = draw_offset;
                        draw_offset += group.count() as u32;
                        r.draw_indexed(0..num_indices, 0, start..draw_offset);
                    }
                },
            )
            .submit();
    }

    pub fn run(
        &mut self,
        state: &mut RenderState,
        display: &Display,
        view_projection: &ViewProjectionUniforms,
        scene: &[InstanceRenderData<ModelVertexData, InstanceDataWithNormalMatrix>],
        occlusion_map: Option<wgpu::BindGroup>,
    ) {
        if scene.len() == 0 {
            return;
        }
        let occlusion_map = occlusion_map.unwrap_or_else(|| state.default_texture().clone());
        self.view_proj_bind_group
            .update(display.queue(), *view_projection);
        state
            .render_pass(
                &display,
                "Forward Rendering Pass",
                &[&self.color_target.resource.view],
                Some(&self.depth_target_view),
                |r| {
                    let mut draw_offset = 0;
                    r.set_pipeline(&self.pipeline);
                    r.set_vertex_buffer(1, self.instance_storage.buffer().slice(..));
                    let global_uniforms = r.render_state.global_uniforms.bind_group().clone();
                    let ta = r.render_state.texture_array.as_ref().unwrap().clone();
                    r.set_bind_group(0, &ta, &[]);
                    r.set_bind_group(1, &global_uniforms, &[]);
                    r.set_bind_group(2, self.view_proj_bind_group.bind_group(), &[]);
                    r.set_bind_group(3, self.lighting_group.bind_group(), &[]);
                    r.set_bind_group(4, &occlusion_map, &[]);
                    for (mesh, group) in &scene.iter().group_by(|i| i.mesh) {
                        let num_indices = {
                            let m = r.render_state.get_mesh(mesh);
                            r.set_vertex_buffer(0, m.vertex_buffer.slice(..));
                            r.set_index_buffer(m.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                            m.num_indices
                        };
                        let start = draw_offset;
                        draw_offset += group.count() as u32;
                        r.draw_indexed(0..num_indices, 0, start..draw_offset);
                    }
                },
            )
            .submit();
    }
}
