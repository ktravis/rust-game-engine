use glam::{Mat3, Mat3A, Mat4, Vec2, Vec3, Vec4};
use shadertype_derive::shader_uniform_data;

use crate::{color::Color, geom::Rect};

pub trait ShaderBuiltin {}

pub trait ShaderNamedType {
    fn wgsl_name() -> String;
}

pub trait ShaderDefinedType {
    fn definitions() -> Vec<String>;
}

impl ShaderDefinedType for () {
    fn definitions() -> Vec<String> {
        vec![]
    }
}

impl<T: ShaderBuiltin> ShaderDefinedType for T {
    fn definitions() -> Vec<String> {
        vec![]
    }
}

pub trait ShaderTypeAligned {
    const ALIGNMENT: usize;
}

pub trait ShaderTypeDefinitions {
    fn definitions() -> Vec<String> {
        vec![]
    }
}

macro_rules! impl_shader_builtin {
    ($t:ty, $name:expr) => {
        impl ShaderNamedType for $t {
            fn wgsl_name() -> String {
                $name.into()
            }
        }
        impl ShaderBuiltin for $t {}
    };
    ($t:ty, $name:expr, align: $align:expr) => {
        impl_shader_builtin!($t, $name);
        impl ShaderTypeAligned for $t {
            const ALIGNMENT: usize = $align;
        }
    };
}

impl_shader_builtin!(i32, "i32", align: 4);
impl_shader_builtin!(u32, "u32", align: 4);
impl_shader_builtin!(f32, "f32", align: 4);
impl_shader_builtin!(Mat3, "mat3x3<f32>", align: 16);
impl_shader_builtin!(Mat3A, "mat3x3<f32>", align: 16);
impl_shader_builtin!(Mat4, "mat4x4<f32>", align: 16);
impl_shader_builtin!(Vec2, "vec2<f32>", align: 8);
impl_shader_builtin!(Vec3, "vec3<f32>", align: 16);
impl_shader_builtin!(Vec4, "vec4<f32>", align: 16);
impl_shader_builtin!(Color, "vec4<f32>", align: 16);

impl<T: ShaderNamedType, const N: usize> ShaderNamedType for [T; N] {
    fn wgsl_name() -> String {
        std::format!("array<{}, {}>", T::wgsl_name(), N)
    }
}

impl<T: ShaderDefinedType, const N: usize> ShaderDefinedType for [T; N] {
    fn definitions() -> Vec<String> {
        T::definitions()
    }
}

impl<T: ShaderTypeAligned, const N: usize> ShaderTypeAligned for [T; N] {
    const ALIGNMENT: usize = T::ALIGNMENT;
}

pub struct BufferLayout {
    pub attributes: Vec<wgpu::VertexAttribute>,
    pub step_mode: wgpu::VertexStepMode,
    pub array_stride: wgpu::BufferAddress,
}

impl BufferLayout {
    pub fn to_wgpu<'a>(&'a self) -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: self.array_stride,
            step_mode: self.step_mode,
            attributes: &self.attributes,
        }
    }
}

pub trait VertexInput: bytemuck::Zeroable + bytemuck::Pod {
    fn definition(location_offset: u32) -> String;
    fn vertex_attributes() -> Vec<wgpu::VertexAttribute>;

    fn step_mode() -> wgpu::VertexStepMode {
        wgpu::VertexStepMode::Vertex
    }

    fn vertex_buffer_layout(location_offset: u32) -> BufferLayout {
        let attributes = Self::vertex_attributes()
            .iter()
            .map(|a| wgpu::VertexAttribute {
                shader_location: location_offset + a.shader_location,
                ..a.clone()
            })
            .collect();

        BufferLayout {
            attributes,
            step_mode: Self::step_mode(),
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
        }
    }

    fn next_offset() -> u32 {
        let vv = Self::vertex_attributes();
        vv.last()
            .map(|a| a.shader_location + 1)
            .unwrap_or(vv.len() as u32)
    }
}

