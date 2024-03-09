use glam::{Mat4, Quat, Vec2, Vec3, Vec3Swizzles};

pub trait Transform: Default + Copy + Clone + std::fmt::Debug {
    fn as_mat4(self) -> Mat4;
}

impl Transform for Mat4 {
    #[inline]
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

#[derive(Debug, Copy, Clone)]
pub struct Transform2D {
    pub position: Vec2,
    pub scale: Vec2,
    pub rotation_rad: f32,
}

impl Default for Transform2D {
    #[inline]
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Transform2D {
    const IDENTITY: Self = Self {
        position: Vec2::ZERO,
        scale: Vec2::ONE,
        rotation_rad: 0.0,
    };

    #[inline]
    pub fn from_matrix(m: Mat4) -> Self {
        let (scale, rotation, position) = m.to_scale_rotation_translation();
        Self {
            position: position.xy(),
            rotation_rad: rotation.angle_between(Quat::from_rotation_z(0.0)),
            scale: scale.xy(),
        }
    }

    #[inline]
    pub fn position(position: Vec2) -> Self {
        Self {
            position,
            ..Self::IDENTITY
        }
    }
}

impl Transform for Transform2D {
    #[inline]
    fn as_mat4(self) -> Mat4 {
        Mat4::from_scale_rotation_translation(
            self.scale.extend(1.0),
            Quat::from_rotation_z(self.rotation_rad),
            self.position.extend(0.0),
        )
    }
}

pub trait MatrixTransform {
    fn translate(&mut self, delta: Vec3) -> &mut Self;
    fn scale(&mut self, s: Vec3) -> &mut Self;
    fn rotate(&mut self, quat: Quat) -> &mut Self;
}

impl MatrixTransform for Mat4 {
    fn translate(&mut self, delta: Vec3) -> &mut Self {
        *self = Mat4::from_translation(delta) * *self;
        self
    }

    fn scale(&mut self, s: Vec3) -> &mut Self {
        *self = Mat4::from_scale(s) * *self;
        self
    }

    fn rotate(&mut self, rotation: Quat) -> &mut Self {
        *self = Mat4::from_quat(rotation) * *self;
        self
    }
}
