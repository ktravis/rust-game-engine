use crate::renderer::VertexLayout;
use glam::{vec2, Vec2};
use wgpu::{vertex_attr_array, VertexAttribute, VertexBufferLayout};

pub trait VertexData:
    VertexLayout + std::fmt::Debug + Default + Clone + Copy + bytemuck::Pod + bytemuck::Zeroable
{
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BasicVertexData {
    pub pos: [f32; 4],
    pub uv: [f32; 2],
}

impl VertexLayout for BasicVertexData {
    fn vertex_layout() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

impl BasicVertexData {
    const ATTRIBUTES: [VertexAttribute; 2] = vertex_attr_array![
        0 => Float32x4,
        1 => Float32x2,
    ];
}

impl VertexData for BasicVertexData {}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelVertexData {
    pub pos: [f32; 4],
    pub uv: [f32; 2],
    pub normal: [f32; 3],
}

impl VertexLayout for ModelVertexData {
    fn vertex_layout() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

impl ModelVertexData {
    const ATTRIBUTES: [VertexAttribute; 3] = vertex_attr_array![
        0 => Float32x4,
        1 => Float32x2,
        2 => Float32x3,
    ];
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

impl Point<u32> {
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

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Rect {
    pub dim: Vec2,
    pub pos: Vec2,
}

impl Rect {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {
            dim: vec2(w, h),
            pos: vec2(x, y),
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

impl Default for Rect {
    fn default() -> Self {
        Self {
            pos: Vec2::default(),
            dim: vec2(1., 1.),
        }
    }
}

pub mod quad {
    use super::*;

    #[inline]
    pub fn verts(
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        uv1: (f32, f32),
        uv2: (f32, f32),
    ) -> [BasicVertexData; 4] {
        [
            BasicVertexData {
                pos: [x, y, 0.0, 1.0],
                uv: [uv1.0, uv2.1],
            },
            BasicVertexData {
                pos: [x, y + h, 0.0, 1.0],
                uv: [uv1.0, uv1.1],
            },
            BasicVertexData {
                pos: [x + w, y + h, 0.0, 1.0],
                uv: [uv2.0, uv1.1],
            },
            BasicVertexData {
                pos: [x + w, y, 0.0, 1.0],
                uv: [uv2.0, uv2.1],
            },
        ]
    }

    pub const INDICES: &[u16] = &[0, 1, 2, 0, 2, 3];
}

pub mod cube {
    use glam::Vec3;
    const FORWARD: [f32; 3] = Vec3::NEG_Z.to_array();
    const BACK: [f32; 3] = Vec3::Z.to_array();
    const LEFT: [f32; 3] = Vec3::NEG_X.to_array();
    const RIGHT: [f32; 3] = Vec3::X.to_array();
    const DOWN: [f32; 3] = Vec3::NEG_Y.to_array();
    const UP: [f32; 3] = Vec3::Y.to_array();

    const BL: [f32; 2] = [0.0, 1.0];
    const BR: [f32; 2] = [1.0, 1.0];
    const TL: [f32; 2] = [0.0, 0.0];
    const TR: [f32; 2] = [1.0, 0.0];

    use super::*;

    const fn v(pos: [f32; 3], uv: [f32; 2], normal: [f32; 3]) -> ModelVertexData {
        ModelVertexData {
            pos: [pos[0], pos[1], pos[2], 1.0],
            uv,
            normal,
        }
    }

    pub const VERTICES: [ModelVertexData; 36] = [
        // front face (facing -z direction)
        v([-1., 1., -1.], TL, FORWARD),
        v([-1., -1., -1.], BL, FORWARD),
        v([1., 1., -1.], TR, FORWARD),
        v([1., 1., -1.], TR, FORWARD),
        v([-1., -1., -1.], BL, FORWARD),
        v([1., -1., -1.], BR, FORWARD),
        // back face
        v([1., 1., 1.], TL, BACK),
        v([1., -1., 1.], BL, BACK),
        v([-1., 1., 1.], TR, BACK),
        v([-1., 1., 1.], TR, BACK),
        v([1., -1., 1.], BL, BACK),
        v([-1., -1., 1.], BR, BACK),
        // left face
        v([-1., 1., 1.], TL, LEFT),
        v([-1., -1., 1.], BL, LEFT),
        v([-1., 1., -1.], TR, LEFT),
        v([-1., 1., -1.], TR, LEFT),
        v([-1., -1., 1.], BL, LEFT),
        v([-1., -1., -1.], BR, LEFT),
        // right face
        v([1., 1., -1.], TL, RIGHT),
        v([1., -1., -1.], BL, RIGHT),
        v([1., 1., 1.], TR, RIGHT),
        v([1., 1., 1.], TR, RIGHT),
        v([1., -1., -1.], BL, RIGHT),
        v([1., -1., 1.], BR, RIGHT),
        // top face
        v([-1., 1., 1.], TL, UP),
        v([-1., 1., -1.], BL, UP),
        v([1., 1., 1.], TR, UP),
        v([1., 1., 1.], TR, UP),
        v([-1., 1., -1.], BL, UP),
        v([1., 1., -1.], BR, UP),
        // bottom face
        v([-1., -1., -1.], TL, DOWN),
        v([-1., -1., 1.], BL, DOWN),
        v([1., -1., -1.], TR, DOWN),
        v([1., -1., -1.], TR, DOWN),
        v([-1., -1., 1.], BL, DOWN),
        v([1., -1., 1.], BR, DOWN),
    ];

    pub const INDICES: &[u16] = &[
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
        25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35,
    ];
}