fn vertex_format_wgsl_type_name(f: wgpu::VertexFormat) -> &'static str {
    use wgpu::VertexFormat::*;
    match f {
        Uint8 => "u32",
        Uint8x2 => "vec2<u32>",
        Uint8x4 => "vec4<u32>",
        Sint8 => "i32",
        Sint8x2 => "vec2<i32>",
        Sint8x4 => "vec4<i32>",
        Unorm8 => "f32",
        Unorm8x2 => "vec2<f32>",
        Unorm8x4 => "vec4<f32>",
        Snorm8 => "f32",
        Snorm8x2 => "vec2<f32>",
        Snorm8x4 => "vec4<f32>",
        Uint16 => "u32",
        Uint16x2 => "vec2<u32>",
        Uint16x4 => "vec4<u32>",
        Sint16 => "i32",
        Sint16x2 => "vec2<i32>",
        Sint16x4 => "vec4<i32>",
        Unorm16 => "f32",
        Unorm16x2 => "vec2<f32>",
        Unorm16x4 => "vec4<f32>",
        Snorm16 => "f32",
        Snorm16x2 => "vec2<f32>",
        Snorm16x4 => "vec4<f32>",
        Float16 => "f32",
        Float16x2 => "vec2<f32>",
        Float16x4 => "vec4<f32>",
        Float32 => "f32",
        Float32x2 => "vec2<f32>",
        Float32x3 => "vec3<f32>",
        Float32x4 => "vec4<f32>",
        Uint32 => "u32",
        Uint32x2 => "vec2<u32>",
        Uint32x3 => "vec3<u32>",
        Uint32x4 => "vec4<u32>",
        Sint32 => "i32",
        Sint32x2 => "vec2<i32>",
        Sint32x3 => "vec3<i32>",
        Sint32x4 => "vec4<i32>",
        Float64 => "f32",
        Float64x2 => "vec2<f32>",
        Float64x3 => "vec3<f32>",
        Float64x4 => "vec4<f32>",
        Unorm10_10_10_2 => "vec4<f32>",
        Unorm8x4Bgra => "vec4<f32>",
    }
}

pub trait VertexDataType {
    const N: usize = 1;

