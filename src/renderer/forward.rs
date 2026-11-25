use glam::Vec3;
use wgpu::TextureUsages;

use crate::{
    create_shader_ex, define_bind_group,
    geom::{ModelVertexData, Point},
    renderer::{
        bindings::{
            create_uniform_bind_group, BindGroup, Bindable, ComparisonSampler,
            DepthTextureArrayView, DepthTextureView, MaterialGroup, TextureBindingArray,
            TextureSampler, UniformBindGroup, UniformBuffer,
        },
        instance::{InstanceDataWithNormalMatrix, ObjectData},
        shader_type::GlobalUniforms,
        state::{write_scene_draw_calls, BoundTexture, ColorTarget, TextureArray},
    },
    transform::Transform,
};

use super::{
    bindings::ViewProjectionUniforms, lighting::LightsUniform, Display, Pipeline, PipelineBuilder,
    RenderState, TextureBuilder,
};

const SKYBOX_SHADER: &'static str = crate::wgsl!(
    r#"
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
}

@vertex
fn vs_main(
    vertex: ModelVertexData,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4(2.0 * (vertex.position.xy - vec2(0.5, 0.5)), 0.0, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4(lighting_group_skybox_color, 1.0);
}"#
);

const DEPTH_ONLY_SHADER: &'static str = crate::wgsl!(
    r#"
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

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let view_pos = in.view_pos;
    let albedo_spec = in.tint_color * textureSample(texture_array_views[in.texture], texture_array_samplers[in.texture], in.tex_coords);

    let texelSize = 1.0 / f32(textureDimensions(lighting_group_shadow_maps_view).x);

    var total_light = vec3(0.0, 0.0, 0.0);

    // Material properties
    // TODO: this ignores tint alpha
    let MaterialAmbientColor = lighting_group_lights.ambient_color.rgb;
    let MaterialDiffuseColor = albedo_spec.rgb;
    let MaterialSpecularColor = vec3(0.4, 0.4, 0.4);

    // Normal of the computed fragment, in camera space
    let n = normalize(in.view_space_normal);

    let eyeCoords = in.view_pos.xyz;

    // unlit
    if (in.material == 1) {
        return vec4(MaterialAmbientColor * MaterialDiffuseColor, albedo_spec.a);
    }
    let ao = textureSample(occlusion_map_view, occlusion_map_sampler, in.clip_position.xy / global_uniforms.screen_size).r;

    total_light += ao * MaterialAmbientColor * MaterialDiffuseColor;

    for (var i = 0u; i < lighting_group_lights.count; i++) {
        var visibility = 1.0;

        if lighting_group_lights.items[i].kind == 1 { // spot light
            let light_to_fragment = in.world_pos.xyz - lighting_group_lights.items[i].position;
            let light_dist_sqr = dot(light_to_fragment, light_to_fragment);
            let spot_factor = dot(normalize(light_to_fragment), lighting_group_lights.items[i].direction);
            let reach_sqr = pow(lighting_group_lights.items[i].reach, 2.0);
            if spot_factor > lighting_group_lights.items[i].radius && light_dist_sqr < reach_sqr {
                visibility = (1.0 - (1.0 - spot_factor) * 1.0 / (1.0 - lighting_group_lights.items[i].radius));
            } else {
                visibility = 0.0;
            }
        }

        let LightColor = lighting_group_lights.items[i].color.rgb;
        let LightPower = lighting_group_lights.items[i].color.a;
        // Direction of the light (from the fragment to the light)
        let light_pos_view = (view_proj_uniforms.view * vec4(lighting_group_lights.items[i].position, 1.0)).xyz - eyeCoords;
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

        var bias = lighting_group_lights.shadow_bias_factor * tan(acos(cosTheta));
        // bias = clamp(bias, 0.0, 0.01);
        // var bias = 0.0001;

        let shadow_pos = lighting_group_lights.items[i].view_proj * in.world_pos;

        let flip_correction = vec2<f32>(0.5, -0.5);
        let proj_correction = 1.0 / shadow_pos.w;

        let ShadowCoord = shadow_pos.xy * flip_correction * proj_correction + vec2<f32>(0.5, 0.5);
        var occlusion = 0.0;

        var weight = 0.0;
        for (var x = -lighting_group_lights.shadow_blur_half_kernel_size; x <= lighting_group_lights.shadow_blur_half_kernel_size; x++) {
            for (var y = -lighting_group_lights.shadow_blur_half_kernel_size; y <= lighting_group_lights.shadow_blur_half_kernel_size; y++) {
                let s = textureSampleCompare(lighting_group_shadow_maps_view, lighting_group_shadow_maps_sampler, ShadowCoord.xy + vec2(f32(x), f32(y)) * texelSize, i, (shadow_pos.z) / shadow_pos.w - bias);
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
        pub skybox_color: UniformBuffer<Vec3>,
    }
}

// TODO: factor out lighting_group?
pub struct ForwardGeometryPass {
    pipeline: Pipeline<(ModelVertexData, InstanceDataWithNormalMatrix)>,
    depth_only_pipeline: Pipeline<(ModelVertexData, InstanceDataWithNormalMatrix)>,
    skybox_pipeline: Pipeline<ModelVertexData>,
    pub color_target: BoundTexture,
    pub depth_target_view: DepthTextureView,
    pub depth_target_sampler: TextureSampler,
    pub lighting_group: BindGroup<LightingGroup>,
    view_proj_bind_group: UniformBindGroup<ViewProjectionUniforms>,
    indirect_command_buffer: wgpu::Buffer,
}

const CMD_BUFFER_SIZE: u64 =
    64000 * std::mem::size_of::<wgpu::util::DrawIndexedIndirectArgs>() as u64;

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
                &create_shader_ex!(display, {
                    name => "depth_only",
                    src => DEPTH_ONLY_SHADER.to_string(),
                    inputs => (ModelVertexData, InstanceDataWithNormalMatrix),
                    bind_groups => {
                        view_proj_uniforms => UniformBuffer<ViewProjectionUniforms>,
                    }
                }),
            );
        let occlusion_map_layout = MaterialGroup::create_layout(display.device(), "occlusion map");
        let lighting_group = BindGroup::new(
            display.device(),
            LightingGroup {
                lights: UniformBuffer::new(display.device(), LightsUniform::default()),
                shadow_maps_view: shadow_maps_view.into(),
                shadow_maps_sampler: shadow_maps_sampler.into(),
                skybox_color: UniformBuffer::new(display.device(), Vec3::new(0.4, 0.4, 0.6)),
            },
        );
        let skybox_pipeline = PipelineBuilder::new("Skybox")
            .with_color_target_states(vec![Some(wgpu::ColorTargetState {
                format: color_target.format(),
                blend: Some(PipelineBuilder::DEFAULT_BLEND),
                write_mask: wgpu::ColorWrites::ALL,
            })])
            .with_depth_stencil_state(None)
            .with_bind_group_layouts(vec![&view_proj_uniform_bgl, lighting_group.layout()])
            .build(
                display.device(),
                &create_shader_ex!(display, {
                    name => "skybox",
                    src => SKYBOX_SHADER.to_string(),
                    inputs => ModelVertexData,
                    bind_groups => {
                        view_proj_uniforms => UniformBuffer<ViewProjectionUniforms>,
                        lighting_group => LightingGroup,
                    }
                }),
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
                &TextureArray::<'_>::create_layout(display.device(), "texture array"),
                &global_uniform_bgl,
                &view_proj_uniform_bgl,
                lighting_group.layout(),
                &occlusion_map_layout,
            ])
            .build(
                display.device(),
                &create_shader_ex!(display, {
                    name => "forward",
                    src => FORWARD_LIGHTING_SHADER.to_string(),
                    inputs => (ModelVertexData, InstanceDataWithNormalMatrix),
                    bind_groups => {
                        texture_array => TextureBindingArray<128>,
                        global_uniforms => UniformBuffer<GlobalUniforms>,
                        view_proj_uniforms => UniformBuffer<ViewProjectionUniforms>,
                        lighting_group => LightingGroup,
                        occlusion_map => MaterialGroup,
                    }
                }),
            );
        let color_target = state.bind_texture(display, color_target);

        let indirect_command_buffer = display.device().create_buffer(&wgpu::BufferDescriptor {
            label: Some("forward pass command buffer"),
            size: CMD_BUFFER_SIZE,
            usage: wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Self {
            pipeline,
            depth_only_pipeline,
            skybox_pipeline,
            color_target,
            depth_target_view: depth_target.view.into(),
            depth_target_sampler: depth_target.sampler.into(),
            lighting_group,
            view_proj_bind_group,
            indirect_command_buffer,
        }
    }

    pub fn depth_prepass<T: Transform>(
        &mut self,
        state: &mut RenderState,
        display: &Display,
        view_projection: &ViewProjectionUniforms,
        scene: &[ObjectData<T>],
        instance_buffer: &wgpu::Buffer,
    ) {
        self.view_proj_bind_group
            .update(display.queue(), *view_projection);
        let count = write_scene_draw_calls(display, &self.indirect_command_buffer, scene, false);
        state
            .render_pass(
                &display,
                "Depth Pre-Pass",
                &[],
                Some(&self.depth_target_view),
                |r| {
                    r.set_pipeline(&self.depth_only_pipeline);
                    r.set_bind_group(0, self.view_proj_bind_group.bind_group(), &[]);
                    r.set_vertex_buffer(1, instance_buffer.slice(..));
                    r.multi_draw_indexed_indirect(&self.indirect_command_buffer, 0, count);
                },
            )
            .submit();
    }

    pub fn run<T: Transform>(
        &mut self,
        state: &mut RenderState,
        display: &Display,
        view_projection: &ViewProjectionUniforms,
        scene: &[ObjectData<T>],
        occlusion_map: Option<wgpu::BindGroup>,
        instance_buffer: &wgpu::Buffer,
    ) {
        if scene.len() == 0 {
            return;
        }
        let occlusion_map =
            occlusion_map.unwrap_or_else(|| state.default_texture_bind_group().clone());
        self.view_proj_bind_group
            .update(display.queue(), *view_projection);
        state
            .render_pass(
                &display,
                "Skybox pass",
                &[ColorTarget::from(&self.color_target.resource.view)],
                None,
                |r| {
                    r.set_pipeline(&self.skybox_pipeline);
                    r.set_bind_group(0, self.view_proj_bind_group.bind_group(), &[]);
                    r.set_bind_group(1, self.lighting_group.bind_group(), &[]);
                    r.draw_scene_mesh(r.render_state.quad_mesh(), 0..1);
                },
            )
            .submit();
        let count = write_scene_draw_calls(display, &self.indirect_command_buffer, scene, true);
        state
            .render_pass(
                &display,
                "Forward Rendering Pass",
                &[ColorTarget::WithOps {
                    view: &self.color_target.resource.view,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                }],
                Some(&self.depth_target_view),
                |r| {
                    r.set_pipeline(&self.pipeline);
                    r.set_vertex_buffer(1, instance_buffer.slice(..));
                    let global_uniforms = r.render_state.global_uniforms.bind_group().clone();
                    let ta = r.render_state.texture_array_bind_group().clone();
                    r.set_bind_group(0, &ta, &[]);
                    r.set_bind_group(1, &global_uniforms, &[]);
                    r.set_bind_group(2, self.view_proj_bind_group.bind_group(), &[]);
                    r.set_bind_group(3, self.lighting_group.bind_group(), &[]);
                    r.set_bind_group(4, &occlusion_map, &[]);
                    r.multi_draw_indexed_indirect(&self.indirect_command_buffer, 0, count);
                },
            )
            .submit();
    }
}
