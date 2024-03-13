#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Default for Color {
    fn default() -> Self {
        Self::WHITE
    }
}

impl Color {
    pub const BLACK: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };
    pub const WHITE: Self = Self {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
    pub const RED: Self = Self {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    pub const GREEN: Self = Self {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    };

    pub fn as_u8(self) -> [u8; 4] {
        [
            (self.r * 255.0) as _,
            (self.g * 255.0) as _,
            (self.b * 255.0) as _,
            (self.a * 255.0) as _,
        ]
    }
}

impl From<glam::Vec4> for Color {
    fn from(glam::Vec4 { x, y, z, w }: glam::Vec4) -> Self {
        Self {
            r: x,
            g: y,
            b: z,
            a: w,
        }
    }
}

impl From<Color> for glam::Vec4 {
    fn from(Color { r, g, b, a }: Color) -> Self {
        Self {
            x: r,
            y: g,
            z: b,
            w: a,
        }
    }
}

impl From<(f32, f32, f32, f32)> for Color {
    fn from((r, g, b, a): (f32, f32, f32, f32)) -> Self {
        Self { r, g, b, a }
    }
}

impl From<(f32, f32, f32)> for Color {
    fn from((r, g, b): (f32, f32, f32)) -> Self {
        Self { r, g, b, a: 1.0 }
    }
}

impl From<u32> for Color {
    fn from(c: u32) -> Self {
        Self {
            r: ((c >> 24) & 0xff) as f32 / 255.,
            g: ((c >> 16) & 0xff) as f32 / 255.,
            b: ((c >> 8) & 0xff) as f32 / 255.,
            a: (c & 0xff) as f32 / 255.,
        }
    }
}

impl From<Color> for (f32, f32, f32, f32) {
    fn from(Color { r, g, b, a }: Color) -> Self {
        (r, g, b, a)
    }
}

impl From<Color> for (f32, f32, f32) {
    fn from(Color { r, g, b, a: _ }: Color) -> Self {
        (r, g, b)
    }
}
