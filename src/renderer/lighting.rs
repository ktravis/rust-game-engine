use std::{ops::Deref, sync::Arc};

use glam::{vec3, vec4, Mat4, Quat, Vec3, Vec4, Vec4Swizzles};
use wgpu::include_wgsl;

use crate::{camera::Frustum, color::Color, geom::BasicVertexData};

use super::{
    instance::InstanceRenderData, state::ViewProjectionUniforms, BasicInstanceData, Display,
    PipelineRef, RenderState, RenderTarget, TextureRef, UniformBindGroup, UniformData,
};

pub const MAX_LIGHTS: usize = 8;

#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
struct LightRaw {
    direction: Vec3, // TODO: pack this better?
    kind: u32,       // 0 = directional, 1 = spot, 2 = point
    color: Vec4,
    view_proj: Mat4,
    position: Vec3,
    radius: f32,
    reach: f32,
    _pad: [f32; 3],
}

#[derive(Copy, Clone, Debug)]
pub enum LightKind {
    Directional {
        theta: f32,
        phi: f32,
    },
    Spot {
        position: Vec3,
        direction: Vec3,
        fov_degrees: f32,
        reach: f32,
    },
}

impl LightKind {
    pub fn position(&self) -> Vec3 {
        match self {
            LightKind::Directional { theta, phi } => {
                let theta = theta.to_radians();
                let phi = phi.to_radians();
                (Mat4::from_quat(Quat::from_rotation_y(phi) * Quat::from_rotation_x(theta))
                    * Vec4::Y)
                    .xyz()
                    .normalize()
            }
            LightKind::Spot { position, .. } => *position,
        }
    }

    pub fn view_matrix_from_position(&self, pos: Vec3) -> Mat4 {
        match self {
            LightKind::Directional { .. } => {
                let mut up = Vec3::Y;
                if up == pos {
                    up += vec3(0.0001, 0.0, 0.0);
                }
                Mat4::look_at_rh(pos, Vec3::ZERO, up)
            }
            LightKind::Spot { direction, .. } => {
                let mut up = Vec3::Y;
                if up == pos {
                    up += vec3(0.0001, 0.0, 0.0);
                }
                Mat4::look_to_rh(pos, *direction, up)
            }
        }
    }

    // pub fn projection_matrix(&self, view_frustum: &Frustum) -> Mat4 {
    //     match self {
    //         LightKind::Directional { .. } => {
    //             Mat4::orthographic_rh(-20.0, 20.0, -10.0, 20.0, -20.0, 20.0)
    //         }
    //     }
    // }

