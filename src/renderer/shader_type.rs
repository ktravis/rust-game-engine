use glam::{Mat3, Mat4, Vec2, Vec3, Vec4};
use shadertype_derive::{shader_uniform_type, ShaderType, VertexDataType, VertexInput};

use crate::color::Color;

pub trait ShaderType {
    const NAME: &'static str;
}

pub trait ShaderTypeAligned: ShaderType {
    const ALIGNMENT: usize;
}

macro_rules! impl_basic_shader_type {
    ($t:ty, $name:expr) => {
        impl ShaderType for $t {
            const NAME: &'static str = $name;
        }
    };
    ($t:ty, $name:expr, align: $align:expr) => {
        impl_basic_shader_type!($t, $name);
        impl ShaderTypeAligned for $t {
            const ALIGNMENT: usize = $align;
        }
    };
}

impl_basic_shader_type!(i32, "i32", align: 4);
impl_basic_shader_type!(u32, "u32", align: 4);
impl_basic_shader_type!(f32, "f32", align: 4);
impl_basic_shader_type!(Mat3, "mat3x3<f32>", align: 16);
impl_basic_shader_type!(Mat4, "mat4x4<f32>", align: 16);
impl_basic_shader_type!(Vec2, "vec2<f32>", align: 8);
impl_basic_shader_type!(Vec3, "vec3<f32>", align: 16);
impl_basic_shader_type!(Vec4, "vec4<f32>", align: 16);
impl_basic_shader_type!(Color, Vec4::NAME, align: Vec4::ALIGNMENT);

pub trait VertexInput: VertexDataType {
    fn definition() -> String;
}

#[repr(C)]
#[derive(ShaderType, VertexDataType, VertexInput)]
pub struct InstanceInput {
    uv_scale: Vec2,
    uv_offset: Vec2,
    tint: Vec4,
    model: Mat4,
    normal: Mat4,
}

pub trait VertexDataType {
    const N: usize = 1;

    fn wgsl_fields(field_name: &'static str) -> Vec<String>;
}

pub trait ShaderUniformType {
    fn definition() -> String;
}

macro_rules! impl_vertex_data_type {
    ($t:ty) => {
        impl VertexDataType for $t {
            fn wgsl_fields(name: &'static str) -> Vec<String> {
                vec![format!("{}: {}", name, <Self as ShaderType>::NAME)]
            }
        }
    };
    ($t:ty: [ $($rest:ty),+ ]) => {
        impl VertexDataType for $t {
            const N: usize = ($(<$rest as ShaderType>::N +)*);
            fn wgsl_fields(name: &'static str) -> Vec<String> {
                [
                    $(<$rest as ShaderType>::NAME,)*
                ].into_iter()
                    .enumerate()
                    .map(|(i, t)| format!("{}_{}: {}", name, i+1, t))
                    .collect::<Vec<String>>()
           }
        }
    };
    ($t:ty: [ $x:ty; $n:literal ]) => {
        impl VertexDataType for $t {
            const N: usize = $n;
            fn wgsl_fields(name: &'static str) -> Vec<String> {
                [<$x as ShaderType>::NAME; $n].into_iter()
                    .enumerate()
                    .map(|(i, t)| format!("{}_{}: {}", name, i+1, t))
                    .collect::<Vec<String>>()
            }
        }
    };
}

impl_vertex_data_type!(Vec2);
impl_vertex_data_type!(Vec3);
impl_vertex_data_type!(Vec4);
impl_vertex_data_type!(Color);
impl_vertex_data_type!(Mat4: [Vec4; 4]);

#[derive(Debug, PartialEq)]
#[shader_uniform_type]
pub struct GlobalUniforms {
    pub time: f32,
    #[skip]
    pub _pad_time: [u8; 4u32 as usize],
    pub screen_size: Vec2,
}

#[cfg(test)]
mod testing {
    use super::InstanceInput;
    use super::VertexInput;

    #[test]
    fn test_blah() {
        println!("{}", InstanceInput::definition());
        assert!(false);
    }
}
