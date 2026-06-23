use super::{instance::InstanceRenderData, MeshRef, PipelineRef, TextureRef};
use crate::{color::*, geom::*, transform::*};
use glam::{Mat3, Mat4};
use shadertype_derive::{ShaderType, VertexInput};
use std::fmt::Debug;

#[derive(Copy, Clone, Default)]
pub struct RenderData<V, I> {
    pub pipeline: Option<PipelineRef<V, I>>,
    pub texture: TextureRef,
    pub mesh: MeshRef<V>,
}

impl<V, I> RenderData<V, I> {
    pub fn for_instance(self, instance: I) -> InstanceRenderData<V, I> {
        InstanceRenderData {
            texture: Some(self.texture.into()),
            pipeline: self.pipeline,
            mesh: self.mesh,
            instance,
        }
    }
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, ShaderType, VertexInput)]
pub struct BasicInstanceData {
    pub subtexture: Rect,
    pub tint: Color,
    pub transform: Mat4,
}

impl BasicInstanceData {
    #[inline]
    pub fn transform(transform: impl Transform) -> Self {
        Self {
            transform: transform.as_mat4(),
            ..Default::default()
        }
    }
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, ShaderType, VertexInput)]
pub struct InstanceDataWithNormalMatrix {
    pub subtexture: Rect,
    pub tint: Color,
    pub transform: Mat4,
    pub normal_matrix: Mat3,
    pub material: u32,
}

impl InstanceDataWithNormalMatrix {
    pub fn from_basic(other: BasicInstanceData, view_matrix: Mat4) -> Self {
        Self {
            transform: other.transform,
            tint: other.tint,
            subtexture: other.subtexture,
            // TODO: this is the big performance killer, we need to not do this every frame
            normal_matrix: Mat3::from_mat4(view_matrix * other.transform)
                .inverse()
                .transpose(),
            material: 0,
        }
    }
}

#[rustfmt::skip]
pub const DEFAULT_TEXTURE_DATA: [u8; 16] = [
    255, 255, 255, 255,
    255, 255, 255, 255,
    255, 255, 255, 255,
    255, 255, 255, 255,
];

#[derive(Debug)]
pub struct OffscreenFramebuffer {
    pub color: TextureRef,
    pub depth: Option<TextureRef>,
    pub(super) size: Point<u32>,
    pub(super) format: wgpu::TextureFormat,
}

impl OffscreenFramebuffer {
    pub fn size_pixels(&self) -> Point<u32> {
        self.size
    }

    pub fn color_format(&self) -> wgpu::TextureFormat {
        self.format
    }
}
