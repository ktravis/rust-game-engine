use std::ops::Deref;

use bytemuck::Zeroable;
use glam::{vec2, vec4, Vec2, Vec4};

use crate::{
    camera::Camera,
    geom::{BasicVertexData, Point},
};

use super::{
    instance::InstanceRenderData,
    shaders::{self, ssao_from_depth as shader},
    state::{BoundTexture, ViewProjectionUniforms},
    BasicInstanceData, Bindable, Display, PipelineRef, RenderState, RenderTarget, Texture,
    TextureBuilder, TextureRef, UniformBindGroup,
};

type SSAOKernel = shaders::ssao_from_depth::types::Kernel;

impl SSAOKernel {
    const SIZE: usize = shaders::ssao_from_depth::constants::KERNEL_SIZE::VALUE as _;
    const DEFAULT_RADIUS: f32 = 0.3;
    const DEFAULT_BIAS: f32 = 0.025;

    fn generate_items() -> [Vec4; Self::SIZE] {
        std::array::from_fn(|i| {
            let scale = i as f32 / Self::SIZE as f32;
            let v = rand::random::<f32>()
                * vec4(
                    2.0 * rand::random::<f32>() - 1.0,
                    2.0 * rand::random::<f32>() - 1.0,
                    rand::random::<f32>(),
                    0.0,
                )
                .normalize();
            v * (0.05 + 0.95 * scale * scale)
        })
    }

    fn new(noise_texture_scale: Vec2, camera: &Camera) -> Self {
        let items = Self::generate_items();
        let aspect_ratio = camera.aspect_ratio();
        let tan_half_fov = (camera.fov_radians() / 2.0).tan();
        Self {
            items,
            radius: Self::DEFAULT_RADIUS,
            bias: Self::DEFAULT_BIAS,
            noise_texture_scale,
            aspect_ratio,
            tan_half_fov,
            inverse_proj: camera.perspective_matrix().inverse(),
            ..Zeroable::zeroed()
        }
    }
}

pub type BlurUniforms = shaders::ssao_blur::types::BlurUniforms;

impl Default for BlurUniforms {
    fn default() -> Self {
        Self {
            half_kernel_size: 2,
            sharpness: 40.0,
            step: Vec2::ZERO,
        }
    }
}

pub struct SSAOPass {
    pipeline: PipelineRef<BasicVertexData, BasicInstanceData>,
    output_texture: TextureRef,
    kernel: UniformBindGroup<SSAOKernel>,
    noise_texture: BoundTexture,
    blur_enabled: bool,
    blur_uniforms: UniformBindGroup<BlurUniforms>,
    blur_pipeline: PipelineRef<BasicVertexData, BasicInstanceData>,
    blur_temp_buffer: TextureRef,
    depth_buffer_bind_group: wgpu::BindGroup,
    buffer_size: Point<u32>,
}

impl SSAOPass {
    pub const OCCLUSION_MAP_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R16Float;

    const NOISE_SCALE: usize = 4;

