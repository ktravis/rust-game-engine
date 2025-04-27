use bytemuck::Zeroable;
use glam::{vec3, Mat4, Quat, Vec3, Vec4, Vec4Swizzles};

use crate::{camera::Frustum, color::Color};

use super::{shaders, state::ViewProjectionUniforms, UniformData};

pub type LightRaw = shaders::forward::types::Light;

impl Default for LightRaw {
    fn default() -> Self {
        Self {
            view_proj: Mat4::IDENTITY,
            ..Zeroable::zeroed()
        }
    }
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
        let camera_pos = self.kind.position();
        let view = self.kind.view_matrix_from_position(camera_pos);
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
            camera_pos,
            inverse_view,
            ..Default::default()
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
                    ..Default::default()
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
                    ..Default::default()
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

pub type LightingUniformsRaw = shaders::forward::types::LightsUniform;

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