    pub fn view_matrix(&self) -> Mat4 {
        self.view_matrix_from_position(self.position())
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Light {
    pub color: Color,
    pub kind: LightKind,
}

impl Light {
    pub fn view_proj_uniforms(&self, view_frustum: &Frustum) -> ViewProjectionUniforms {
        let pos = self.kind.position();
        let view = self.kind.view_matrix_from_position(pos);
        let inverse_view = view.inverse();

        let projection = match self.kind {
            LightKind::Directional { .. } => {
                let light_view_frustum = view_frustum.mul(view);
                let (bounds_min, bounds_max) = light_view_frustum.aabb();
                Mat4::orthographic_rh(
                    bounds_min.x,
                    bounds_max.x,
                    bounds_min.y,
                    bounds_max.y,
                    -bounds_max.z,
                    -bounds_min.z,
                )
            }
            LightKind::Spot {
                fov_degrees, reach, ..
            } => Mat4::perspective_rh(fov_degrees.to_radians(), 1.0, 0.1, reach),
        };
        ViewProjectionUniforms {
            view,
            projection,
            pos,
            inverse_view,
        }
    }

    pub fn debug_ui(&mut self, ui: &mut egui::Ui) {
        match &mut self.kind {
            LightKind::Directional { theta, phi } => {
                ui.add(egui::Slider::new(theta, -180.0..=180.0).text("theta"));
                ui.add(egui::Slider::new(phi, -180.0..=180.0).text("phi"));
            }
            LightKind::Spot {
                position,
                direction,
                fov_degrees,
                reach,
            } => {
                ui.label("Position: ");
                ui.add(egui::Slider::new(&mut position.x, -100.0..=100.0).text("x"));
                ui.add(egui::Slider::new(&mut position.y, -100.0..=100.0).text("y"));
                ui.add(egui::Slider::new(&mut position.z, -100.0..=100.0).text("z"));
                ui.label("Direction: ");
                ui.add(egui::Slider::new(&mut direction.x, -100.0..=100.0).text("x"));
                ui.add(egui::Slider::new(&mut direction.y, -100.0..=100.0).text("y"));
                ui.add(egui::Slider::new(&mut direction.z, -100.0..=100.0).text("z"));
                ui.add(egui::Slider::new(fov_degrees, 1.0..=180.0).text("fov"));
                ui.add(egui::Slider::new(reach, 0.1..=180.0).text("reach"));
            }
        }
        let mut c = egui::Rgba::from_rgba_premultiplied(
            self.color.r,
            self.color.g,
            self.color.b,
            self.color.a,
        );
        ui.horizontal(|ui| {
            ui.label("Color: ");
            egui::color_picker::color_edit_button_rgba(
                ui,
                &mut c,
                egui::color_picker::Alpha::OnlyBlend,
            );
        });
        self.color = c.into();
    }

    fn to_raw(&self, view_frustum: &Frustum) -> LightRaw {
        let position = self.kind.position();
        let view = self.kind.view_matrix_from_position(position);

        match self.kind {
            LightKind::Directional { .. } => {
                let light_view_frustum = view_frustum.mul(view);
                let (bounds_min, bounds_max) = light_view_frustum.aabb();
                let projection = Mat4::orthographic_rh(
                    bounds_min.x,
                    bounds_max.x,
                    bounds_min.y,
                    bounds_max.y,
                    -bounds_max.z,
                    -bounds_min.z,
                );
                LightRaw {
                    kind: 0,
                    color: self.color.into(),
                    view_proj: projection * view,
                    position,
                    direction: -position.normalize(),
                    radius: 0.0,
                    reach: 0.0,
                    _pad: [0.0, 0.0, 0.0],
                }
            }
            LightKind::Spot {
                fov_degrees,
                reach,
                direction,
                ..
            } => {
                let fov_radians = fov_degrees.to_radians();
                let projection = Mat4::perspective_rh(fov_radians, 1.0, 0.1, reach);
                LightRaw {
                    kind: 1,
                    color: self.color.into(),
                    view_proj: projection * view,
                    position,
                    direction: direction.normalize(),
                    radius: (fov_radians / 2.0).cos(),
                    reach,
                    _pad: [0.0, 0.0, 0.0],
                }
            }
        }
    }
}

impl From<LightKind> for Light {
    fn from(kind: LightKind) -> Self {
        Self {
            color: Color::WHITE,
            kind,
        }
    }
}

#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct LightingUniformsRaw {
    items: [LightRaw; MAX_LIGHTS],
    count: u32,
    shadow_bias_minimum: f32,
    shadow_bias_factor: f32,
    shadow_blur_half_kernel_size: i32,
    ambient_color: Vec4,
}

#[derive(Clone, Debug)]
pub struct LightsUniform {
    pub lights: Vec<Light>,
    pub view_frustum: Frustum,
    pub shadow_bias_minimum: f32,
    pub shadow_bias_factor: f32,
    pub shadow_blur_half_kernel_size: i32,
    pub ambient_color: Color,
}

impl Default for LightsUniform {
    fn default() -> Self {
        Self {
            lights: vec![],
            view_frustum: Default::default(),
            shadow_bias_minimum: 0.005,
            shadow_bias_factor: 0.025,
            shadow_blur_half_kernel_size: 4,
            ambient_color: Color::from(Vec4::splat(0.1)),
        }
    }
}

impl UniformData for LightsUniform {
    type Raw = LightingUniformsRaw;

    fn raw(&self) -> Self::Raw {
        LightingUniformsRaw {
            items: std::array::from_fn(|i| {
                if i < self.lights.len() {
                    self.lights[i].to_raw(&self.view_frustum)
                } else {
                    Default::default()
                }
            }),
            count: self.lights.len() as _,
            shadow_bias_minimum: self.shadow_bias_minimum,
            shadow_bias_factor: self.shadow_bias_factor,
            shadow_blur_half_kernel_size: self.shadow_blur_half_kernel_size,
            ambient_color: self.ambient_color.into(),
        }
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
}
