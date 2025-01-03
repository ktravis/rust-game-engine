use std::{ops::Deref, sync::Arc};

use glam::{Mat4, Vec4};
use wgpu::include_wgsl;

use crate::geom::BasicVertexData;

use super::{
    instance::InstanceRenderData, state::ViewProjectionUniforms, BasicInstanceData, Display,
    PipelineRef, RenderState, RenderTarget, TextureRef, UniformBindGroup, UniformData,
};

pub const MAX_LIGHTS: usize = 8;

#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
struct LightRaw {
    position: Vec4,
    color: Vec4,
    view_proj: Mat4,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Light {
    pub position: Vec4,
    pub color: Vec4,
    pub view: Mat4,
    pub proj: Mat4,
}

impl From<Light> for LightRaw {
    fn from(value: Light) -> Self {
        LightRaw {
            position: value.position,
            color: value.color,
            view_proj: value.proj * value.view,
        }
    }
}

#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct LightingUniformsRaw {
    items: [LightRaw; MAX_LIGHTS],
    count: u32,
    // shadow_bias_minimum: f32,
    // shadow_bias_factor: f32,
    // shadow_blur_half_kernel_size: i32,
}

#[derive(Clone, Debug)]
pub struct LightsUniform {
    lights: Vec<Light>,
    // shadow_bias_minimum: f32,
    // shadow_bias_factor: f32,
    // shadow_blur_half_kernel_size: i32,
}

impl Default for LightsUniform {
    fn default() -> Self {
        Self {
            lights: vec![],
            // shadow_bias_minimum: 0.005,
            // shadow_bias_factor: 0.005,
            // shadow_blur_half_kernel_size: 3,
        }
    }
}

impl UniformData for LightsUniform {
    type Raw = LightingUniformsRaw;

    fn raw(&self) -> Self::Raw {
        LightingUniformsRaw {
            items: std::array::from_fn(|i| {
                if i < self.lights.len() {
                    self.lights[i].into()
                } else {
                    Default::default()
                }
            }),
            count: self.lights.len() as _,
            // shadow_bias_minimum: self.shadow_bias_minimum,
            // shadow_bias_factor: self.shadow_bias_factor,
            // shadow_blur_half_kernel_size: self.shadow_blur_half_kernel_size,
        }
    }
}

impl LightsUniform {
    fn debug_ui(&mut self, _ui: &mut egui::Ui) {
        // for mut i in 0..(self.lights.len() as usize) {
        //     if i > 0 {
        //         ui.separator();
        //     }
        //     let removed = ui
        //         .horizontal(|ui| {
        //             ui.label(&format!("Light {}", i));
        //             if ui.button("remove").clicked() {
        //                 self.lights.remove(i);
        //                 i = i.saturating_sub(1);
        //                 true
        //             } else {
        //                 false
        //             }
        //         })
        //         .inner;
        //     if removed {
        //         continue;
        //     }
        //     let light = &mut self.lights[i];
        //     ui.add(egui::Slider::new(&mut light.position.x, -20.0..=20.0).text("x"));
        //     ui.add(egui::Slider::new(&mut light.position.y, -20.0..=20.0).text("y"));
        //     ui.add(egui::Slider::new(&mut light.position.z, -20.0..=20.0).text("z"));
        //     let mut c = egui::Rgba::from_rgba_premultiplied(
        //         light.color.x,
        //         light.color.y,
        //         light.color.z,
        //         light.color.w,
        //     );
        //     ui.horizontal(|ui| {
        //         ui.label("Color: ");
        //         egui::color_picker::color_edit_button_rgba(
        //             ui,
        //             &mut c,
        //             egui::color_picker::Alpha::OnlyBlend,
        //         );
        //     });
        //     light.color = c.to_array().into();
        //     light.view = Mat4::look_at_rh(light.position.xyz(), Vec3::ZERO, Vec3::X);
        // }
    }
}

pub struct LightingPass {
    pipeline: PipelineRef<BasicVertexData, BasicInstanceData>,
    lights_uniform: UniformBindGroup<LightsUniform>,
}

impl LightingPass {
    pub fn new(
        state: &mut RenderState,
        display: &Display,
        geometry_pass_bgl: &wgpu::BindGroupLayout,
    ) -> Self {
        let (lights_uniform, uniform_bgl) =
            state.create_uniform_bind_group(display.device(), LightsUniform::default());
        let pipeline = state
            .pipeline_builder()
            .with_label("Lighting Render Pipeline")
            .with_extra_bind_group_layouts(vec![geometry_pass_bgl, &uniform_bgl])
            .with_depth_stencil_state(None)
            .build(
                display.device(),
                &display
                    .device()
                    .create_shader_module(include_wgsl!("../../res/shaders/lighting.wgsl")),
            );
        Self {
            pipeline,
            lights_uniform,
        }
    }

    // pub fn bind_group(&self) -> &Arc<wgpu::BindGroup> {
    //     &self.bind_group
    // }
    //
    // pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
    //     &self.bind_group_layout
    // }
    //
    // pub fn shadow_map_texture(&self) -> &Texture {
    //     &self.shadow_map
    // }

    pub fn run(
        &mut self,
        state: &mut RenderState,
        display: &Display,
        destination: RenderTarget,
        view_projection: &ViewProjectionUniforms,
        geometry_pass_bg: Arc<wgpu::BindGroup>,
        occlusion_map: TextureRef,
        lights: &[Light],
    ) {
        self.lights_uniform.update(
            display.queue(),
            LightsUniform {
                lights: lights.to_vec(),
                ..Default::default()
            },
        );
        let quad = state.quad_mesh();
        state
            .render_pass(
                display,
                "Lighting Pass",
                &[destination],
                None,
                view_projection,
                |r| {
                    r.set_bind_group(3, geometry_pass_bg.deref(), &[]);
                    r.set_bind_group(4, self.lights_uniform.bind_group().deref(), &[]);
                    r.draw_instance(&InstanceRenderData {
                        mesh: quad,
                        instance: Default::default(),
                        texture: Some(occlusion_map),
                        pipeline: Some(self.pipeline),
                    });
                },
            )
            .submit();
    }

    pub fn debug_ui(&mut self, ui: &mut egui::Ui) {
        self.lights_uniform.debug_ui(ui)
    }
}