    pub fn new(
        state: &mut RenderState,
        display: &Display,
        size: Point<u32>,
        depth_target: &Texture,
        camera: &Camera,
    ) -> Self {
        let noise: [Vec4; Self::NOISE_SCALE * Self::NOISE_SCALE] = std::array::from_fn(|_| {
            vec4(
                2.0 * rand::random::<f32>() - 1.0,
                2.0 * rand::random::<f32>() - 1.0,
                0.0,
                1.0,
            )
        });
        let noise_texture = TextureBuilder::labeled("ssao_noise")
            .with_usage(wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST)
            .with_format(wgpu::TextureFormat::Rgba32Float)
            .with_address_mode(wgpu::AddressMode::Repeat)
            .from_raw_bytes(
                display.device(),
                display.queue(),
                bytemuck::bytes_of(&noise),
                Point::new(Self::NOISE_SCALE as _, Self::NOISE_SCALE as _),
            );
        let noise_texture_bgl =
            state.bind_group_layout(display.device(), noise_texture.binding_type());

        let noise_texture = BoundTexture::new(display.device(), &noise_texture_bgl, noise_texture);
        let (kernel, uniform_bgl) = state.create_uniform_bind_group(
            display.device(),
            SSAOKernel::new(size.as_vec2() / Self::NOISE_SCALE as f32, camera),
        );
        let output_texture = state.load_texture(
            &display,
            TextureBuilder::render_target()
                .with_label("ssao")
                .with_format(Self::OCCLUSION_MAP_FORMAT)
                .with_filter_mode(wgpu::FilterMode::Linear)
                .with_usage(
                    wgpu::TextureUsages::TEXTURE_BINDING
                        | wgpu::TextureUsages::COPY_DST
                        | wgpu::TextureUsages::RENDER_ATTACHMENT,
                )
                .build(display.device(), size),
        );
        let depth_buffer_bgl = display
            .device()
            .create_bind_group_layout(&shaders::ssao_from_depth::globals::group3::layout());
        let depth_buffer_bind_group =
            display
                .device()
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("forward pass depth target"),
                    layout: &depth_buffer_bgl,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&depth_target.view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&depth_target.sampler),
                        },
                    ],
                });
        let pipeline = state
            .pipeline_builder()
            .with_label("SSAO Pipeline")
            .with_extra_bind_group_layouts(vec![
                &depth_buffer_bgl,
                &uniform_bgl,
                &noise_texture_bgl,
            ])
            .with_color_target_states(vec![Some(wgpu::ColorTargetState {
                blend: None,
                format: Self::OCCLUSION_MAP_FORMAT,
                write_mask: wgpu::ColorWrites::ALL,
            })])
            .with_depth_stencil_state(None)
            .build(
                display.device(),
                &display
                    .device()
                    .create_shader_module(shaders::ssao_from_depth::DESCRIPTOR.clone()),
            );
        let blur_pipeline = state
            .pipeline_builder()
            .with_label("SSAO Blur Pipeline")
            .with_color_target_states(vec![Some(wgpu::ColorTargetState {
                blend: None,
                format: Self::OCCLUSION_MAP_FORMAT,
                write_mask: wgpu::ColorWrites::ALL,
            })])
            .with_extra_bind_group_layouts(vec![&depth_buffer_bgl, &uniform_bgl])
            .with_depth_stencil_state(None)
            .build(
                display.device(),
                &display
                    .device()
                    .create_shader_module(shaders::ssao_blur::DESCRIPTOR.clone()),
            );
        let blur_temp_buffer = state.load_texture(
            display,
            TextureBuilder::render_target()
                .with_label("blurred_ssao")
                .with_format(Self::OCCLUSION_MAP_FORMAT)
                .with_filter_mode(wgpu::FilterMode::Linear)
                .with_usage(
                    wgpu::TextureUsages::TEXTURE_BINDING
                        | wgpu::TextureUsages::COPY_DST
                        | wgpu::TextureUsages::RENDER_ATTACHMENT,
                )
                .build(display.device(), size),
        );

        let (blur_uniforms, _) =
            state.create_uniform_bind_group(display.device(), Default::default());
        Self {
            pipeline,
            output_texture,
            kernel,
            noise_texture,
            blur_pipeline,
            blur_uniforms,
            blur_enabled: true,
            blur_temp_buffer,
            depth_buffer_bind_group,
            buffer_size: size,
        }
    }

    pub fn run(
        &mut self,
        state: &mut RenderState,
        display: &Display,
        view_projection: &ViewProjectionUniforms,
    ) -> TextureRef {
        let mut u = *self.kernel.uniform();
        u.inverse_proj = view_projection.projection.inverse();
        self.kernel.update(display.queue(), u);
        let quad = state.quad_mesh();
        state
            .render_pass(
                &display,
                "SSAO Pass",
                &[RenderTarget::TextureRef(self.output_texture)],
                None,
                view_projection,
                |r| {
                    use shader::globals::*;
                    r.set_bind_group(depth_buffer::GROUP, &self.depth_buffer_bind_group, &[]);
                    r.set_bind_group(kernel::GROUP, self.kernel.bind_group().deref(), &[]);
                    r.set_bind_group(
                        ssao_noise::GROUP,
                        self.noise_texture.bind_group().deref(),
                        &[],
                    );
                    r.draw_instance(&InstanceRenderData {
                        mesh: quad,
                        instance: BasicInstanceData::default(),
                        texture: None,
                        pipeline: Some(self.pipeline),
                    });
                },
            )
            .submit();
        if self.blur_enabled {
            self.blur_uniforms.update_with(display.queue(), |s| {
                s.step = vec2(1.0 / self.buffer_size.x as f32, 0.0);
            });
            state
                .render_pass(
                    display,
                    "SSAO Blur Pass - X",
                    &[RenderTarget::TextureRef(self.blur_temp_buffer)],
                    None,
                    &ViewProjectionUniforms {
                        // projection: display_view.orthographic_projection(),
                        ..Default::default()
                    },
                    |r| {
                        use shaders::ssao_blur::globals::*;
                        r.set_bind_group(depth_buffer::GROUP, &self.depth_buffer_bind_group, &[]);
                        r.set_bind_group(
                            blur_settings::GROUP,
                            self.blur_uniforms.bind_group().deref(),
                            &[],
                        );
                        r.draw_instance(&InstanceRenderData {
                            mesh: quad,
                            instance: BasicInstanceData {
                                ..Default::default()
                            },
                            texture: Some(self.output_texture),
                            pipeline: Some(self.blur_pipeline),
                        });
                    },
                )
                .submit();
            self.blur_uniforms.update_with(display.queue(), |s| {
                s.step = vec2(0.0, 1.0 / self.buffer_size.y as f32);
            });
            state
                .render_pass(
                    display,
                    "SSAO Blur Pass - Y",
                    &[RenderTarget::TextureRef(self.output_texture)],
                    None,
                    &ViewProjectionUniforms {
                        // projection: display_view.orthographic_projection(),
                        ..Default::default()
                    },
                    |r| {
                        use shaders::ssao_blur::globals::*;
                        r.set_bind_group(depth_buffer::GROUP, &self.depth_buffer_bind_group, &[]);
                        r.set_bind_group(
                            blur_settings::GROUP,
                            self.blur_uniforms.bind_group().deref(),
                            &[],
                        );
                        r.draw_instance(&InstanceRenderData {
                            mesh: quad,
                            instance: BasicInstanceData {
                                ..Default::default()
                            },
                            texture: Some(self.blur_temp_buffer),
                            pipeline: Some(self.blur_pipeline),
                        });
                    },
                )
                .submit();
        }
        self.output_texture
    }

    pub fn debug_ui(&mut self, ui: &mut egui::Ui) {
        ui.add(egui::Slider::new(&mut self.kernel.radius, 0.0..=5.0).text("radius"));
        ui.add(egui::Slider::new(&mut self.kernel.bias, 0.0..=2.0).text("bias"));
        if ui.add(egui::Button::new("Regenerate")).clicked() {
            self.kernel.items = SSAOKernel::generate_items();
        }

        ui.separator();
        ui.label("Blur");
        ui.add(egui::Checkbox::new(&mut self.blur_enabled, "enabled"));
        ui.add(
            egui::Slider::new(&mut self.blur_uniforms.half_kernel_size, 0..=10)
                .text("half kernel size"),
        );
        ui.add(
            egui::Slider::new(&mut self.blur_uniforms.sharpness, 0.0..=100.0)
                .text("edge sharpness"),
        );
    }
}
