use glam::{Mat4, Quat, Vec3};

pub trait Transform: Default + Copy + Clone + std::fmt::Debug {
    fn as_mat4(self) -> Mat4;
}

impl Transform for Mat4 {
    fn as_mat4(self) -> Mat4 {
        self
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Transform3D {
    pub position: Vec3,
    pub scale: Vec3,
    pub rotation: Quat,
}

impl Default for Transform3D {
    #[inline]
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Transform3D {
    const IDENTITY: Self = Self {
        position: Vec3::ZERO,
        scale: Vec3::ONE,
        rotation: Quat::IDENTITY,
    };

    #[inline]
    pub fn from_matrix(m: Mat4) -> Self {
        let (scale, rotation, position) = m.to_scale_rotation_translation();
        Self {
            position,
            rotation,
            scale,
        }
    }
}

impl Transform for Transform3D {
    #[inline]
    fn as_mat4(self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position)
    }
}
