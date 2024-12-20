use glam::{vec3, Mat4, Vec3};

#[derive(Debug, Copy, Clone)]
pub struct Camera {
    position: Vec3,
    pitch: f32,
    yaw: f32,
    pub look_dir: Vec3,
    fov_radians: f32,

    aspect_ratio: f32,
    min_yaw: f32,
    max_yaw: f32,
    z_near: f32,
    z_far: f32,
}

impl Camera {
    pub const DEFAULT_FOV_RADIANS: f32 = 55.0 * (std::f32::consts::PI / 180.0);
    const DEFAULT_MAX_YAW: f32 = 80.0 * (std::f32::consts::PI / 180.0);
    const DEFAULT_MIN_YAW: f32 = -80.0 * (std::f32::consts::PI / 180.0);
    pub const DEFAULT_Z_NEAR: f32 = 0.01;
    pub const DEFAULT_Z_FAR: f32 = 100.0;

    pub fn new(position: Vec3, aspect_ratio: f32) -> Self {
        Self {
            position,
            aspect_ratio,
            pitch: -12.0f32.to_radians(),
            yaw: -90.0f32.to_radians(),
            look_dir: Vec3::NEG_Z,
            ..Default::default()
        }
    }

    pub fn position(&self) -> Vec3 {
        self.position
    }

    pub fn look_dir(&self) -> Vec3 {
        self.look_dir
    }

    pub fn pitch(&self) -> f32 {
        self.pitch
    }

    pub fn yaw(&self) -> f32 {
        self.yaw
    }

    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_to_rh(self.position, self.look_dir, Vec3::Y)
    }

    pub fn perspective_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fov_radians, self.aspect_ratio, self.z_near, self.z_far)
    }

    pub fn update_position(&mut self, d: Vec3) -> Vec3 {
        let flat = vec3(self.look_dir.x, 0.0, self.look_dir.z).normalize();
        self.position += d.x * flat.cross(Vec3::Y) - d.z * flat;
        self.position.y += d.y;
        self.position
    }

    pub fn update_angle(&mut self, delta_yaw: f32, delta_pitch: f32) -> Vec3 {
        self.yaw += delta_yaw;
        self.pitch -= delta_pitch;
        self.pitch = self.pitch.clamp(self.min_yaw, self.max_yaw);
        let (pitch_sin, pitch_cos) = self.pitch.sin_cos();
        let (yaw_sin, yaw_cos) = self.yaw.sin_cos();
        self.look_dir = vec3(pitch_cos * yaw_cos, pitch_sin, pitch_cos * yaw_sin).normalize();
        self.look_dir
    }
}

impl Default for Camera {
    fn default() -> Self {
        Camera {
            position: Vec3::ZERO,
            pitch: 0.0,
            yaw: 0.0,
            fov_radians: Self::DEFAULT_FOV_RADIANS,
            look_dir: Vec3::NEG_Z,
            min_yaw: Self::DEFAULT_MIN_YAW,
            max_yaw: Self::DEFAULT_MAX_YAW,
            z_near: Self::DEFAULT_Z_NEAR,
            z_far: Self::DEFAULT_Z_FAR,
            aspect_ratio: 16.0 / 9.0,
        }
    }
}
