use crate::mesh::Mesh;
use crate::renderer::VertexLayout;
use glam::{vec2, vec4, Vec2, Vec4};
use miniquad::{VertexAttribute, VertexFormat};

pub trait VertexData: VertexLayout + std::fmt::Debug + Default + Clone + Copy {}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct ModelVertexData {
    pub pos: Vec4,
    pub uv: Vec2,
}

impl VertexLayout for ModelVertexData {
    fn vertex_layout() -> Vec<VertexAttribute> {
        vec![
            VertexAttribute::with_buffer("position", VertexFormat::Float4, 0),
            VertexAttribute::with_buffer("uv", VertexFormat::Float2, 0),
        ]
    }
}

impl VertexData for ModelVertexData {}

#[derive(Clone, Copy, Debug, Default)]
pub struct Point<T = i32> {
    pub x: T,
    pub y: T,
}

impl<T> Point<T> {
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

impl<T> From<(T, T)> for Point<T> {
    fn from((x, y): (T, T)) -> Self {
        Self::new(x, y)
    }
}

impl<T> From<Point<T>> for (T, T) {
    fn from(p: Point<T>) -> Self {
        (p.x, p.y)
    }
}

impl Point {
    pub fn as_vec2(&self) -> Vec2 {
        Vec2::new(self.x as f32, self.y as f32)
    }
}

impl<T> std::ops::Add for Point<T>
where
    T: std::ops::Add<Output = T>,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl<T> std::ops::Sub for Point<T>
where
    T: std::ops::Sub<Output = T>,
{
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl<T> std::ops::Mul<T> for Point<T>
where
    T: Copy + std::ops::Mul<Output = T>,
{
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        Self::Output {
            x: rhs * self.x,
            y: rhs * self.y,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub pos: Vec2,
    pub dim: Vec2,
}

impl Rect {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {
            pos: vec2(x, y),
            dim: vec2(w, h),
        }
    }

    /// Upper-left and lower-right corner of rectangle
    pub fn bounds(&self) -> (Vec2, Vec2) {
        (self.pos, self.pos + self.dim)
    }

    /// All 4 corners ordered clockwise beginning with upper-left
    pub fn corners(&self) -> [Vec2; 4] {
        [
            self.pos,
            vec2(self.pos.x + self.dim.x, self.pos.y),
            self.pos + self.dim,
            vec2(self.pos.x, self.pos.y + self.dim.y),
        ]
    }
}

pub mod quad {
    use super::*;

    // #[rustfmt::skip]
    // pub const VERTICES: &[VertexData] = &verts(0.0, 0.0, 1.0, 1.0, (0.0, 0.0), (1.0, 1.0));
    // 0.0, 0.0, 0.0, 1.0,   0.0, 0.0,
    // 1.0, 0.0, 0.0, 1.0,   1.0, 0.0,
    // 1.0,  1.0, 0.0, 1.0,   1.0, 1.0,
    // 0.0,  1.0, 0.0, 1.0,   0.0, 1.0,

    #[inline]
    pub fn verts(
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        uv1: (f32, f32),
        uv2: (f32, f32),
    ) -> [ModelVertexData; 4] {
        [
            ModelVertexData {
                pos: glam::vec4(x, y, 0.0, 1.0),
                uv: glam::vec2(uv1.0, uv1.1),
            },
            ModelVertexData {
                pos: glam::vec4(x, y + h, 0.0, 1.0),
                uv: glam::vec2(uv1.0, uv2.1),
            },
            ModelVertexData {
                pos: glam::vec4(x + w, y + h, 0.0, 1.0),
                uv: glam::vec2(uv2.0, uv2.1),
            },
            ModelVertexData {
                pos: glam::vec4(x + w, y, 0.0, 1.0),
                uv: glam::vec2(uv2.0, uv1.1),
            },
        ]
    }

    pub const INDICES: &[u16] = &[0, 1, 2, 0, 2, 3];

    pub fn mesh() -> Mesh<ModelVertexData> {
        Mesh {
            vertices: verts(0., 0., 1., 1., (0., 0.), (1., 1.)).to_vec(),
            indices: INDICES.to_vec(),
        }
    }
}

pub mod cube {
    use super::*;

    pub const VERTICES: [ModelVertexData; 24] = [
        ModelVertexData {
            pos: vec4(-1.0, -1.0, -1.0, 1.),
            uv: vec2(0.0, 0.0),
        },
        ModelVertexData {
            pos: vec4(1.0, -1.0, -1.0, 1.),
            uv: vec2(1.0, 0.0),
        },
        ModelVertexData {
            pos: vec4(1.0, 1.0, -1.0, 1.),
            uv: vec2(1.0, 1.0),
        },
        ModelVertexData {
            pos: vec4(-1.0, 1.0, -1.0, 1.),
            uv: vec2(0.0, 1.0),
        },
        ModelVertexData {
            pos: vec4(-1.0, -1.0, 1.0, 1.),
            uv: vec2(0.0, 0.0),
        },
        ModelVertexData {
            pos: vec4(1.0, -1.0, 1.0, 1.),
            uv: vec2(1.0, 0.0),
        },
        ModelVertexData {
            pos: vec4(1.0, 1.0, 1.0, 1.),
            uv: vec2(1.0, 1.0),
        },
        ModelVertexData {
            pos: vec4(-1.0, 1.0, 1.0, 1.),
            uv: vec2(0.0, 1.0),
        },
        ModelVertexData {
            pos: vec4(-1.0, -1.0, -1.0, 1.),
            uv: vec2(0.0, 0.0),
        },
        ModelVertexData {
            pos: vec4(-1.0, 1.0, -1.0, 1.),
            uv: vec2(1.0, 0.0),
        },
        ModelVertexData {
            pos: vec4(-1.0, 1.0, 1.0, 1.),
            uv: vec2(1.0, 1.0),
        },
        ModelVertexData {
            pos: vec4(-1.0, -1.0, 1.0, 1.),
            uv: vec2(0.0, 1.0),
        },
        ModelVertexData {
            pos: vec4(1.0, -1.0, -1.0, 1.),
            uv: vec2(0.0, 0.0),
        },
        ModelVertexData {
            pos: vec4(1.0, 1.0, -1.0, 1.),
            uv: vec2(1.0, 0.0),
        },
        ModelVertexData {
            pos: vec4(1.0, 1.0, 1.0, 1.),
            uv: vec2(1.0, 1.0),
        },
        ModelVertexData {
            pos: vec4(1.0, -1.0, 1.0, 1.),
            uv: vec2(0.0, 1.0),
        },
        ModelVertexData {
            pos: vec4(-1.0, -1.0, -1.0, 1.),
            uv: vec2(0.0, 0.0),
        },
        ModelVertexData {
            pos: vec4(-1.0, -1.0, 1.0, 1.),
            uv: vec2(1.0, 0.0),
        },
        ModelVertexData {
            pos: vec4(1.0, -1.0, 1.0, 1.),
            uv: vec2(1.0, 1.0),
        },
        ModelVertexData {
            pos: vec4(1.0, -1.0, -1.0, 1.),
            uv: vec2(0.0, 1.0),
        },
        ModelVertexData {
            pos: vec4(-1.0, 1.0, -1.0, 1.),
            uv: vec2(0.0, 0.0),
        },
        ModelVertexData {
            pos: vec4(-1.0, 1.0, 1.0, 1.),
            uv: vec2(1.0, 0.0),
        },
        ModelVertexData {
            pos: vec4(1.0, 1.0, 1.0, 1.),
            uv: vec2(1.0, 1.0),
        },
        ModelVertexData {
            pos: vec4(1.0, 1.0, -1.0, 1.),
            uv: vec2(0.0, 1.0),
        },
    ];

    pub const INDICES: &[u16] = &[
        0, 1, 2, 0, 2, 3, 6, 5, 4, 7, 6, 4, 8, 9, 10, 8, 10, 11, 14, 13, 12, 15, 14, 12, 16, 17,
        18, 16, 18, 19, 22, 21, 20, 23, 22, 20,
    ];

    pub fn mesh() -> Mesh {
        Mesh {
            vertices: VERTICES.to_vec(),
            indices: INDICES.to_vec(),
        }
    }
}
