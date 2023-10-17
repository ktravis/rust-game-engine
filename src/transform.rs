use glam::{vec2, vec3, Mat4, Quat, Vec2, Vec3};

pub trait ModelTransform: Default + Copy + Clone {
    fn model_transform(self) -> Mat4;
}

#[derive(Debug, Copy, Clone)]
pub struct Transform2D {
    pub pos: Vec2,
    pub scale: Vec2,
    pub angle: f32,
}

impl ModelTransform for Transform2D {
    #[inline]
    fn model_transform(self) -> Mat4 {
        let Self { scale, angle, pos } = self;
        Mat4::from_scale_rotation_translation(
            scale.extend(1.),
            Quat::from_rotation_z(angle),
            pos.extend(0.),
        )
    }
}

impl Default for Transform2D {
    fn default() -> Self {
        Self {
            pos: Default::default(),
            scale: vec2(1., 1.),
            angle: 0.,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Transform3D {
    pub pos: Vec3,
    pub scale: Vec3,
    pub rotation: Quat,
}

impl ModelTransform for Transform3D {
    #[inline]
    fn model_transform(self) -> Mat4 {
        let Self {
            scale,
            rotation,
            pos,
        } = self;
        Mat4::from_scale_rotation_translation(scale, rotation, pos)
    }
}

impl Default for Transform3D {
    fn default() -> Self {
        Self {
            pos: Default::default(),
            scale: vec3(1., 1., 1.),
            rotation: Quat::IDENTITY,
        }
    }
}
