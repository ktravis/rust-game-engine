use std::sync::Arc;

use glam::{vec4, Vec2, Vec4};
use wgpu::{include_wgsl, BindGroupLayout};

use crate::geom::{BasicVertexData, Point};

use super::{
    instance::InstanceRenderData,
    state::{BoundTexture, ViewProjectionUniforms},
    BasicInstanceData, Display, PipelineRef, RenderState, RenderTarget, TextureBuilder, TextureRef,
    UniformBindGroup,
};

#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct SSAOKernel {
    items: [Vec4; SSAOKernel::SIZE],
    count: u32,
    radius: f32,
    bias: f32,
    noise_texture_scale: Vec2,
}

impl SSAOKernel {
    const SIZE: usize = 64;
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
            v * (0.1 + 0.9 * scale * scale)
        })
    }

    fn new(noise_texture_scale: Vec2) -> Self {
        let items = Self::generate_items();
        Self {
            items,
            count: Self::SIZE as u32,
            radius: Self::DEFAULT_RADIUS,
            bias: Self::DEFAULT_BIAS,
            noise_texture_scale,
        }
    }
}

impl UniformBindGroup<SSAOKernel> {
    pub fn debug_ui(&mut self, display: &Display, ui: &mut egui::Ui) {
        ui.add(egui::Slider::new(&mut self.radius, 0.0..=5.0).text("radius"));
        ui.add(egui::Slider::new(&mut self.bias, 0.0..=2.0).text("bias"));
        let mut u = *self.uniform();
        if ui.add(egui::Button::new("Regenerate")).clicked() {
            u.items = SSAOKernel::generate_items();
        }
        self.update(display.queue(), u);
    }
}

pub struct SSAOPass {
    pipeline: PipelineRef<BasicVertexData, BasicInstanceData>,
    // blur_pipeline: PipelineRef<BasicVertexData, BasicInstanceData>,
    // blur_settings: UniformBindGroup<BlurUniforms>,
    output_texture: TextureRef,
    // temp_texture: TextureRef,
    kernel: UniformBindGroup<SSAOKernel>,
    _noise_texture_bgl: wgpu::BindGroupLayout,
    noise_texture: BoundTexture,
}

impl SSAOPass {
    const NOISE_SCALE: usize = 4;
    pub fn new(
        state: &mut RenderState,
        display: &Display,
        geometry_pass_bgl: &BindGroupLayout,
    ) -> Self {
        let fb_size = display.size_pixels();
        let noise: [Vec4; Self::NOISE_SCALE * Self::NOISE_SCALE] = std::array::from_fn(|_| {
            vec4(
                2.0 * rand::random::<f32>() - 1.0,
                2.0 * rand::random::<f32>() - 1.0,
                0.0,
                1.0,
            )
        });
        let uniform_bgl =
            &state.bind_group_layout(display.device(), super::state::BindingType::Uniform);
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
            display
                .device()
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                            count: None,
                        },
                    ],
                });
        let noise_texture = BoundTexture::new(display.device(), &noise_texture_bgl, noise_texture);
        let mut kernel = state.create_uniform_bind_group(
            display.device(),
            SSAOKernel::new(fb_size.as_vec2() / Self::NOISE_SCALE as f32),
        );
        let u = *kernel.uniform();
        kernel.update(display.queue(), u);
        let output_texture = state.load_texture(
            &display,
            TextureBuilder::render_target()
                .with_label("ssao")
                .with_format(wgpu::TextureFormat::R16Float)
                .with_usage(
                    wgpu::TextureUsages::TEXTURE_BINDING
                        | wgpu::TextureUsages::COPY_DST
                        | wgpu::TextureUsages::RENDER_ATTACHMENT,
                )
                .build(display.device(), fb_size),
        );
        let pipeline = state
            .pipeline_builder()
            .with_label("SSAO Pipeline")
            // .with_key(self.render_pipelines.ssao)
            .with_extra_bind_group_layouts(vec![geometry_pass_bgl, uniform_bgl, &noise_texture_bgl])
            .with_color_target_states(vec![Some(wgpu::ColorTargetState {
                blend: None,
                format: wgpu::TextureFormat::R16Float,
                write_mask: wgpu::ColorWrites::ALL,
            })])
            .with_depth_stencil_state(None)
            .build(
                display.device(),
                &display
                    .device()
                    .create_shader_module(include_wgsl!("../../res/shaders/ssao.wgsl")),
            );
        Self {
            pipeline,
            output_texture,
            kernel,
            _noise_texture_bgl: noise_texture_bgl,
            noise_texture,
        }
    }

    pub fn run(
        &mut self,
        state: &mut RenderState,
        display: &Display,
        view_projection: &ViewProjectionUniforms,
        geometry_pass_bg: Arc<wgpu::BindGroup>,
    ) -> TextureRef {
        let quad = state.quad_mesh();
        state
            .render_pass(
                &display,
                "SSAO Pass",
                &[RenderTarget::TextureRef(self.output_texture)],
                None,
                view_projection,
                |r| {
                    r.set_bind_group(3, geometry_pass_bg);
                    r.set_bind_group(4, self.kernel.bind_group().clone());
                    r.set_bind_group(5, self.noise_texture.bind_group().clone());
                    r.draw_instance(&InstanceRenderData {
                        mesh: quad,
                        instance: BasicInstanceData::default(),
                        texture: None,
                        pipeline: Some(self.pipeline),
                    });
                },
            )
            .submit();
        self.output_texture
    }

    pub fn kernel_mut(&mut self) -> &mut UniformBindGroup<SSAOKernel> {
        &mut self.kernel
    }
}
