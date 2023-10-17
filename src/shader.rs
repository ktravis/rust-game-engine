use crate::geom::VertexData;
use crate::renderer::InstanceData;
use miniquad::{
    GraphicsContext, ShaderError, ShaderMeta, UniformBlockLayout, UniformDesc, UniformType,
};
use std::marker::PhantomData;

pub trait Uniforms {
    fn block_layout() -> UniformBlockLayout;
}

#[repr(C)]
pub struct BasicUniforms {
    pub view: glam::Mat4,
    pub projection: glam::Mat4,
}

impl Uniforms for BasicUniforms {
    fn block_layout() -> UniformBlockLayout {
        UniformBlockLayout {
            uniforms: vec![
                UniformDesc::new("view", UniformType::Mat4),
                UniformDesc::new("projection", UniformType::Mat4),
            ],
        }
    }
}

#[derive(Clone)]
pub struct Shader<V: VertexData, I: InstanceData = ()> {
    program: miniquad::Shader,
    meta: ShaderMeta,
    _marker: PhantomData<(V, I)>,
}

impl<V: VertexData, I: InstanceData> Shader<V, I> {
    pub fn new(
        ctx: &mut GraphicsContext,
        vertex_src: impl AsRef<str>,
        fragment_src: impl AsRef<str>,
        image_names: Vec<String>,
    ) -> Result<Self, ShaderError> {
        let meta = ShaderMeta {
            images: image_names,
            uniforms: BasicUniforms::block_layout(),
        };
        let program = miniquad::Shader::new(
            ctx,
            vertex_src.as_ref(),
            fragment_src.as_ref(),
            meta.clone(),
        )?;
        Ok(Self {
            program,
            meta,
            _marker: Default::default(),
        })
    }

    pub fn program(&self) -> miniquad::Shader {
        self.program
    }
}
