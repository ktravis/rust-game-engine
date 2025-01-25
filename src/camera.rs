use glam::{vec3, vec4, Mat4, Vec3, Vec4, Vec4Swizzles};

#[derive(Debug, Copy, Clone)]
pub struct Camera {
    position: Vec3,
    pitch: f32,
    yaw: f32,
    look_dir: Vec3,
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
    pub const DEFAULT_Z_NEAR: f32 = 0.1;
    pub const DEFAULT_Z_FAR: f32 = 50.0;

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

    pub fn frustum(&self) -> Frustum {
        let inverse_proj_view = (self.perspective_matrix() * self.view_matrix()).inverse();
        let mut ndc = Frustum {
            nlt: vec4(-1.0, 1.0, 1.0, 1.0),
            nrt: vec4(1.0, 1.0, 1.0, 1.0),
            nlb: vec4(-1.0, -1.0, 1.0, 1.0),
            nrb: vec4(1.0, -1.0, 1.0, 1.0),
            flt: vec4(-1.0, 1.0, -1.0, 1.0),
            frt: vec4(1.0, 1.0, -1.0, 1.0),
            flb: vec4(-1.0, -1.0, -1.0, 1.0),
            frb: vec4(1.0, -1.0, -1.0, 1.0),
        };
        ndc.nlt = inverse_proj_view * ndc.nlt;
        ndc.nlt /= ndc.nlt.w;
        ndc.nrt = inverse_proj_view * ndc.nrt;
        ndc.nrt /= ndc.nrt.w;
        ndc.nlb = inverse_proj_view * ndc.nlb;
        ndc.nlb /= ndc.nlb.w;
        ndc.nrb = inverse_proj_view * ndc.nrb;
        ndc.nrb /= ndc.nrb.w;
        ndc.flt = inverse_proj_view * ndc.flt;
        ndc.flt /= ndc.flt.w;
        ndc.frt = inverse_proj_view * ndc.frt;
        ndc.frt /= ndc.frt.w;
        ndc.flb = inverse_proj_view * ndc.flb;
        ndc.flb /= ndc.flb.w;
        ndc.frb = inverse_proj_view * ndc.frb;
        ndc.frb /= ndc.frb.w;
        ndc
    }

    pub fn aspect_ratio(&self) -> f32 {
        self.aspect_ratio
    }

    pub fn fov_radians(&self) -> f32 {
        self.fov_radians
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

#[derive(Copy, Clone, Debug, Default)]
pub struct Frustum {
    pub nlt: Vec4,
    pub nrt: Vec4,
    pub nlb: Vec4,
    pub nrb: Vec4,
    pub flt: Vec4,
    pub frt: Vec4,
    pub flb: Vec4,
    pub frb: Vec4,
}

impl Frustum {
    pub fn mul(&self, m: Mat4) -> Frustum {
        Frustum {
            nlt: m * self.nlt,
            nrt: m * self.nrt,
            nlb: m * self.nlb,
            nrb: m * self.nrb,
            flt: m * self.flt,
            frt: m * self.frt,
            flb: m * self.flb,
            frb: m * self.frb,
        }
    }

    pub fn aabb(&self) -> (Vec3, Vec3) {
        let mut min = vec3(f32::INFINITY, f32::INFINITY, f32::INFINITY);
        let mut max = vec3(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY);

        let points = [
            self.nlt, self.nrt, self.nlb, self.nrb, self.flt, self.frt, self.flb, self.frb,
        ];

        for p in points {
            min = min.min(p.xyz());
            max = max.max(p.xyz());
        }
        (min, max)
    }
}