    fn wgsl_fields(field_name: &'static str) -> Vec<String>;
    fn vertex_formats() -> Vec<wgpu::VertexFormat>;
}

pub(crate) fn define_wgsl_struct(
    name: &'static str,
    fields_iter: impl IntoIterator<Item = (usize, Vec<String>)>,
    location_offset: u32,
) -> String {
    format!(
        "struct {} {{\n{}}}",
        name,
        fields_iter
            .into_iter()
            .map(|(loc, f)| {
                f.iter()
                    .enumerate()
                    .map(|(i, s)| {
                        format!(
                            "  @location({}) {},\n",
                            loc + i + location_offset as usize,
                            s
                        )
                    })
                    .collect::<String>()
            })
            .collect::<String>()
    )
}

pub(crate) fn expand_vertex_attributes(
    iter: impl IntoIterator<Item = (usize, Vec<wgpu::VertexFormat>)>,
) -> Vec<wgpu::VertexAttribute> {
    let mut offset = 0;
    iter.into_iter()
        .flat_map(|(loc, x)| {
            x.into_iter()
                .enumerate()
                .map(|(i, format)| {
                    let a = ::wgpu::VertexAttribute {
                        format,
                        shader_location: (loc + i) as u32,
                        offset,
                    };
                    offset += format.size();
                    a
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>()
}

pub trait ShaderUniformData {
    type Raw: bytemuck::Pod + bytemuck::Zeroable;
    fn definition() -> String;
    fn raw(&self) -> Self::Raw;
}

macro_rules! impl_vertex_data_type {
    (@count) => {0usize};
    (@count $head:expr, $($tail:expr,)*) => {
        1usize + impl_vertex_data_type!(@count $($tail,)*)
    };
    ($t:ty, $f:expr) => {
        impl VertexDataType for $t {
            fn wgsl_fields(name: &'static str) -> Vec<String> {
                vec![format!("{}: {}", name, vertex_format_wgsl_type_name($f))]
            }
            fn vertex_formats() -> Vec<wgpu::VertexFormat> {
                vec![$f]
            }
        }
    };
    ($t:ty: [ $($rest:expr),+ ]) => {
        impl VertexDataType for $t {
            const N: usize = ($(<$rest as ShaderType>::N +)*);
            fn wgsl_fields(name: &'static str) -> Vec<String> {
                [
                    $($rest,)*
                ].into_iter()
                    .enumerate()
                    .map(|(i, t)| format!("{}_{}: {}", name, i+1, vertex_format_wgsl_type_name(t)))
                    .collect::<Vec<String>>()
           }
            fn vertex_formats() -> Vec<wgpu::VertexFormat> {
                vec![
                    $($rest),*
                ]
            }
        }
    };
    ($t:ty: [ $x:expr; $n:literal ]) => {
        impl VertexDataType for $t {
            const N: usize = $n;
            fn wgsl_fields(name: &'static str) -> Vec<String> {
                (0..$n).map(|i| format!("{}_{}: {}", name, i+1, vertex_format_wgsl_type_name($x)))
                    .collect::<Vec<String>>()
            }
            fn vertex_formats() -> Vec<wgpu::VertexFormat> {
                vec![$x; $n]
            }
        }
    };
    ($t:ty: { $( $field:ident => $tp:expr, )+ }) => {
        impl VertexDataType for $t {
            const N: usize = impl_vertex_data_type!(@count $($tp,)*);
            fn wgsl_fields(name: &'static str) -> Vec<String> {
                vec![
                    $(format!("{}_{}: {}", name, stringify!($field), vertex_format_wgsl_type_name($tp)),)*
                ]
            }
            fn vertex_formats() -> Vec<wgpu::VertexFormat> {
                vec![
                    $($tp),*
                ]
            }
        }
    };
}

impl_vertex_data_type!(u32, wgpu::VertexFormat::Uint32);
impl_vertex_data_type!(f32, wgpu::VertexFormat::Float32);
impl_vertex_data_type!(Vec2, wgpu::VertexFormat::Float32x2);
impl_vertex_data_type!([f32; 2], wgpu::VertexFormat::Float32x2);
impl_vertex_data_type!(Vec3, wgpu::VertexFormat::Float32x3);
impl_vertex_data_type!([f32; 3], wgpu::VertexFormat::Float32x3);
impl_vertex_data_type!(Vec4, wgpu::VertexFormat::Float32x4);
impl_vertex_data_type!([f32; 4], wgpu::VertexFormat::Float32x4);
impl_vertex_data_type!(Color, wgpu::VertexFormat::Float32x4);
impl_vertex_data_type!(Mat3: [wgpu::VertexFormat::Float32x3; 3]);
impl_vertex_data_type!(Mat4: [wgpu::VertexFormat::Float32x4; 4]);
impl_vertex_data_type!(Rect: {
    scale => wgpu::VertexFormat::Float32x2,
    offset => wgpu::VertexFormat::Float32x2,
});

#[macro_export]
macro_rules! wgsl {
    ($s:literal) => {
        $s
    };
}

// TODO: should be able to define methods/helper functions for wgsl that will be included in the
// definition
// example:
//   let model_transform = mat4x4<f32>(
//       instance.transform_1,
//       instance.transform_2,
//       instance.transform_3,
//       instance.transform_4,
//   );

macro_rules! impl_shader_uniforms_tuple {
    ($($T:ident),*) => {
        impl<$($T: ShaderNamedType),*> ShaderNamedType for ($($T),*) {
            fn wgsl_name() -> String {
                "".into()
            }
        }
        impl<$($T: ShaderDefinedType),*> ShaderDefinedType for ($($T),*) {
            fn definitions() -> Vec<String> {
                [$(<$T as ShaderDefinedType>::definitions()),*].into_iter().flatten().collect()
            }
        }
    };
}
impl_shader_uniforms_tuple!(T1, T2);
impl_shader_uniforms_tuple!(T1, T2, T3);
impl_shader_uniforms_tuple!(T1, T2, T3, T4);
impl_shader_uniforms_tuple!(T1, T2, T3, T4, T5);
impl_shader_uniforms_tuple!(T1, T2, T3, T4, T5, T6);

pub trait VertexInputs {
    fn definition() -> String;
    fn locations_count() -> u32;

    fn vertex_buffer_layouts() -> Vec<BufferLayout>;
}

impl<V: VertexInput> VertexInputs for V {
    fn definition() -> String {
        <V as VertexInput>::definition(0)
    }
    fn locations_count() -> u32 {
        // should this be 1 + the last attribute's offset instead? does it matter?
        <V as VertexInput>::vertex_attributes().len() as u32
    }

    fn vertex_buffer_layouts() -> Vec<BufferLayout> {
        vec![V::vertex_buffer_layout(0)]
    }
}

macro_rules! impl_vertex_inputs_tuple {
    ($($T:ident),*) => {
        impl<$($T: VertexInput),*> VertexInputs for ($($T),*) {
            fn locations_count() -> u32 {
                [
                    $(<$T as VertexInputs>::locations_count()),*
                ].iter().sum()
            }
            fn definition() -> String {
                let counts = [
                    $(<$T as VertexInputs>::locations_count()),*
                ];
                let calls = [
                    $(
                        |o: u32| -> String {
                            <$T as VertexInput>::definition(o)
                        }
                    ),*
                ];
                let mut offset = 0;
                counts.iter().zip(calls.into_iter()).map(|(o, f)| {
                    let s = f(offset);
                    offset += o;
                    s
                }).collect::<Vec<String>>().join("\n")
            }

            fn vertex_buffer_layouts() -> Vec<BufferLayout> {
                let counts = [
                    $(<$T as VertexInputs>::locations_count()),*
                ];
                let calls = [
                    $(
                        |o: u32| -> BufferLayout {
                            <$T as VertexInput>::vertex_buffer_layout(o)
                        }
                    ),*
                ];
                let mut offset = 0;
                counts.iter().zip(calls.into_iter()).map(|(o, f)| {
                    let s = f(offset);
                    offset += o;
                    s
                }).collect()
            }
        }
    };
}

impl_vertex_inputs_tuple!(T1, T2);
impl_vertex_inputs_tuple!(T1, T2, T3);
impl_vertex_inputs_tuple!(T1, T2, T3, T4);
impl_vertex_inputs_tuple!(T1, T2, T3, T4, T5);
impl_vertex_inputs_tuple!(T1, T2, T3, T4, T5, T6);

#[macro_export]
macro_rules! create_shader_ex {
    ($display:expr, {
        name => $name:literal,
        src => $src:expr,
        inputs => $inputs:ty,
        $(defs => [
            $($defs:ty),* $(,)?
        ], )?
        bind_groups => {
            $($bname:ident => $btype:ty,)*
        } $(,)?
    }) => {
        {
            let mut u = std::collections::HashSet::new();
            let definitions = ([
                $(
                    $(<$defs as ::rust_game_engine::renderer::shader_type::ShaderDefinedType>::definitions(),)*
                )?
                $(<$btype as ::rust_game_engine::renderer::shader_type::ShaderDefinedType>::definitions(),)*
            ] as [Vec<String>; _]).into_iter().flatten().filter(|d: &String| {
                let seen = u.contains(d);
                u.insert(d.clone());
                !seen
            }).collect::<Vec<_>>().join("\n");
            let mut i = 0;
            $display
                .device()
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some($name),
                    source: wgpu::ShaderSource::Wgsl(dbg!(std::borrow::Cow::Owned([
                        <$inputs as ::rust_game_engine::renderer::shader_type::VertexInputs>::definition(),
                        definitions,
                        $({
                            i += 1;
                            let s = <$btype as ::rust_game_engine::renderer::bindings::Bindable>::shader_bindings(stringify!($bname), i-1, &mut 0);
                            s
                        },)*
                        $src,
                    ].concat()))),
                })
        }
    };
}

#[repr(C, align(16))]
#[derive(Copy, Clone, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
#[shader_uniform_data]
pub struct GlobalUniforms {
    pub time: f32,
    #[skip]
    pub _pad_time: [u8; 4u32 as usize],
    pub screen_size: Vec2,
}

impl ShaderUniformData for Vec3 {
    type Raw = Self;

    fn definition() -> String {
        "".into()
    }

    fn raw(&self) -> Self::Raw {
        *self
    }
}

impl ShaderUniformData for Mat4 {
    type Raw = Self;

    fn definition() -> String {
        "".into()
    }

    fn raw(&self) -> Self::Raw {
        *self
    }
}
