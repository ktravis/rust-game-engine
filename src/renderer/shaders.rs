pub mod ssao_from_depth {
    #[allow(unused)]
    ///Information about the globals within the module, exposed as constants and functions.
    pub mod globals {
        #[allow(unused)]
        use super::*;
        ///Information about the `t_diffuse` global variable within this shader module.
        pub mod t_diffuse {
            #[allow(unused)]
            use super::*;
            pub const NAME: &'static str = "t_diffuse";
            pub const GROUP: u32 = 0u32;
            pub const BINDING: u32 = 0u32;
        }
        ///Information about the `s_diffuse` global variable within this shader module.
        pub mod s_diffuse {
            #[allow(unused)]
            use super::*;
            pub const NAME: &'static str = "s_diffuse";
            pub const GROUP: u32 = 0u32;
            pub const BINDING: u32 = 1u32;
        }
        ///Information about the `global_uniforms` global variable within this shader module.
        pub mod global_uniforms {
            #[allow(unused)]
            use super::*;
            pub const NAME: &'static str = "global_uniforms";
            pub type Ty = super::super::super::global::types::GlobalUniforms;
            pub const GROUP: u32 = 1u32;
            pub const BINDING: u32 = 0u32;
            pub fn layout() -> ::wgpu::BindGroupLayoutDescriptor<'static> {
                ::wgpu::BindGroupLayoutDescriptor {
                    label: Some("global_uniforms"),
                    entries: &[
                        ::wgpu::BindGroupLayoutEntry {
                            binding: 0u32,
                            visibility: ::wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: ::wgpu::BindingType::Buffer {
                                ty: ::wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                }
            }
        }
        ///Information about the `view_proj_uniforms` global variable within this shader module.
        pub mod view_proj_uniforms {
            #[allow(unused)]
            use super::*;
            pub const NAME: &'static str = "view_proj_uniforms";
            pub type Ty = super::super::super::global::types::ViewProjectionUniforms;
            pub const GROUP: u32 = 2u32;
            pub const BINDING: u32 = 0u32;
            pub fn layout() -> ::wgpu::BindGroupLayoutDescriptor<'static> {
                ::wgpu::BindGroupLayoutDescriptor {
                    label: Some("view_proj_uniforms"),
                    entries: &[
                        ::wgpu::BindGroupLayoutEntry {
                            binding: 0u32,
                            visibility: ::wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: ::wgpu::BindingType::Buffer {
                                ty: ::wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                }
            }
        }
        ///Information about the `depth_buffer` global variable within this shader module.
        pub mod depth_buffer {
            #[allow(unused)]
            use super::*;
            pub const NAME: &'static str = "depth_buffer";
            pub const GROUP: u32 = 3u32;
            pub const BINDING: u32 = 0u32;
        }
        ///Information about the `depth_buffer_sampler` global variable within this shader module.
        pub mod depth_buffer_sampler {
            #[allow(unused)]
            use super::*;
            pub const NAME: &'static str = "depth_buffer_sampler";
            pub const GROUP: u32 = 3u32;
            pub const BINDING: u32 = 1u32;
        }
        ///Information about the `kernel` global variable within this shader module.
        pub mod kernel {
            #[allow(unused)]
            use super::*;
            pub const NAME: &'static str = "kernel";
            pub type Ty = Kernel;
            pub const GROUP: u32 = 4u32;
            pub const BINDING: u32 = 0u32;
            pub fn layout() -> ::wgpu::BindGroupLayoutDescriptor<'static> {
                ::wgpu::BindGroupLayoutDescriptor {
                    label: Some("kernel"),
                    entries: &[
                        ::wgpu::BindGroupLayoutEntry {
                            binding: 0u32,
                            visibility: ::wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: ::wgpu::BindingType::Buffer {
                                ty: ::wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                }
            }
        }
        ///Information about the `ssao_noise` global variable within this shader module.
        pub mod ssao_noise {
            #[allow(unused)]
            use super::*;
            pub const NAME: &'static str = "ssao_noise";
            pub const GROUP: u32 = 5u32;
            pub const BINDING: u32 = 0u32;
        }
        ///Information about the `ssao_noise_sampler` global variable within this shader module.
        pub mod ssao_noise_sampler {
            #[allow(unused)]
            use super::*;
            pub const NAME: &'static str = "ssao_noise_sampler";
            pub const GROUP: u32 = 5u32;
            pub const BINDING: u32 = 1u32;
        }
        ///Contains the following bindings: ssao_noise, ssao_noise_sampler
        pub mod group5 {
            #[allow(unused)]
            use super::*;
            pub const GROUP: u32 = 5u32;
            pub fn layout() -> ::wgpu::BindGroupLayoutDescriptor<'static> {
                ::wgpu::BindGroupLayoutDescriptor {
                    label: Some("group5"),
                    entries: &[
                        ::wgpu::BindGroupLayoutEntry {
                            binding: 0u32,
                            visibility: ::wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: ::wgpu::TextureViewDimension::D2,
                                sample_type: ::wgpu::TextureSampleType::Float {
                                    filterable: true,
                                },
                            },
                            count: None,
                        },
                        ::wgpu::BindGroupLayoutEntry {
                            binding: 1u32,
                            visibility: ::wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: ::wgpu::BindingType::Sampler(
                                ::wgpu::SamplerBindingType::Filtering,
                            ),
                            count: None,
                        },
                    ],
                }
            }
        }
        ///Contains the following bindings: depth_buffer, depth_buffer_sampler
        pub mod group3 {
            #[allow(unused)]
            use super::*;
            pub const GROUP: u32 = 3u32;
            pub fn layout() -> ::wgpu::BindGroupLayoutDescriptor<'static> {
                ::wgpu::BindGroupLayoutDescriptor {
                    label: Some("group3"),
                    entries: &[
                        ::wgpu::BindGroupLayoutEntry {
                            binding: 0u32,
                            visibility: ::wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: ::wgpu::TextureViewDimension::D2,
                                sample_type: ::wgpu::TextureSampleType::Depth,
                            },
                            count: None,
                        },
                        ::wgpu::BindGroupLayoutEntry {
                            binding: 1u32,
                            visibility: ::wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: ::wgpu::BindingType::Sampler(
                                ::wgpu::SamplerBindingType::Filtering,
                            ),
                            count: None,
                        },
                    ],
                }
            }
        }
        ///Contains the following bindings: t_diffuse, s_diffuse
        pub mod group0 {
            #[allow(unused)]
            use super::*;
            pub const GROUP: u32 = 0u32;
            pub fn layout() -> ::wgpu::BindGroupLayoutDescriptor<'static> {
                ::wgpu::BindGroupLayoutDescriptor {
                    label: Some("group0"),
                    entries: &[
                        ::wgpu::BindGroupLayoutEntry {
                            binding: 0u32,
                            visibility: ::wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: ::wgpu::TextureViewDimension::D2,
                                sample_type: ::wgpu::TextureSampleType::Float {
                                    filterable: true,
                                },
                            },
                            count: None,
                        },
                        ::wgpu::BindGroupLayoutEntry {
                            binding: 1u32,
                            visibility: ::wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: ::wgpu::BindingType::Sampler(
                                ::wgpu::SamplerBindingType::Filtering,
                            ),
                            count: None,
                        },
                    ],
                }
            }
        }
    }
    #[allow(unused)]
    ///Information about the constants within the module, exposed as constants and functions.
    pub mod constants {
        #[allow(unused)]
        use super::*;
        #[allow(non_snake_case)]
        ///Information about the `KERNEL_SIZE` constant variable within this shader module.
        pub mod KERNEL_SIZE {
            #[allow(unused)]
            use super::*;
            pub const NAME: &'static str = "KERNEL_SIZE";
            pub const VALUE: u32 = 64u32;
        }
    }
    #[allow(unused)]
    ///Information about the entry points within the module, exposed as constants and functions.
    pub mod entry_points {
        #[allow(unused)]
        use super::*;
        pub mod vs_main {
            pub const NAME: &'static str = "vs_main";
            ///The sourcecode for the shader, as a constant string, excluding any other entry points. This is useful when the `minify` feature is enabled for this crate, as it allows more aggressive minification to be performed with the knowledge of the specific entry point that will be used.
            pub const EXCLUSIVE_SOURCE: &'static str = "struct GlobalUniformsX_naga_oil_mod_XM5WG6YTBNQX {\n    time: f32,\n    screen_size: vec2<f32>,\n}\n\nstruct ViewProjectionUniformsX_naga_oil_mod_XM5WG6YTBNQX {\n    view: mat4x4<f32>,\n    projection: mat4x4<f32>,\n    camera_pos: vec3<f32>,\n    inverse_view: mat4x4<f32>,\n}\n\nstruct Kernel {\n    items: array<vec4<f32>, 64>,\n    radius: f32,\n    bias: f32,\n    noise_texture_scale: vec2<f32>,\n    aspect_ratio: f32,\n    tan_half_fov: f32,\n    inverse_proj: mat4x4<f32>,\n}\n\nstruct VertexInput {\n    @location(0) position: vec4<f32>,\n    @location(1) tex_coords: vec2<f32>,\n}\n\nstruct InstanceInput {\n    @location(2) uv_scale: vec2<f32>,\n    @location(3) uv_offset: vec2<f32>,\n    @location(4) tint: vec4<f32>,\n    @location(5) model_1_: vec4<f32>,\n    @location(6) model_2_: vec4<f32>,\n    @location(7) model_3_: vec4<f32>,\n    @location(8) model_4_: vec4<f32>,\n}\n\nstruct VertexOutput {\n    @builtin(position) clip_position: vec4<f32>,\n    @location(0) tex_coords: vec2<f32>,\n}\n\nconst KERNEL_SIZE: u32 = 64u;\n\n@group(0) @binding(0) \nvar t_diffuse: texture_2d<f32>;\n@group(0) @binding(1) \nvar s_diffuse: sampler;\n@group(1) @binding(0) \nvar<uniform> global_uniforms: GlobalUniformsX_naga_oil_mod_XM5WG6YTBNQX;\n@group(2) @binding(0) \nvar<uniform> view_proj_uniforms: ViewProjectionUniformsX_naga_oil_mod_XM5WG6YTBNQX;\n@group(3) @binding(0) \nvar depth_buffer: texture_depth_2d;\n@group(3) @binding(1) \nvar depth_buffer_sampler: sampler;\n@group(4) @binding(0) \nvar<uniform> kernel: Kernel;\n@group(5) @binding(0) \nvar ssao_noise: texture_2d<f32>;\n@group(5) @binding(1) \nvar ssao_noise_sampler: sampler;\n\nfn reconstructPosition(coords: vec2<f32>) -> vec3<f32> {\n    let x = ((coords.x * 2f) - 1f);\n    let y = (((1f - coords.y) * 2f) - 1f);\n    let z = textureSample(depth_buffer, depth_buffer_sampler, coords);\n    let position_s = vec4<f32>(x, y, z, 1f);\n    let _e20 = kernel.inverse_proj;\n    let position_v = (_e20 * position_s);\n    return (position_v.xyz / vec3(position_v.w));\n}\n\nfn normalFromDepth(center: vec3<f32>, coords_1: vec2<f32>) -> vec3<f32> {\n    var y1_: vec3<f32>;\n    var y2_: vec3<f32>;\n    var x1_: vec3<f32>;\n    var x2_: vec3<f32>;\n\n    let _e6 = global_uniforms.screen_size;\n    let _e9 = reconstructPosition((coords_1 + (vec2<f32>(0f, 1f) / _e6)));\n    let _e15 = global_uniforms.screen_size;\n    let _e18 = reconstructPosition((coords_1 + (vec2<f32>(0f, -1f) / _e15)));\n    y1_ = _e9;\n    y2_ = center;\n    if (abs((_e18.z - center.z)) < abs((_e9.z - center.z))) {\n        y1_ = center;\n        y2_ = _e18;\n    }\n    let _e36 = global_uniforms.screen_size;\n    let _e39 = reconstructPosition((coords_1 + (vec2<f32>(-1f, 0f) / _e36)));\n    let _e45 = global_uniforms.screen_size;\n    let _e48 = reconstructPosition((coords_1 + (vec2<f32>(1f, 0f) / _e45)));\n    x1_ = _e39;\n    x2_ = center;\n    if (abs((_e48.z - center.z)) < abs((_e39.z - center.z))) {\n        x1_ = center;\n        x2_ = _e48;\n    }\n    let _e60 = x2_;\n    let _e61 = x1_;\n    let _e63 = y2_;\n    let _e64 = y1_;\n    return normalize(cross((_e60 - _e61), (_e63 - _e64)));\n}\n\n@vertex \nfn vs_main(vertex: VertexInput, instance: InstanceInput) -> VertexOutput {\n    var out: VertexOutput;\n    var model: vec4<f32>;\n\n    let model_transform = mat4x4<f32>(instance.model_1_, instance.model_2_, instance.model_3_, instance.model_4_);\n    out.tex_coords = (instance.uv_offset + (instance.uv_scale * vertex.tex_coords));\n    model = vertex.position;\n    let _e18 = model.x;\n    model.x = ((_e18 * 2f) - 1f);\n    let _e25 = model.y;\n    model.y = ((_e25 * 2f) - 1f);\n    let model_view = model;\n    out.clip_position = model_view;\n    let _e32 = out;\n    return _e32;\n}\n";
        }
        pub mod fs_main {
            pub const NAME: &'static str = "fs_main";
            ///The sourcecode for the shader, as a constant string, excluding any other entry points. This is useful when the `minify` feature is enabled for this crate, as it allows more aggressive minification to be performed with the knowledge of the specific entry point that will be used.
            pub const EXCLUSIVE_SOURCE: &'static str = "struct GlobalUniformsX_naga_oil_mod_XM5WG6YTBNQX {\n    time: f32,\n    screen_size: vec2<f32>,\n}\n\nstruct ViewProjectionUniformsX_naga_oil_mod_XM5WG6YTBNQX {\n    view: mat4x4<f32>,\n    projection: mat4x4<f32>,\n    camera_pos: vec3<f32>,\n    inverse_view: mat4x4<f32>,\n}\n\nstruct Kernel {\n    items: array<vec4<f32>, 64>,\n    radius: f32,\n    bias: f32,\n    noise_texture_scale: vec2<f32>,\n    aspect_ratio: f32,\n    tan_half_fov: f32,\n    inverse_proj: mat4x4<f32>,\n}\n\nstruct VertexInput {\n    @location(0) position: vec4<f32>,\n    @location(1) tex_coords: vec2<f32>,\n}\n\nstruct InstanceInput {\n    @location(2) uv_scale: vec2<f32>,\n    @location(3) uv_offset: vec2<f32>,\n    @location(4) tint: vec4<f32>,\n    @location(5) model_1_: vec4<f32>,\n    @location(6) model_2_: vec4<f32>,\n    @location(7) model_3_: vec4<f32>,\n    @location(8) model_4_: vec4<f32>,\n}\n\nstruct VertexOutput {\n    @builtin(position) clip_position: vec4<f32>,\n    @location(0) tex_coords: vec2<f32>,\n}\n\nconst KERNEL_SIZE: u32 = 64u;\n\n@group(0) @binding(0) \nvar t_diffuse: texture_2d<f32>;\n@group(0) @binding(1) \nvar s_diffuse: sampler;\n@group(1) @binding(0) \nvar<uniform> global_uniforms: GlobalUniformsX_naga_oil_mod_XM5WG6YTBNQX;\n@group(2) @binding(0) \nvar<uniform> view_proj_uniforms: ViewProjectionUniformsX_naga_oil_mod_XM5WG6YTBNQX;\n@group(3) @binding(0) \nvar depth_buffer: texture_depth_2d;\n@group(3) @binding(1) \nvar depth_buffer_sampler: sampler;\n@group(4) @binding(0) \nvar<uniform> kernel: Kernel;\n@group(5) @binding(0) \nvar ssao_noise: texture_2d<f32>;\n@group(5) @binding(1) \nvar ssao_noise_sampler: sampler;\n\nfn reconstructPosition(coords: vec2<f32>) -> vec3<f32> {\n    let x = ((coords.x * 2f) - 1f);\n    let y = (((1f - coords.y) * 2f) - 1f);\n    let z = textureSample(depth_buffer, depth_buffer_sampler, coords);\n    let position_s = vec4<f32>(x, y, z, 1f);\n    let _e20 = kernel.inverse_proj;\n    let position_v = (_e20 * position_s);\n    return (position_v.xyz / vec3(position_v.w));\n}\n\nfn normalFromDepth(center: vec3<f32>, coords_1: vec2<f32>) -> vec3<f32> {\n    var y1_: vec3<f32>;\n    var y2_: vec3<f32>;\n    var x1_: vec3<f32>;\n    var x2_: vec3<f32>;\n\n    let _e6 = global_uniforms.screen_size;\n    let _e9 = reconstructPosition((coords_1 + (vec2<f32>(0f, 1f) / _e6)));\n    let _e15 = global_uniforms.screen_size;\n    let _e18 = reconstructPosition((coords_1 + (vec2<f32>(0f, -1f) / _e15)));\n    y1_ = _e9;\n    y2_ = center;\n    if (abs((_e18.z - center.z)) < abs((_e9.z - center.z))) {\n        y1_ = center;\n        y2_ = _e18;\n    }\n    let _e36 = global_uniforms.screen_size;\n    let _e39 = reconstructPosition((coords_1 + (vec2<f32>(-1f, 0f) / _e36)));\n    let _e45 = global_uniforms.screen_size;\n    let _e48 = reconstructPosition((coords_1 + (vec2<f32>(1f, 0f) / _e45)));\n    x1_ = _e39;\n    x2_ = center;\n    if (abs((_e48.z - center.z)) < abs((_e39.z - center.z))) {\n        x1_ = center;\n        x2_ = _e48;\n    }\n    let _e60 = x2_;\n    let _e61 = x1_;\n    let _e63 = y2_;\n    let _e64 = y1_;\n    return normalize(cross((_e60 - _e61), (_e63 - _e64)));\n}\n\n@fragment \nfn fs_main(in: VertexOutput) -> @location(0) f32 {\n    var view_pos: vec3<f32>;\n    var view_space_normal: vec3<f32>;\n    var random_vec: vec3<f32>;\n    var occlusion: f32 = 0f;\n    var i: i32 = 0i;\n    var sample: vec3<f32>;\n    var offset: vec4<f32>;\n    var sample_depth: f32;\n    var range_check: f32;\n\n    let _e4 = reconstructPosition(in.tex_coords);\n    view_pos = _e4;\n    let _e6 = view_pos;\n    let _e8 = normalFromDepth(_e6, in.tex_coords);\n    view_space_normal = _e8;\n    let _e14 = kernel.noise_texture_scale;\n    let _e18 = textureSample(ssao_noise, ssao_noise_sampler, (_e14 * in.tex_coords.xy));\n    random_vec = _e18.xyz;\n    let _e21 = random_vec;\n    let _e22 = view_space_normal;\n    let _e23 = random_vec;\n    let _e24 = view_space_normal;\n    let tangent = normalize((_e21 - (_e22 * dot(_e23, _e24))));\n    let _e29 = view_space_normal;\n    let bitangent = cross(_e29, tangent);\n    let _e31 = view_space_normal;\n    let TBN = mat3x3<f32>(tangent, bitangent, _e31);\n    loop {\n        let _e34 = i;\n        if (_e34 < 64i) {\n        } else {\n            break;\n        }\n        {\n            let _e37 = view_pos;\n            let _e41 = kernel.radius;\n            let _e45 = i;\n            let _e47 = kernel.items[_e45];\n            sample = (_e37.xyz + ((_e41 * TBN) * _e47.xyz));\n            let _e54 = view_proj_uniforms.projection;\n            let _e55 = sample;\n            offset = (_e54 * vec4<f32>(_e55, 1f));\n            let _e62 = offset.w;\n            let _e63 = offset.x;\n            offset.x = (_e63 / _e62);\n            let _e67 = offset.w;\n            let _e68 = offset.y;\n            offset.y = (_e68 / _e67);\n            let _e72 = offset.x;\n            offset.x = ((_e72 * 0.5f) + 0.5f);\n            let _e79 = offset.y;\n            offset.y = ((_e79 * 0.5f) + 0.5f);\n            let _e86 = offset.y;\n            offset.y = (1f - _e86);\n            let _e89 = offset;\n            let _e91 = reconstructPosition(_e89.xy);\n            sample_depth = _e91.z;\n            let _e98 = kernel.radius;\n            let _e100 = view_pos.z;\n            let _e101 = sample_depth;\n            range_check = smoothstep(0f, 1f, (_e98 / abs((_e100 - _e101))));\n            let _e107 = sample_depth;\n            let _e109 = sample.z;\n            let _e112 = kernel.bias;\n            if (_e107 >= (_e109 + _e112)) {\n                let _e116 = range_check;\n                let _e117 = range_check;\n                let _e119 = occlusion;\n                occlusion = (_e119 + (_e116 * _e117));\n            }\n        }\n        continuing {\n            let _e122 = i;\n            i = (_e122 + 1i);\n        }\n    }\n    let _e124 = occlusion;\n    occlusion = (1f - (_e124 / 64f));\n    let _e129 = occlusion;\n    occlusion = pow(_e129, 2f);\n    let _e132 = occlusion;\n    return _e132;\n}\n";
        }
    }
    #[allow(unused)]
    ///Equivalent Rust definitions of the types defined in this module.
    pub mod types {
        #[allow(unused, non_camel_case_types)]
        #[repr(C)]
        #[derive(Debug, PartialEq, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
        pub struct Kernel {
            pub items: [glam::f32::Vec4; 64u32 as usize],
            pub radius: f32,
            pub bias: f32,
            pub noise_texture_scale: glam::f32::Vec2,
            pub aspect_ratio: f32,
            pub tan_half_fov: f32,
            pub _pad_tan_half_fov: [u8; 8u32 as usize],
            pub inverse_proj: glam::f32::Mat4,
        }
    }
    #[allow(unused)]
    use types::*;
    ///The sourcecode for the shader, as a constant string.
    pub const SOURCE: &'static str = "struct GlobalUniformsX_naga_oil_mod_XM5WG6YTBNQX {\n    time: f32,\n    screen_size: vec2<f32>,\n}\n\nstruct ViewProjectionUniformsX_naga_oil_mod_XM5WG6YTBNQX {\n    view: mat4x4<f32>,\n    projection: mat4x4<f32>,\n    camera_pos: vec3<f32>,\n    inverse_view: mat4x4<f32>,\n}\n\nstruct Kernel {\n    items: array<vec4<f32>, 64>,\n    radius: f32,\n    bias: f32,\n    noise_texture_scale: vec2<f32>,\n    aspect_ratio: f32,\n    tan_half_fov: f32,\n    inverse_proj: mat4x4<f32>,\n}\n\nstruct VertexInput {\n    @location(0) position: vec4<f32>,\n    @location(1) tex_coords: vec2<f32>,\n}\n\nstruct InstanceInput {\n    @location(2) uv_scale: vec2<f32>,\n    @location(3) uv_offset: vec2<f32>,\n    @location(4) tint: vec4<f32>,\n    @location(5) model_1_: vec4<f32>,\n    @location(6) model_2_: vec4<f32>,\n    @location(7) model_3_: vec4<f32>,\n    @location(8) model_4_: vec4<f32>,\n}\n\nstruct VertexOutput {\n    @builtin(position) clip_position: vec4<f32>,\n    @location(0) tex_coords: vec2<f32>,\n}\n\nconst KERNEL_SIZE: u32 = 64u;\n\n@group(0) @binding(0) \nvar t_diffuse: texture_2d<f32>;\n@group(0) @binding(1) \nvar s_diffuse: sampler;\n@group(1) @binding(0) \nvar<uniform> global_uniforms: GlobalUniformsX_naga_oil_mod_XM5WG6YTBNQX;\n@group(2) @binding(0) \nvar<uniform> view_proj_uniforms: ViewProjectionUniformsX_naga_oil_mod_XM5WG6YTBNQX;\n@group(3) @binding(0) \nvar depth_buffer: texture_depth_2d;\n@group(3) @binding(1) \nvar depth_buffer_sampler: sampler;\n@group(4) @binding(0) \nvar<uniform> kernel: Kernel;\n@group(5) @binding(0) \nvar ssao_noise: texture_2d<f32>;\n@group(5) @binding(1) \nvar ssao_noise_sampler: sampler;\n\nfn reconstructPosition(coords: vec2<f32>) -> vec3<f32> {\n    let x = ((coords.x * 2f) - 1f);\n    let y = (((1f - coords.y) * 2f) - 1f);\n    let z = textureSample(depth_buffer, depth_buffer_sampler, coords);\n    let position_s = vec4<f32>(x, y, z, 1f);\n    let _e20 = kernel.inverse_proj;\n    let position_v = (_e20 * position_s);\n    return (position_v.xyz / vec3(position_v.w));\n}\n\nfn normalFromDepth(center: vec3<f32>, coords_1: vec2<f32>) -> vec3<f32> {\n    var y1_: vec3<f32>;\n    var y2_: vec3<f32>;\n    var x1_: vec3<f32>;\n    var x2_: vec3<f32>;\n\n    let _e6 = global_uniforms.screen_size;\n    let _e9 = reconstructPosition((coords_1 + (vec2<f32>(0f, 1f) / _e6)));\n    let _e15 = global_uniforms.screen_size;\n    let _e18 = reconstructPosition((coords_1 + (vec2<f32>(0f, -1f) / _e15)));\n    y1_ = _e9;\n    y2_ = center;\n    if (abs((_e18.z - center.z)) < abs((_e9.z - center.z))) {\n        y1_ = center;\n        y2_ = _e18;\n    }\n    let _e36 = global_uniforms.screen_size;\n    let _e39 = reconstructPosition((coords_1 + (vec2<f32>(-1f, 0f) / _e36)));\n    let _e45 = global_uniforms.screen_size;\n    let _e48 = reconstructPosition((coords_1 + (vec2<f32>(1f, 0f) / _e45)));\n    x1_ = _e39;\n    x2_ = center;\n    if (abs((_e48.z - center.z)) < abs((_e39.z - center.z))) {\n        x1_ = center;\n        x2_ = _e48;\n    }\n    let _e60 = x2_;\n    let _e61 = x1_;\n    let _e63 = y2_;\n    let _e64 = y1_;\n    return normalize(cross((_e60 - _e61), (_e63 - _e64)));\n}\n\n@vertex \nfn vs_main(vertex: VertexInput, instance: InstanceInput) -> VertexOutput {\n    var out: VertexOutput;\n    var model: vec4<f32>;\n\n    let model_transform = mat4x4<f32>(instance.model_1_, instance.model_2_, instance.model_3_, instance.model_4_);\n    out.tex_coords = (instance.uv_offset + (instance.uv_scale * vertex.tex_coords));\n    model = vertex.position;\n    let _e18 = model.x;\n    model.x = ((_e18 * 2f) - 1f);\n    let _e25 = model.y;\n    model.y = ((_e25 * 2f) - 1f);\n    let model_view = model;\n    out.clip_position = model_view;\n    let _e32 = out;\n    return _e32;\n}\n\n@fragment \nfn fs_main(in: VertexOutput) -> @location(0) f32 {\n    var view_pos: vec3<f32>;\n    var view_space_normal: vec3<f32>;\n    var random_vec: vec3<f32>;\n    var occlusion: f32 = 0f;\n    var i: i32 = 0i;\n    var sample: vec3<f32>;\n    var offset: vec4<f32>;\n    var sample_depth: f32;\n    var range_check: f32;\n\n    let _e4 = reconstructPosition(in.tex_coords);\n    view_pos = _e4;\n    let _e6 = view_pos;\n    let _e8 = normalFromDepth(_e6, in.tex_coords);\n    view_space_normal = _e8;\n    let _e14 = kernel.noise_texture_scale;\n    let _e18 = textureSample(ssao_noise, ssao_noise_sampler, (_e14 * in.tex_coords.xy));\n    random_vec = _e18.xyz;\n    let _e21 = random_vec;\n    let _e22 = view_space_normal;\n    let _e23 = random_vec;\n    let _e24 = view_space_normal;\n    let tangent = normalize((_e21 - (_e22 * dot(_e23, _e24))));\n    let _e29 = view_space_normal;\n    let bitangent = cross(_e29, tangent);\n    let _e31 = view_space_normal;\n    let TBN = mat3x3<f32>(tangent, bitangent, _e31);\n    loop {\n        let _e34 = i;\n        if (_e34 < 64i) {\n        } else {\n            break;\n        }\n        {\n            let _e37 = view_pos;\n            let _e41 = kernel.radius;\n            let _e45 = i;\n            let _e47 = kernel.items[_e45];\n            sample = (_e37.xyz + ((_e41 * TBN) * _e47.xyz));\n            let _e54 = view_proj_uniforms.projection;\n            let _e55 = sample;\n            offset = (_e54 * vec4<f32>(_e55, 1f));\n            let _e62 = offset.w;\n            let _e63 = offset.x;\n            offset.x = (_e63 / _e62);\n            let _e67 = offset.w;\n            let _e68 = offset.y;\n            offset.y = (_e68 / _e67);\n            let _e72 = offset.x;\n            offset.x = ((_e72 * 0.5f) + 0.5f);\n            let _e79 = offset.y;\n            offset.y = ((_e79 * 0.5f) + 0.5f);\n            let _e86 = offset.y;\n            offset.y = (1f - _e86);\n            let _e89 = offset;\n            let _e91 = reconstructPosition(_e89.xy);\n            sample_depth = _e91.z;\n            let _e98 = kernel.radius;\n            let _e100 = view_pos.z;\n            let _e101 = sample_depth;\n            range_check = smoothstep(0f, 1f, (_e98 / abs((_e100 - _e101))));\n            let _e107 = sample_depth;\n            let _e109 = sample.z;\n            let _e112 = kernel.bias;\n            if (_e107 >= (_e109 + _e112)) {\n                let _e116 = range_check;\n                let _e117 = range_check;\n                let _e119 = occlusion;\n                occlusion = (_e119 + (_e116 * _e117));\n            }\n        }\n        continuing {\n            let _e122 = i;\n            i = (_e122 + 1i);\n        }\n    }\n    let _e124 = occlusion;\n    occlusion = (1f - (_e124 / 64f));\n    let _e129 = occlusion;\n    occlusion = pow(_e129, 2f);\n    let _e132 = occlusion;\n    return _e132;\n}\n";
    ///Shader module descriptor.
    pub const DESCRIPTOR: ::wgpu::ShaderModuleDescriptor = ::wgpu::ShaderModuleDescriptor {
        label: Some("ssao_from_depth"),
        source: ::wgpu::ShaderSource::Wgsl(::std::borrow::Cow::Borrowed(SOURCE)),
    };
}
pub mod picking {
    #[allow(unused)]
    ///Information about the globals within the module, exposed as constants and functions.
    pub mod globals {
        #[allow(unused)]
        use super::*;
        ///Information about the `t_diffuse` global variable within this shader module.
        pub mod t_diffuse {
            #[allow(unused)]
            use super::*;
            pub const NAME: &'static str = "t_diffuse";
            pub const GROUP: u32 = 0u32;
            pub const BINDING: u32 = 0u32;
        }
        ///Information about the `s_diffuse` global variable within this shader module.
        pub mod s_diffuse {
            #[allow(unused)]
            use super::*;
            pub const NAME: &'static str = "s_diffuse";
            pub const GROUP: u32 = 0u32;
            pub const BINDING: u32 = 1u32;
        }
        ///Information about the `global_uniforms` global variable within this shader module.
        pub mod global_uniforms {
            #[allow(unused)]
            use super::*;
            pub const NAME: &'static str = "global_uniforms";
            pub const GROUP: u32 = 1u32;
            pub const BINDING: u32 = 0u32;
            pub fn layout() -> ::wgpu::BindGroupLayoutDescriptor<'static> {
                ::wgpu::BindGroupLayoutDescriptor {
                    label: Some("global_uniforms"),
                    entries: &[
                        ::wgpu::BindGroupLayoutEntry {
                            binding: 0u32,
                            visibility: ::wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: ::wgpu::BindingType::Buffer {
                                ty: ::wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                }
            }
        }
        ///Information about the `view_proj_uniforms` global variable within this shader module.
        pub mod view_proj_uniforms {
            #[allow(unused)]
            use super::*;
            pub const NAME: &'static str = "view_proj_uniforms";
            pub const GROUP: u32 = 2u32;
            pub const BINDING: u32 = 0u32;
            pub fn layout() -> ::wgpu::BindGroupLayoutDescriptor<'static> {
                ::wgpu::BindGroupLayoutDescriptor {
                    label: Some("view_proj_uniforms"),
                    entries: &[
                        ::wgpu::BindGroupLayoutEntry {
                            binding: 0u32,
                            visibility: ::wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: ::wgpu::BindingType::Buffer {
                                ty: ::wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                }
            }
        }
        ///Contains the following bindings: t_diffuse, s_diffuse
        pub mod group0 {
            #[allow(unused)]
            use super::*;
            pub const GROUP: u32 = 0u32;
            pub fn layout() -> ::wgpu::BindGroupLayoutDescriptor<'static> {
                ::wgpu::BindGroupLayoutDescriptor {
                    label: Some("group0"),
                    entries: &[
                        ::wgpu::BindGroupLayoutEntry {
                            binding: 0u32,
                            visibility: ::wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: ::wgpu::TextureViewDimension::D2,
                                sample_type: ::wgpu::TextureSampleType::Float {
                                    filterable: true,
                                },
                            },
                            count: None,
                        },
                        ::wgpu::BindGroupLayoutEntry {
                            binding: 1u32,
                            visibility: ::wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: ::wgpu::BindingType::Sampler(
                                ::wgpu::SamplerBindingType::Filtering,
                            ),
                            count: None,
                        },
                    ],
                }
            }
        }
    }
    #[allow(unused)]
    ///Information about the constants within the module, exposed as constants and functions.
    pub mod constants {
        #[allow(unused)]
        use super::*;
    }
    #[allow(unused)]
    ///Information about the entry points within the module, exposed as constants and functions.
    pub mod entry_points {
        #[allow(unused)]
        use super::*;
        pub mod vs_main {
            pub const NAME: &'static str = "vs_main";
            ///The sourcecode for the shader, as a constant string, excluding any other entry points. This is useful when the `minify` feature is enabled for this crate, as it allows more aggressive minification to be performed with the knowledge of the specific entry point that will be used.
            pub const EXCLUSIVE_SOURCE: &'static str = "struct GlobalUniforms {\n    time: f32,\n}\n\nstruct ViewProjectionUniforms {\n    view: mat4x4<f32>,\n    projection: mat4x4<f32>,\n    camera_pos: vec3<f32>,\n}\n\nstruct VertexInput {\n    @location(0) position: vec4<f32>,\n    @location(1) tex_coords: vec2<f32>,\n    @location(2) normal: vec3<f32>,\n}\n\nstruct InstanceInput {\n    @location(3) model_1_: vec4<f32>,\n    @location(4) model_2_: vec4<f32>,\n    @location(5) model_3_: vec4<f32>,\n    @location(6) model_4_: vec4<f32>,\n    @location(7) @interpolate(flat) id_color: vec2<u32>,\n}\n\nstruct VertexOutput {\n    @builtin(position) clip_position: vec4<f32>,\n    @location(1) world_pos: vec4<f32>,\n    @location(2) @interpolate(flat) id_color: vec2<u32>,\n}\n\n@group(0) @binding(0) \nvar t_diffuse: texture_2d<f32>;\n@group(0) @binding(1) \nvar s_diffuse: sampler;\n@group(1) @binding(0) \nvar<uniform> global_uniforms: GlobalUniforms;\n@group(2) @binding(0) \nvar<uniform> view_proj_uniforms: ViewProjectionUniforms;\n\n@vertex \nfn vs_main(vertex: VertexInput, instance: InstanceInput) -> VertexOutput {\n    var out: VertexOutput;\n\n    let model_transform = mat4x4<f32>(instance.model_1_, instance.model_2_, instance.model_3_, instance.model_4_);\n    let model = (model_transform * vertex.position);\n    let _e11 = view_proj_uniforms.view;\n    let model_view = (_e11 * model);\n    let _e17 = view_proj_uniforms.projection;\n    out.clip_position = (_e17 * model_view);\n    out.world_pos = model;\n    out.id_color = instance.id_color;\n    let _e22 = out;\n    return _e22;\n}\n";
        }
        pub mod fs_main {
            pub const NAME: &'static str = "fs_main";
            ///The sourcecode for the shader, as a constant string, excluding any other entry points. This is useful when the `minify` feature is enabled for this crate, as it allows more aggressive minification to be performed with the knowledge of the specific entry point that will be used.
            pub const EXCLUSIVE_SOURCE: &'static str = "struct GlobalUniforms {\n    time: f32,\n}\n\nstruct ViewProjectionUniforms {\n    view: mat4x4<f32>,\n    projection: mat4x4<f32>,\n    camera_pos: vec3<f32>,\n}\n\nstruct VertexInput {\n    @location(0) position: vec4<f32>,\n    @location(1) tex_coords: vec2<f32>,\n    @location(2) normal: vec3<f32>,\n}\n\nstruct InstanceInput {\n    @location(3) model_1_: vec4<f32>,\n    @location(4) model_2_: vec4<f32>,\n    @location(5) model_3_: vec4<f32>,\n    @location(6) model_4_: vec4<f32>,\n    @location(7) @interpolate(flat) id_color: vec2<u32>,\n}\n\nstruct VertexOutput {\n    @builtin(position) clip_position: vec4<f32>,\n    @location(1) world_pos: vec4<f32>,\n    @location(2) @interpolate(flat) id_color: vec2<u32>,\n}\n\n@group(0) @binding(0) \nvar t_diffuse: texture_2d<f32>;\n@group(0) @binding(1) \nvar s_diffuse: sampler;\n@group(1) @binding(0) \nvar<uniform> global_uniforms: GlobalUniforms;\n@group(2) @binding(0) \nvar<uniform> view_proj_uniforms: ViewProjectionUniforms;\n\n@fragment \nfn fs_main(in: VertexOutput) -> @location(0) @interpolate(flat) vec4<u32> {\n    return vec4<u32>((in.id_color.x & 255u), ((in.id_color.x >> 8u) & 255u), ((in.id_color.x >> 16u) & 255u), (in.id_color.x >> 24u));\n}\n";
        }
    }
    #[allow(unused)]
    ///Equivalent Rust definitions of the types defined in this module.
    pub mod types {}
    #[allow(unused)]
    use types::*;
    ///The sourcecode for the shader, as a constant string.
    pub const SOURCE: &'static str = "struct GlobalUniforms {\n    time: f32,\n}\n\nstruct ViewProjectionUniforms {\n    view: mat4x4<f32>,\n    projection: mat4x4<f32>,\n    camera_pos: vec3<f32>,\n}\n\nstruct VertexInput {\n    @location(0) position: vec4<f32>,\n    @location(1) tex_coords: vec2<f32>,\n    @location(2) normal: vec3<f32>,\n}\n\nstruct InstanceInput {\n    @location(3) model_1_: vec4<f32>,\n    @location(4) model_2_: vec4<f32>,\n    @location(5) model_3_: vec4<f32>,\n    @location(6) model_4_: vec4<f32>,\n    @location(7) @interpolate(flat) id_color: vec2<u32>,\n}\n\nstruct VertexOutput {\n    @builtin(position) clip_position: vec4<f32>,\n    @location(1) world_pos: vec4<f32>,\n    @location(2) @interpolate(flat) id_color: vec2<u32>,\n}\n\n@group(0) @binding(0) \nvar t_diffuse: texture_2d<f32>;\n@group(0) @binding(1) \nvar s_diffuse: sampler;\n@group(1) @binding(0) \nvar<uniform> global_uniforms: GlobalUniforms;\n@group(2) @binding(0) \nvar<uniform> view_proj_uniforms: ViewProjectionUniforms;\n\n@vertex \nfn vs_main(vertex: VertexInput, instance: InstanceInput) -> VertexOutput {\n    var out: VertexOutput;\n\n    let model_transform = mat4x4<f32>(instance.model_1_, instance.model_2_, instance.model_3_, instance.model_4_);\n    let model = (model_transform * vertex.position);\n    let _e11 = view_proj_uniforms.view;\n    let model_view = (_e11 * model);\n    let _e17 = view_proj_uniforms.projection;\n    out.clip_position = (_e17 * model_view);\n    out.world_pos = model;\n    out.id_color = instance.id_color;\n    let _e22 = out;\n    return _e22;\n}\n\n@fragment \nfn fs_main(in: VertexOutput) -> @location(0) @interpolate(flat) vec4<u32> {\n    return vec4<u32>((in.id_color.x & 255u), ((in.id_color.x >> 8u) & 255u), ((in.id_color.x >> 16u) & 255u), (in.id_color.x >> 24u));\n}\n";
    ///Shader module descriptor.
    pub const DESCRIPTOR: ::wgpu::ShaderModuleDescriptor = ::wgpu::ShaderModuleDescriptor {
        label: Some("picking"),
        source: ::wgpu::ShaderSource::Wgsl(::std::borrow::Cow::Borrowed(SOURCE)),
    };
}
pub mod deferred_lighting {
    #[allow(unused)]
    ///Information about the globals within the module, exposed as constants and functions.
    pub mod globals {
        #[allow(unused)]
        use super::*;
        ///Information about the `t_diffuse` global variable within this shader module.
        pub mod t_diffuse {
            #[allow(unused)]
            use super::*;
            pub const NAME: &'static str = "t_diffuse";
            pub const GROUP: u32 = 0u32;
            pub const BINDING: u32 = 0u32;
        }
        ///Information about the `s_diffuse` global variable within this shader module.
        pub mod s_diffuse {
            #[allow(unused)]
            use super::*;
            pub const NAME: &'static str = "s_diffuse";
            pub const GROUP: u32 = 0u32;
            pub const BINDING: u32 = 1u32;
        }
        ///Information about the `global_uniforms` global variable within this shader module.
        pub mod global_uniforms {
            #[allow(unused)]
            use super::*;
            pub const NAME: &'static str = "global_uniforms";
            pub const GROUP: u32 = 1u32;
            pub const BINDING: u32 = 0u32;
            pub fn layout() -> ::wgpu::BindGroupLayoutDescriptor<'static> {
                ::wgpu::BindGroupLayoutDescriptor {
                    label: Some("global_uniforms"),
                    entries: &[
                        ::wgpu::BindGroupLayoutEntry {
                            binding: 0u32,
                            visibility: ::wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: ::wgpu::BindingType::Buffer {
                                ty: ::wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                }
            }
        }
        ///Information about the `view_proj_uniforms` global variable within this shader module.
        pub mod view_proj_uniforms {
            #[allow(unused)]
            use super::*;
            pub const NAME: &'static str = "view_proj_uniforms";
            pub const GROUP: u32 = 2u32;
            pub const BINDING: u32 = 0u32;
            pub fn layout() -> ::wgpu::BindGroupLayoutDescriptor<'static> {
                ::wgpu::BindGroupLayoutDescriptor {
                    label: Some("view_proj_uniforms"),
                    entries: &[
                        ::wgpu::BindGroupLayoutEntry {
                            binding: 0u32,
                            visibility: ::wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: ::wgpu::BindingType::Buffer {
                                ty: ::wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                }
            }
        }
        ///Information about the `g_position` global variable within this shader module.
        pub mod g_position {
            #[allow(unused)]
            use super::*;
            pub const NAME: &'static str = "g_position";
            pub const GROUP: u32 = 3u32;
            pub const BINDING: u32 = 0u32;
        }
        ///Information about the `g_position_sampler` global variable within this shader module.
        pub mod g_position_sampler {
            #[allow(unused)]
            use super::*;
            pub const NAME: &'static str = "g_position_sampler";
            pub const GROUP: u32 = 3u32;
            pub const BINDING: u32 = 1u32;
        }
        ///Information about the `g_normal` global variable within this shader module.
        pub mod g_normal {
            #[allow(unused)]
            use super::*;
            pub const NAME: &'static str = "g_normal";
            pub const GROUP: u32 = 3u32;
            pub const BINDING: u32 = 2u32;
        }
        ///Information about the `g_normal_sampler` global variable within this shader module.
        pub mod g_normal_sampler {
            #[allow(unused)]
            use super::*;
            pub const NAME: &'static str = "g_normal_sampler";
            pub const GROUP: u32 = 3u32;
            pub const BINDING: u32 = 3u32;
        }
        ///Information about the `g_albedo_spec` global variable within this shader module.
        pub mod g_albedo_spec {
            #[allow(unused)]
            use super::*;
            pub const NAME: &'static str = "g_albedo_spec";
            pub const GROUP: u32 = 3u32;
            pub const BINDING: u32 = 4u32;
        }
        ///Information about the `g_albedo_spec_sampler` global variable within this shader module.
        pub mod g_albedo_spec_sampler {
            #[allow(unused)]
            use super::*;
            pub const NAME: &'static str = "g_albedo_spec_sampler";
            pub const GROUP: u32 = 3u32;
            pub const BINDING: u32 = 5u32;
        }
        ///Information about the `lights` global variable within this shader module.
        pub mod lights {
            #[allow(unused)]
            use super::*;
            pub const NAME: &'static str = "lights";
            pub type Ty = LightsUniform;
            pub const GROUP: u32 = 4u32;
            pub const BINDING: u32 = 0u32;
        }
        ///Information about the `shadow_map` global variable within this shader module.
        pub mod shadow_map {
            #[allow(unused)]
            use super::*;
            pub const NAME: &'static str = "shadow_map";
            pub const GROUP: u32 = 4u32;
            pub const BINDING: u32 = 1u32;
        }
        ///Information about the `shadow_map_sampler` global variable within this shader module.
        pub mod shadow_map_sampler {
            #[allow(unused)]
            use super::*;
            pub const NAME: &'static str = "shadow_map_sampler";
            pub const GROUP: u32 = 4u32;
            pub const BINDING: u32 = 2u32;
        }
        ///Contains the following bindings: t_diffuse, s_diffuse
        pub mod group0 {
            #[allow(unused)]
            use super::*;
            pub const GROUP: u32 = 0u32;
            pub fn layout() -> ::wgpu::BindGroupLayoutDescriptor<'static> {
                ::wgpu::BindGroupLayoutDescriptor {
                    label: Some("group0"),
                    entries: &[
                        ::wgpu::BindGroupLayoutEntry {
                            binding: 0u32,
                            visibility: ::wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: ::wgpu::TextureViewDimension::D2,
                                sample_type: ::wgpu::TextureSampleType::Float {
                                    filterable: true,
                                },
                            },
                            count: None,
                        },
                        ::wgpu::BindGroupLayoutEntry {
                            binding: 1u32,
                            visibility: ::wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: ::wgpu::BindingType::Sampler(
                                ::wgpu::SamplerBindingType::Filtering,
                            ),
                            count: None,
                        },
                    ],
                }
            }
        }
        ///Contains the following bindings: lights, shadow_map, shadow_map_sampler
        pub mod group4 {
            #[allow(unused)]
            use super::*;
            pub const GROUP: u32 = 4u32;
            pub fn layout() -> ::wgpu::BindGroupLayoutDescriptor<'static> {
                ::wgpu::BindGroupLayoutDescriptor {
                    label: Some("group4"),
                    entries: &[
                        ::wgpu::BindGroupLayoutEntry {
                            binding: 0u32,
                            visibility: ::wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: ::wgpu::BindingType::Buffer {
                                ty: ::wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        ::wgpu::BindGroupLayoutEntry {
                            binding: 1u32,
                            visibility: ::wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: ::wgpu::TextureViewDimension::D2Array,
                                sample_type: ::wgpu::TextureSampleType::Depth,
                            },
                            count: None,
                        },
                        ::wgpu::BindGroupLayoutEntry {
                            binding: 2u32,
                            visibility: ::wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: ::wgpu::BindingType::Sampler(
                                ::wgpu::SamplerBindingType::Comparison,
                            ),
                            count: None,
                        },
                    ],
                }
            }
        }
        ///Contains the following bindings: g_position, g_position_sampler, g_normal, g_normal_sampler, g_albedo_spec, g_albedo_spec_sampler
        pub mod group3 {
            #[allow(unused)]
            use super::*;
            pub const GROUP: u32 = 3u32;
            pub fn layout() -> ::wgpu::BindGroupLayoutDescriptor<'static> {
                ::wgpu::BindGroupLayoutDescriptor {
                    label: Some("group3"),
                    entries: &[
                        ::wgpu::BindGroupLayoutEntry {
                            binding: 0u32,
                            visibility: ::wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: ::wgpu::TextureViewDimension::D2,
                                sample_type: ::wgpu::TextureSampleType::Float {
                                    filterable: true,
                                },
                            },
                            count: None,
                        },
                        ::wgpu::BindGroupLayoutEntry {
                            binding: 1u32,
                            visibility: ::wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: ::wgpu::BindingType::Sampler(
                                ::wgpu::SamplerBindingType::Filtering,
                            ),
                            count: None,
                        },
                        ::wgpu::BindGroupLayoutEntry {
                            binding: 2u32,
                            visibility: ::wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: ::wgpu::TextureViewDimension::D2,
                                sample_type: ::wgpu::TextureSampleType::Float {
                                    filterable: true,
                                },
                            },
                            count: None,
                        },
                        ::wgpu::BindGroupLayoutEntry {
                            binding: 3u32,
                            visibility: ::wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: ::wgpu::BindingType::Sampler(
                                ::wgpu::SamplerBindingType::Filtering,
                            ),
                            count: None,
                        },
                        ::wgpu::BindGroupLayoutEntry {
                            binding: 4u32,
                            visibility: ::wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: ::wgpu::TextureViewDimension::D2,
                                sample_type: ::wgpu::TextureSampleType::Float {
                                    filterable: true,
                                },
                            },
                            count: None,
                        },
                        ::wgpu::BindGroupLayoutEntry {
                            binding: 5u32,
                            visibility: ::wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: ::wgpu::BindingType::Sampler(
                                ::wgpu::SamplerBindingType::Filtering,
                            ),
                            count: None,
                        },
                    ],
                }
            }
        }
    }
    #[allow(unused)]
    ///Information about the constants within the module, exposed as constants and functions.
    pub mod constants {
        #[allow(unused)]
        use super::*;
        #[allow(non_snake_case)]
        ///Information about the `AMBIENT_LIGHT_FACTOR` constant variable within this shader module.
        pub mod AMBIENT_LIGHT_FACTOR {
            #[allow(unused)]
            use super::*;
            pub const NAME: &'static str = "AMBIENT_LIGHT_FACTOR";
        }
    }
    #[allow(unused)]
    ///Information about the entry points within the module, exposed as constants and functions.
    pub mod entry_points {
        #[allow(unused)]
        use super::*;
        pub mod vs_main {
            pub const NAME: &'static str = "vs_main";
            ///The sourcecode for the shader, as a constant string, excluding any other entry points. This is useful when the `minify` feature is enabled for this crate, as it allows more aggressive minification to be performed with the knowledge of the specific entry point that will be used.
            pub const EXCLUSIVE_SOURCE: &'static str = "struct GlobalUniforms {\n    time: f32,\n}\n\nstruct ViewProjectionUniforms {\n    view: mat4x4<f32>,\n    projection: mat4x4<f32>,\n    camera_pos: vec3<f32>,\n    inverse_view: mat4x4<f32>,\n}\n\nstruct VertexInput {\n    @location(0) position: vec4<f32>,\n    @location(1) tex_coords: vec2<f32>,\n}\n\nstruct InstanceInput {\n    @location(2) uv_scale: vec2<f32>,\n    @location(3) uv_offset: vec2<f32>,\n    @location(4) tint: vec4<f32>,\n    @location(5) model_1_: vec4<f32>,\n    @location(6) model_2_: vec4<f32>,\n    @location(7) model_3_: vec4<f32>,\n    @location(8) model_4_: vec4<f32>,\n}\n\nstruct VertexOutput {\n    @builtin(position) clip_position: vec4<f32>,\n    @location(0) tex_coords: vec2<f32>,\n    @location(2) tint_color: vec4<f32>,\n}\n\nstruct Light {\n    position: vec4<f32>,\n    color: vec4<f32>,\n    view_proj: mat4x4<f32>,\n}\n\nstruct LightsUniform {\n    items: array<Light, 8>,\n    count: u32,\n}\n\nconst AMBIENT_LIGHT_FACTOR: vec3<f32> = vec3<f32>(0.5f, 0.5f, 0.5f);\n\n@group(0) @binding(0) \nvar t_diffuse: texture_2d<f32>;\n@group(0) @binding(1) \nvar s_diffuse: sampler;\n@group(1) @binding(0) \nvar<uniform> global_uniforms: GlobalUniforms;\n@group(2) @binding(0) \nvar<uniform> view_proj_uniforms: ViewProjectionUniforms;\n@group(3) @binding(0) \nvar g_position: texture_2d<f32>;\n@group(3) @binding(1) \nvar g_position_sampler: sampler;\n@group(3) @binding(2) \nvar g_normal: texture_2d<f32>;\n@group(3) @binding(3) \nvar g_normal_sampler: sampler;\n@group(3) @binding(4) \nvar g_albedo_spec: texture_2d<f32>;\n@group(3) @binding(5) \nvar g_albedo_spec_sampler: sampler;\n@group(4) @binding(0) \nvar<uniform> lights: LightsUniform;\n@group(4) @binding(1) \nvar shadow_map: texture_depth_2d_array;\n@group(4) @binding(2) \nvar shadow_map_sampler: sampler_comparison;\n\n@vertex \nfn vs_main(vertex: VertexInput, instance: InstanceInput) -> VertexOutput {\n    var out: VertexOutput;\n    var model: vec4<f32>;\n\n    let model_transform = mat4x4<f32>(instance.model_1_, instance.model_2_, instance.model_3_, instance.model_4_);\n    out.tex_coords = (instance.uv_offset + (instance.uv_scale * vertex.tex_coords));\n    model = vertex.position;\n    let _e18 = model.x;\n    model.x = ((_e18 * 2f) - 1f);\n    let _e25 = model.y;\n    model.y = ((_e25 * 2f) - 1f);\n    let model_view = model;\n    out.clip_position = model_view;\n    out.tint_color = instance.tint;\n    let _e34 = out;\n    return _e34;\n}\n";
        }
        pub mod fs_main {
            pub const NAME: &'static str = "fs_main";
            ///The sourcecode for the shader, as a constant string, excluding any other entry points. This is useful when the `minify` feature is enabled for this crate, as it allows more aggressive minification to be performed with the knowledge of the specific entry point that will be used.
            pub const EXCLUSIVE_SOURCE: &'static str = "struct GlobalUniforms {\n    time: f32,\n}\n\nstruct ViewProjectionUniforms {\n    view: mat4x4<f32>,\n    projection: mat4x4<f32>,\n    camera_pos: vec3<f32>,\n    inverse_view: mat4x4<f32>,\n}\n\nstruct VertexInput {\n    @location(0) position: vec4<f32>,\n    @location(1) tex_coords: vec2<f32>,\n}\n\nstruct InstanceInput {\n    @location(2) uv_scale: vec2<f32>,\n    @location(3) uv_offset: vec2<f32>,\n    @location(4) tint: vec4<f32>,\n    @location(5) model_1_: vec4<f32>,\n    @location(6) model_2_: vec4<f32>,\n    @location(7) model_3_: vec4<f32>,\n    @location(8) model_4_: vec4<f32>,\n}\n\nstruct VertexOutput {\n    @builtin(position) clip_position: vec4<f32>,\n    @location(0) tex_coords: vec2<f32>,\n    @location(2) tint_color: vec4<f32>,\n}\n\nstruct Light {\n    position: vec4<f32>,\n    color: vec4<f32>,\n    view_proj: mat4x4<f32>,\n}\n\nstruct LightsUniform {\n    items: array<Light, 8>,\n    count: u32,\n}\n\nconst AMBIENT_LIGHT_FACTOR: vec3<f32> = vec3<f32>(0.5f, 0.5f, 0.5f);\n\n@group(0) @binding(0) \nvar t_diffuse: texture_2d<f32>;\n@group(0) @binding(1) \nvar s_diffuse: sampler;\n@group(1) @binding(0) \nvar<uniform> global_uniforms: GlobalUniforms;\n@group(2) @binding(0) \nvar<uniform> view_proj_uniforms: ViewProjectionUniforms;\n@group(3) @binding(0) \nvar g_position: texture_2d<f32>;\n@group(3) @binding(1) \nvar g_position_sampler: sampler;\n@group(3) @binding(2) \nvar g_normal: texture_2d<f32>;\n@group(3) @binding(3) \nvar g_normal_sampler: sampler;\n@group(3) @binding(4) \nvar g_albedo_spec: texture_2d<f32>;\n@group(3) @binding(5) \nvar g_albedo_spec_sampler: sampler;\n@group(4) @binding(0) \nvar<uniform> lights: LightsUniform;\n@group(4) @binding(1) \nvar shadow_map: texture_depth_2d_array;\n@group(4) @binding(2) \nvar shadow_map_sampler: sampler_comparison;\n\n@fragment \nfn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {\n    var specular: vec4<f32> = vec4<f32>(0f, 0f, 0f, 0f);\n    var diffuse: vec4<f32> = vec4<f32>(0f, 0f, 0f, 0f);\n    var total_light: vec4<f32>;\n    var i: u32 = 0u;\n    var shadow: f32;\n    var specular_factor: f32;\n\n    let view_pos = textureSample(g_position, g_position_sampler, in.tex_coords);\n    let view_space_normal = textureSample(g_normal, g_normal_sampler, in.tex_coords);\n    let _e16 = textureSample(g_albedo_spec, g_albedo_spec_sampler, in.tex_coords);\n    let albedo_spec = (in.tint_color * _e16);\n    let _e21 = textureSample(t_diffuse, s_diffuse, in.tex_coords);\n    total_light = vec4(clamp(_e21.x, 0f, 1f));\n    loop {\n        let _e29 = i;\n        let _e32 = lights.count;\n        if (_e29 < _e32) {\n        } else {\n            break;\n        }\n        {\n            let _e36 = i;\n            let light_color = lights.items[_e36].color;\n            let _e42 = i;\n            let light_pos_w = lights.items[_e42].position;\n            let view_dir_v = normalize(view_pos);\n            let _e49 = view_proj_uniforms.view;\n            let light_pos_v = (_e49 * light_pos_w);\n            let light_dir_v = normalize((light_pos_v - view_pos).xyz);\n            let _e56 = i;\n            let _e59 = lights.items[_e56].view_proj;\n            let _e62 = view_proj_uniforms.inverse_view;\n            let shadow_pos = ((_e59 * _e62) * view_pos);\n            shadow = 1f;\n            let half_dir_v = normalize((view_dir_v.xyz + light_dir_v));\n            let reflect_dir_v = reflect(light_dir_v, view_space_normal.xyz);\n            specular_factor = clamp(dot(view_space_normal.xyz, half_dir_v), 0f, 1f);\n            let _e78 = specular_factor;\n            specular_factor = pow(_e78, 32f);\n            let _e81 = specular_factor;\n            if (_e81 > 0f) {\n                let _e85 = shadow;\n                let _e87 = specular_factor;\n                let _e90 = total_light;\n                total_light = (_e90 + (((_e85 * 0.6f) * _e87) * light_color));\n            }\n            let d = clamp(dot(view_space_normal.xyz, light_dir_v), 0f, 1f);\n            let _e97 = shadow;\n            let _e100 = total_light;\n            total_light = (_e100 + ((_e97 * d) * light_color));\n        }\n        continuing {\n            let _e103 = i;\n            i = (_e103 + 1u);\n        }\n    }\n    let _e106 = total_light;\n    return vec4<f32>((albedo_spec.xyz * _e106.xyz), albedo_spec.w);\n}\n";
        }
    }
    #[allow(unused)]
    ///Equivalent Rust definitions of the types defined in this module.
    pub mod types {
        #[allow(unused, non_camel_case_types)]
        #[repr(C)]
        #[derive(Debug, PartialEq, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
        pub struct Light {
            pub position: glam::f32::Vec4,
            pub color: glam::f32::Vec4,
            pub view_proj: glam::f32::Mat4,
        }
        #[allow(unused, non_camel_case_types)]
        #[repr(C)]
        #[derive(Debug, PartialEq, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
        pub struct LightsUniform {
            pub items: [Light; 8u32 as usize],
            pub count: u32,
            pub _pad: [u8; 12u32 as usize],
        }
    }
    #[allow(unused)]
    use types::*;
    ///The sourcecode for the shader, as a constant string.
    pub const SOURCE: &'static str = "struct GlobalUniforms {\n    time: f32,\n}\n\nstruct ViewProjectionUniforms {\n    view: mat4x4<f32>,\n    projection: mat4x4<f32>,\n    camera_pos: vec3<f32>,\n    inverse_view: mat4x4<f32>,\n}\n\nstruct VertexInput {\n    @location(0) position: vec4<f32>,\n    @location(1) tex_coords: vec2<f32>,\n}\n\nstruct InstanceInput {\n    @location(2) uv_scale: vec2<f32>,\n    @location(3) uv_offset: vec2<f32>,\n    @location(4) tint: vec4<f32>,\n    @location(5) model_1_: vec4<f32>,\n    @location(6) model_2_: vec4<f32>,\n    @location(7) model_3_: vec4<f32>,\n    @location(8) model_4_: vec4<f32>,\n}\n\nstruct VertexOutput {\n    @builtin(position) clip_position: vec4<f32>,\n    @location(0) tex_coords: vec2<f32>,\n    @location(2) tint_color: vec4<f32>,\n}\n\nstruct Light {\n    position: vec4<f32>,\n    color: vec4<f32>,\n    view_proj: mat4x4<f32>,\n}\n\nstruct LightsUniform {\n    items: array<Light, 8>,\n    count: u32,\n}\n\nconst AMBIENT_LIGHT_FACTOR: vec3<f32> = vec3<f32>(0.5f, 0.5f, 0.5f);\n\n@group(0) @binding(0) \nvar t_diffuse: texture_2d<f32>;\n@group(0) @binding(1) \nvar s_diffuse: sampler;\n@group(1) @binding(0) \nvar<uniform> global_uniforms: GlobalUniforms;\n@group(2) @binding(0) \nvar<uniform> view_proj_uniforms: ViewProjectionUniforms;\n@group(3) @binding(0) \nvar g_position: texture_2d<f32>;\n@group(3) @binding(1) \nvar g_position_sampler: sampler;\n@group(3) @binding(2) \nvar g_normal: texture_2d<f32>;\n@group(3) @binding(3) \nvar g_normal_sampler: sampler;\n@group(3) @binding(4) \nvar g_albedo_spec: texture_2d<f32>;\n@group(3) @binding(5) \nvar g_albedo_spec_sampler: sampler;\n@group(4) @binding(0) \nvar<uniform> lights: LightsUniform;\n@group(4) @binding(1) \nvar shadow_map: texture_depth_2d_array;\n@group(4) @binding(2) \nvar shadow_map_sampler: sampler_comparison;\n\n@vertex \nfn vs_main(vertex: VertexInput, instance: InstanceInput) -> VertexOutput {\n    var out: VertexOutput;\n    var model: vec4<f32>;\n\n    let model_transform = mat4x4<f32>(instance.model_1_, instance.model_2_, instance.model_3_, instance.model_4_);\n    out.tex_coords = (instance.uv_offset + (instance.uv_scale * vertex.tex_coords));\n    model = vertex.position;\n    let _e18 = model.x;\n    model.x = ((_e18 * 2f) - 1f);\n    let _e25 = model.y;\n    model.y = ((_e25 * 2f) - 1f);\n    let model_view = model;\n    out.clip_position = model_view;\n    out.tint_color = instance.tint;\n    let _e34 = out;\n    return _e34;\n}\n\n@fragment \nfn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {\n    var specular: vec4<f32> = vec4<f32>(0f, 0f, 0f, 0f);\n    var diffuse: vec4<f32> = vec4<f32>(0f, 0f, 0f, 0f);\n    var total_light: vec4<f32>;\n    var i: u32 = 0u;\n    var shadow: f32;\n    var specular_factor: f32;\n\n    let view_pos = textureSample(g_position, g_position_sampler, in.tex_coords);\n    let view_space_normal = textureSample(g_normal, g_normal_sampler, in.tex_coords);\n    let _e16 = textureSample(g_albedo_spec, g_albedo_spec_sampler, in.tex_coords);\n    let albedo_spec = (in.tint_color * _e16);\n    let _e21 = textureSample(t_diffuse, s_diffuse, in.tex_coords);\n    total_light = vec4(clamp(_e21.x, 0f, 1f));\n    loop {\n        let _e29 = i;\n        let _e32 = lights.count;\n        if (_e29 < _e32) {\n        } else {\n            break;\n        }\n        {\n            let _e36 = i;\n            let light_color = lights.items[_e36].color;\n            let _e42 = i;\n            let light_pos_w = lights.items[_e42].position;\n            let view_dir_v = normalize(view_pos);\n            let _e49 = view_proj_uniforms.view;\n            let light_pos_v = (_e49 * light_pos_w);\n            let light_dir_v = normalize((light_pos_v - view_pos).xyz);\n            let _e56 = i;\n            let _e59 = lights.items[_e56].view_proj;\n            let _e62 = view_proj_uniforms.inverse_view;\n            let shadow_pos = ((_e59 * _e62) * view_pos);\n            shadow = 1f;\n            let half_dir_v = normalize((view_dir_v.xyz + light_dir_v));\n            let reflect_dir_v = reflect(light_dir_v, view_space_normal.xyz);\n            specular_factor = clamp(dot(view_space_normal.xyz, half_dir_v), 0f, 1f);\n            let _e78 = specular_factor;\n            specular_factor = pow(_e78, 32f);\n            let _e81 = specular_factor;\n            if (_e81 > 0f) {\n                let _e85 = shadow;\n                let _e87 = specular_factor;\n                let _e90 = total_light;\n                total_light = (_e90 + (((_e85 * 0.6f) * _e87) * light_color));\n            }\n            let d = clamp(dot(view_space_normal.xyz, light_dir_v), 0f, 1f);\n            let _e97 = shadow;\n            let _e100 = total_light;\n            total_light = (_e100 + ((_e97 * d) * light_color));\n        }\n        continuing {\n            let _e103 = i;\n            i = (_e103 + 1u);\n        }\n    }\n    let _e106 = total_light;\n    return vec4<f32>((albedo_spec.xyz * _e106.xyz), albedo_spec.w);\n}\n";
    ///Shader module descriptor.
    pub const DESCRIPTOR: ::wgpu::ShaderModuleDescriptor = ::wgpu::ShaderModuleDescriptor {
        label: Some("deferred_lighting"),
        source: ::wgpu::ShaderSource::Wgsl(::std::borrow::Cow::Borrowed(SOURCE)),
    };
}
pub mod depth_only {
    #[allow(unused)]
    ///Information about the globals within the module, exposed as constants and functions.
    pub mod globals {
        #[allow(unused)]
        use super::*;
        ///Information about the `t_diffuse` global variable within this shader module.
        pub mod t_diffuse {
            #[allow(unused)]
            use super::*;
            pub const NAME: &'static str = "t_diffuse";
            pub const GROUP: u32 = 0u32;
            pub const BINDING: u32 = 0u32;
        }
        ///Information about the `s_diffuse` global variable within this shader module.
        pub mod s_diffuse {
            #[allow(unused)]
            use super::*;
            pub const NAME: &'static str = "s_diffuse";
            pub const GROUP: u32 = 0u32;
            pub const BINDING: u32 = 1u32;
        }
        ///Information about the `global_uniforms` global variable within this shader module.
        pub mod global_uniforms {
            #[allow(unused)]
            use super::*;
            pub const NAME: &'static str = "global_uniforms";
            pub type Ty = super::super::super::global::types::GlobalUniforms;
            pub const GROUP: u32 = 1u32;
            pub const BINDING: u32 = 0u32;
            pub fn layout() -> ::wgpu::BindGroupLayoutDescriptor<'static> {
                ::wgpu::BindGroupLayoutDescriptor {
                    label: Some("global_uniforms"),
                    entries: &[
                        ::wgpu::BindGroupLayoutEntry {
                            binding: 0u32,
                            visibility: ::wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: ::wgpu::BindingType::Buffer {
                                ty: ::wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                }
            }
        }
        ///Information about the `view_proj_uniforms` global variable within this shader module.
        pub mod view_proj_uniforms {
            #[allow(unused)]
            use super::*;
            pub const NAME: &'static str = "view_proj_uniforms";
            pub type Ty = super::super::super::global::types::ViewProjectionUniforms;
            pub const GROUP: u32 = 2u32;
            pub const BINDING: u32 = 0u32;
            pub fn layout() -> ::wgpu::BindGroupLayoutDescriptor<'static> {
                ::wgpu::BindGroupLayoutDescriptor {
                    label: Some("view_proj_uniforms"),
                    entries: &[
                        ::wgpu::BindGroupLayoutEntry {
                            binding: 0u32,
                            visibility: ::wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: ::wgpu::BindingType::Buffer {
                                ty: ::wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                }
            }
        }
        ///Contains the following bindings: t_diffuse, s_diffuse
        pub mod group0 {
            #[allow(unused)]
            use super::*;
            pub const GROUP: u32 = 0u32;
            pub fn layout() -> ::wgpu::BindGroupLayoutDescriptor<'static> {
                ::wgpu::BindGroupLayoutDescriptor {
                    label: Some("group0"),
                    entries: &[
                        ::wgpu::BindGroupLayoutEntry {
                            binding: 0u32,
                            visibility: ::wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: ::wgpu::TextureViewDimension::D2,
                                sample_type: ::wgpu::TextureSampleType::Float {
                                    filterable: true,
                                },
                            },
                            count: None,
                        },
                        ::wgpu::BindGroupLayoutEntry {
                            binding: 1u32,
                            visibility: ::wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: ::wgpu::BindingType::Sampler(
                                ::wgpu::SamplerBindingType::Filtering,
                            ),
                            count: None,
                        },
                    ],
                }
            }
        }
    }
    #[allow(unused)]
    ///Information about the constants within the module, exposed as constants and functions.
    pub mod constants {
        #[allow(unused)]
        use super::*;
    }
    #[allow(unused)]
    ///Information about the entry points within the module, exposed as constants and functions.
    pub mod entry_points {
        #[allow(unused)]
        use super::*;
        pub mod vs_main {
            pub const NAME: &'static str = "vs_main";
            ///The sourcecode for the shader, as a constant string, excluding any other entry points. This is useful when the `minify` feature is enabled for this crate, as it allows more aggressive minification to be performed with the knowledge of the specific entry point that will be used.
            pub const EXCLUSIVE_SOURCE: &'static str = "struct GlobalUniformsX_naga_oil_mod_XM5WG6YTBNQX {\n    time: f32,\n    screen_size: vec2<f32>,\n}\n\nstruct ViewProjectionUniformsX_naga_oil_mod_XM5WG6YTBNQX {\n    view: mat4x4<f32>,\n    projection: mat4x4<f32>,\n    camera_pos: vec3<f32>,\n    inverse_view: mat4x4<f32>,\n}\n\nstruct VertexInputX_naga_oil_mod_XNFXHA5LUOMX {\n    @location(0) position: vec4<f32>,\n    @location(1) tex_coords: vec2<f32>,\n    @location(2) normal: vec3<f32>,\n}\n\nstruct InstanceInputX_naga_oil_mod_XNFXHA5LUOMX {\n    @location(3) uv_scale: vec2<f32>,\n    @location(4) uv_offset: vec2<f32>,\n    @location(5) tint: vec4<f32>,\n    @location(6) model_1x: vec4<f32>,\n    @location(7) model_2x: vec4<f32>,\n    @location(8) model_3x: vec4<f32>,\n    @location(9) model_4x: vec4<f32>,\n    @location(10) normal_1x: vec4<f32>,\n    @location(11) normal_2x: vec4<f32>,\n    @location(12) normal_3x: vec4<f32>,\n    @location(13) normal_4x: vec4<f32>,\n}\n\nstruct VertexOutput {\n    @builtin(position) clip_position: vec4<f32>,\n}\n\n@group(0) @binding(0) \nvar t_diffuse: texture_2d<f32>;\n@group(0) @binding(1) \nvar s_diffuse: sampler;\n@group(1) @binding(0) \nvar<uniform> global_uniforms: GlobalUniformsX_naga_oil_mod_XM5WG6YTBNQX;\n@group(2) @binding(0) \nvar<uniform> view_proj_uniforms: ViewProjectionUniformsX_naga_oil_mod_XM5WG6YTBNQX;\n\n@vertex \nfn vs_main(vertex: VertexInputX_naga_oil_mod_XNFXHA5LUOMX, instance: InstanceInputX_naga_oil_mod_XNFXHA5LUOMX) -> VertexOutput {\n    var out: VertexOutput;\n\n    let model_transform = mat4x4<f32>(instance.model_1x, instance.model_2x, instance.model_3x, instance.model_4x);\n    let normal_matrix = mat4x4<f32>(instance.normal_1x, instance.normal_2x, instance.normal_3x, instance.normal_4x);\n    let _e13 = view_proj_uniforms.view;\n    let model_view = (_e13 * model_transform);\n    let model_view_pos = (model_view * vertex.position);\n    let _e22 = view_proj_uniforms.projection;\n    out.clip_position = (_e22 * model_view_pos);\n    let _e24 = out;\n    return _e24;\n}\n";
        }
        pub mod fs_main {
            pub const NAME: &'static str = "fs_main";
            ///The sourcecode for the shader, as a constant string, excluding any other entry points. This is useful when the `minify` feature is enabled for this crate, as it allows more aggressive minification to be performed with the knowledge of the specific entry point that will be used.
            pub const EXCLUSIVE_SOURCE: &'static str = "struct GlobalUniformsX_naga_oil_mod_XM5WG6YTBNQX {\n    time: f32,\n    screen_size: vec2<f32>,\n}\n\nstruct ViewProjectionUniformsX_naga_oil_mod_XM5WG6YTBNQX {\n    view: mat4x4<f32>,\n    projection: mat4x4<f32>,\n    camera_pos: vec3<f32>,\n    inverse_view: mat4x4<f32>,\n}\n\nstruct VertexInputX_naga_oil_mod_XNFXHA5LUOMX {\n    @location(0) position: vec4<f32>,\n    @location(1) tex_coords: vec2<f32>,\n    @location(2) normal: vec3<f32>,\n}\n\nstruct InstanceInputX_naga_oil_mod_XNFXHA5LUOMX {\n    @location(3) uv_scale: vec2<f32>,\n    @location(4) uv_offset: vec2<f32>,\n    @location(5) tint: vec4<f32>,\n    @location(6) model_1x: vec4<f32>,\n    @location(7) model_2x: vec4<f32>,\n    @location(8) model_3x: vec4<f32>,\n    @location(9) model_4x: vec4<f32>,\n    @location(10) normal_1x: vec4<f32>,\n    @location(11) normal_2x: vec4<f32>,\n    @location(12) normal_3x: vec4<f32>,\n    @location(13) normal_4x: vec4<f32>,\n}\n\nstruct VertexOutput {\n    @builtin(position) clip_position: vec4<f32>,\n}\n\n@group(0) @binding(0) \nvar t_diffuse: texture_2d<f32>;\n@group(0) @binding(1) \nvar s_diffuse: sampler;\n@group(1) @binding(0) \nvar<uniform> global_uniforms: GlobalUniformsX_naga_oil_mod_XM5WG6YTBNQX;\n@group(2) @binding(0) \nvar<uniform> view_proj_uniforms: ViewProjectionUniformsX_naga_oil_mod_XM5WG6YTBNQX;\n\n@fragment \nfn fs_main(in: VertexOutput) {\n    return;\n}\n";
        }
    }
    #[allow(unused)]
    ///Equivalent Rust definitions of the types defined in this module.
    pub mod types {}
    #[allow(unused)]
    use types::*;
    ///The sourcecode for the shader, as a constant string.
    pub const SOURCE: &'static str = "struct GlobalUniformsX_naga_oil_mod_XM5WG6YTBNQX {\n    time: f32,\n    screen_size: vec2<f32>,\n}\n\nstruct ViewProjectionUniformsX_naga_oil_mod_XM5WG6YTBNQX {\n    view: mat4x4<f32>,\n    projection: mat4x4<f32>,\n    camera_pos: vec3<f32>,\n    inverse_view: mat4x4<f32>,\n}\n\nstruct VertexInputX_naga_oil_mod_XNFXHA5LUOMX {\n    @location(0) position: vec4<f32>,\n    @location(1) tex_coords: vec2<f32>,\n    @location(2) normal: vec3<f32>,\n}\n\nstruct InstanceInputX_naga_oil_mod_XNFXHA5LUOMX {\n    @location(3) uv_scale: vec2<f32>,\n    @location(4) uv_offset: vec2<f32>,\n    @location(5) tint: vec4<f32>,\n    @location(6) model_1x: vec4<f32>,\n    @location(7) model_2x: vec4<f32>,\n    @location(8) model_3x: vec4<f32>,\n    @location(9) model_4x: vec4<f32>,\n    @location(10) normal_1x: vec4<f32>,\n    @location(11) normal_2x: vec4<f32>,\n    @location(12) normal_3x: vec4<f32>,\n    @location(13) normal_4x: vec4<f32>,\n}\n\nstruct VertexOutput {\n    @builtin(position) clip_position: vec4<f32>,\n}\n\n@group(0) @binding(0) \nvar t_diffuse: texture_2d<f32>;\n@group(0) @binding(1) \nvar s_diffuse: sampler;\n@group(1) @binding(0) \nvar<uniform> global_uniforms: GlobalUniformsX_naga_oil_mod_XM5WG6YTBNQX;\n@group(2) @binding(0) \nvar<uniform> view_proj_uniforms: ViewProjectionUniformsX_naga_oil_mod_XM5WG6YTBNQX;\n\n@vertex \nfn vs_main(vertex: VertexInputX_naga_oil_mod_XNFXHA5LUOMX, instance: InstanceInputX_naga_oil_mod_XNFXHA5LUOMX) -> VertexOutput {\n    var out: VertexOutput;\n\n    let model_transform = mat4x4<f32>(instance.model_1x, instance.model_2x, instance.model_3x, instance.model_4x);\n    let normal_matrix = mat4x4<f32>(instance.normal_1x, instance.normal_2x, instance.normal_3x, instance.normal_4x);\n    let _e13 = view_proj_uniforms.view;\n    let model_view = (_e13 * model_transform);\n    let model_view_pos = (model_view * vertex.position);\n    let _e22 = view_proj_uniforms.projection;\n    out.clip_position = (_e22 * model_view_pos);\n    let _e24 = out;\n    return _e24;\n}\n\n@fragment \nfn fs_main(in: VertexOutput) {\n    return;\n}\n";
    ///Shader module descriptor.
    pub const DESCRIPTOR: ::wgpu::ShaderModuleDescriptor = ::wgpu::ShaderModuleDescriptor {
        label: Some("depth_only"),
        source: ::wgpu::ShaderSource::Wgsl(::std::borrow::Cow::Borrowed(SOURCE)),
    };
}
pub mod geometry {
    #[allow(unused)]
    ///Information about the globals within the module, exposed as constants and functions.
    pub mod globals {
        #[allow(unused)]
        use super::*;
        ///Information about the `t_diffuse` global variable within this shader module.
        pub mod t_diffuse {
            #[allow(unused)]
            use super::*;
            pub const NAME: &'static str = "t_diffuse";
            pub const GROUP: u32 = 0u32;
            pub const BINDING: u32 = 0u32;
        }
        ///Information about the `s_diffuse` global variable within this shader module.
        pub mod s_diffuse {
            #[allow(unused)]
            use super::*;
            pub const NAME: &'static str = "s_diffuse";
            pub const GROUP: u32 = 0u32;
            pub const BINDING: u32 = 1u32;
        }
        ///Information about the `global_uniforms` global variable within this shader module.
        pub mod global_uniforms {
            #[allow(unused)]
            use super::*;
            pub const NAME: &'static str = "global_uniforms";
            pub type Ty = super::super::super::global::types::GlobalUniforms;
            pub const GROUP: u32 = 1u32;
            pub const BINDING: u32 = 0u32;
            pub fn layout() -> ::wgpu::BindGroupLayoutDescriptor<'static> {
                ::wgpu::BindGroupLayoutDescriptor {
                    label: Some("global_uniforms"),
                    entries: &[
                        ::wgpu::BindGroupLayoutEntry {
                            binding: 0u32,
                            visibility: ::wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: ::wgpu::BindingType::Buffer {
                                ty: ::wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                }
            }
        }
        ///Information about the `view_proj_uniforms` global variable within this shader module.
        pub mod view_proj_uniforms {
            #[allow(unused)]
            use super::*;
            pub const NAME: &'static str = "view_proj_uniforms";
            pub type Ty = super::super::super::global::types::ViewProjectionUniforms;
            pub const GROUP: u32 = 2u32;
            pub const BINDING: u32 = 0u32;
            pub fn layout() -> ::wgpu::BindGroupLayoutDescriptor<'static> {
                ::wgpu::BindGroupLayoutDescriptor {
                    label: Some("view_proj_uniforms"),
                    entries: &[
                        ::wgpu::BindGroupLayoutEntry {
                            binding: 0u32,
                            visibility: ::wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: ::wgpu::BindingType::Buffer {
                                ty: ::wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                }
            }
        }
        ///Contains the following bindings: t_diffuse, s_diffuse
        pub mod group0 {
            #[allow(unused)]
            use super::*;
            pub const GROUP: u32 = 0u32;
            pub fn layout() -> ::wgpu::BindGroupLayoutDescriptor<'static> {
                ::wgpu::BindGroupLayoutDescriptor {
                    label: Some("group0"),
                    entries: &[
                        ::wgpu::BindGroupLayoutEntry {
                            binding: 0u32,
                            visibility: ::wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: ::wgpu::TextureViewDimension::D2,
                                sample_type: ::wgpu::TextureSampleType::Float {
                                    filterable: true,
                                },
                            },
                            count: None,
                        },
                        ::wgpu::BindGroupLayoutEntry {
                            binding: 1u32,
                            visibility: ::wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: ::wgpu::BindingType::Sampler(
                                ::wgpu::SamplerBindingType::Filtering,
                            ),
                            count: None,
                        },
                    ],
                }
            }
        }
    }
    #[allow(unused)]
    ///Information about the constants within the module, exposed as constants and functions.
    pub mod constants {
        #[allow(unused)]
        use super::*;
    }
    #[allow(unused)]
    ///Information about the entry points within the module, exposed as constants and functions.
    pub mod entry_points {
        #[allow(unused)]
        use super::*;
        pub mod vs_main {
            pub const NAME: &'static str = "vs_main";
            ///The sourcecode for the shader, as a constant string, excluding any other entry points. This is useful when the `minify` feature is enabled for this crate, as it allows more aggressive minification to be performed with the knowledge of the specific entry point that will be used.
            pub const EXCLUSIVE_SOURCE: &'static str = "struct GlobalUniformsX_naga_oil_mod_XM5WG6YTBNQX {\n    time: f32,\n    screen_size: vec2<f32>,\n}\n\nstruct ViewProjectionUniformsX_naga_oil_mod_XM5WG6YTBNQX {\n    view: mat4x4<f32>,\n    projection: mat4x4<f32>,\n    camera_pos: vec3<f32>,\n    inverse_view: mat4x4<f32>,\n}\n\nstruct VertexInput {\n    @location(0) position: vec4<f32>,\n    @location(1) tex_coords: vec2<f32>,\n    @location(2) normal: vec3<f32>,\n}\n\nstruct InstanceInput {\n    @location(3) uv_scale: vec2<f32>,\n    @location(4) uv_offset: vec2<f32>,\n    @location(5) tint: vec4<f32>,\n    @location(6) model_1_: vec4<f32>,\n    @location(7) model_2_: vec4<f32>,\n    @location(8) model_3_: vec4<f32>,\n    @location(9) model_4_: vec4<f32>,\n    @location(10) normal_1_: vec4<f32>,\n    @location(11) normal_2_: vec4<f32>,\n    @location(12) normal_3_: vec4<f32>,\n    @location(13) normal_4_: vec4<f32>,\n}\n\nstruct VertexOutput {\n    @builtin(position) clip_position: vec4<f32>,\n    @location(0) tex_coords: vec2<f32>,\n    @location(1) view_pos: vec4<f32>,\n    @location(2) view_space_normal: vec4<f32>,\n    @location(3) tint_color: vec4<f32>,\n}\n\nstruct FragmentOutput {\n    @location(0) g_position: vec4<f32>,\n    @location(1) g_normal: vec4<f32>,\n    @location(2) g_albedo_spec: vec4<f32>,\n}\n\n@group(0) @binding(0) \nvar t_diffuse: texture_2d<f32>;\n@group(0) @binding(1) \nvar s_diffuse: sampler;\n@group(1) @binding(0) \nvar<uniform> global_uniforms: GlobalUniformsX_naga_oil_mod_XM5WG6YTBNQX;\n@group(2) @binding(0) \nvar<uniform> view_proj_uniforms: ViewProjectionUniformsX_naga_oil_mod_XM5WG6YTBNQX;\n\n@vertex \nfn vs_main(vertex: VertexInput, instance: InstanceInput) -> VertexOutput {\n    var out: VertexOutput;\n\n    let model_transform = mat4x4<f32>(instance.model_1_, instance.model_2_, instance.model_3_, instance.model_4_);\n    let normal_matrix = mat4x4<f32>(instance.normal_1_, instance.normal_2_, instance.normal_3_, instance.normal_4_);\n    out.tex_coords = (instance.uv_offset + (instance.uv_scale * vertex.tex_coords));\n    let _e21 = view_proj_uniforms.view;\n    let model_view = (_e21 * model_transform);\n    let model_view_pos = (model_view * vertex.position);\n    let _e28 = view_proj_uniforms.projection;\n    out.clip_position = (_e28 * model_view_pos);\n    out.view_space_normal = (normal_matrix * vec4<f32>(vertex.normal, 1f));\n    out.view_pos = model_view_pos;\n    out.tint_color = instance.tint;\n    let _e38 = out;\n    return _e38;\n}\n";
        }
        pub mod fs_main {
            pub const NAME: &'static str = "fs_main";
            ///The sourcecode for the shader, as a constant string, excluding any other entry points. This is useful when the `minify` feature is enabled for this crate, as it allows more aggressive minification to be performed with the knowledge of the specific entry point that will be used.
            pub const EXCLUSIVE_SOURCE: &'static str = "struct GlobalUniformsX_naga_oil_mod_XM5WG6YTBNQX {\n    time: f32,\n    screen_size: vec2<f32>,\n}\n\nstruct ViewProjectionUniformsX_naga_oil_mod_XM5WG6YTBNQX {\n    view: mat4x4<f32>,\n    projection: mat4x4<f32>,\n    camera_pos: vec3<f32>,\n    inverse_view: mat4x4<f32>,\n}\n\nstruct VertexInput {\n    @location(0) position: vec4<f32>,\n    @location(1) tex_coords: vec2<f32>,\n    @location(2) normal: vec3<f32>,\n}\n\nstruct InstanceInput {\n    @location(3) uv_scale: vec2<f32>,\n    @location(4) uv_offset: vec2<f32>,\n    @location(5) tint: vec4<f32>,\n    @location(6) model_1_: vec4<f32>,\n    @location(7) model_2_: vec4<f32>,\n    @location(8) model_3_: vec4<f32>,\n    @location(9) model_4_: vec4<f32>,\n    @location(10) normal_1_: vec4<f32>,\n    @location(11) normal_2_: vec4<f32>,\n    @location(12) normal_3_: vec4<f32>,\n    @location(13) normal_4_: vec4<f32>,\n}\n\nstruct VertexOutput {\n    @builtin(position) clip_position: vec4<f32>,\n    @location(0) tex_coords: vec2<f32>,\n    @location(1) view_pos: vec4<f32>,\n    @location(2) view_space_normal: vec4<f32>,\n    @location(3) tint_color: vec4<f32>,\n}\n\nstruct FragmentOutput {\n    @location(0) g_position: vec4<f32>,\n    @location(1) g_normal: vec4<f32>,\n    @location(2) g_albedo_spec: vec4<f32>,\n}\n\n@group(0) @binding(0) \nvar t_diffuse: texture_2d<f32>;\n@group(0) @binding(1) \nvar s_diffuse: sampler;\n@group(1) @binding(0) \nvar<uniform> global_uniforms: GlobalUniformsX_naga_oil_mod_XM5WG6YTBNQX;\n@group(2) @binding(0) \nvar<uniform> view_proj_uniforms: ViewProjectionUniformsX_naga_oil_mod_XM5WG6YTBNQX;\n\n@fragment \nfn fs_main(in: VertexOutput) -> FragmentOutput {\n    var out: FragmentOutput;\n\n    out.g_position = in.view_pos;\n    out.g_normal = normalize(in.view_space_normal);\n    let _e12 = textureSample(t_diffuse, s_diffuse, in.tex_coords);\n    out.g_albedo_spec = (in.tint_color * _e12);\n    let _e14 = out;\n    return _e14;\n}\n";
        }
    }
    #[allow(unused)]
    ///Equivalent Rust definitions of the types defined in this module.
    pub mod types {}
    #[allow(unused)]
    use types::*;
    ///The sourcecode for the shader, as a constant string.
    pub const SOURCE: &'static str = "struct GlobalUniformsX_naga_oil_mod_XM5WG6YTBNQX {\n    time: f32,\n    screen_size: vec2<f32>,\n}\n\nstruct ViewProjectionUniformsX_naga_oil_mod_XM5WG6YTBNQX {\n    view: mat4x4<f32>,\n    projection: mat4x4<f32>,\n    camera_pos: vec3<f32>,\n    inverse_view: mat4x4<f32>,\n}\n\nstruct VertexInput {\n    @location(0) position: vec4<f32>,\n    @location(1) tex_coords: vec2<f32>,\n    @location(2) normal: vec3<f32>,\n}\n\nstruct InstanceInput {\n    @location(3) uv_scale: vec2<f32>,\n    @location(4) uv_offset: vec2<f32>,\n    @location(5) tint: vec4<f32>,\n    @location(6) model_1_: vec4<f32>,\n    @location(7) model_2_: vec4<f32>,\n    @location(8) model_3_: vec4<f32>,\n    @location(9) model_4_: vec4<f32>,\n    @location(10) normal_1_: vec4<f32>,\n    @location(11) normal_2_: vec4<f32>,\n    @location(12) normal_3_: vec4<f32>,\n    @location(13) normal_4_: vec4<f32>,\n}\n\nstruct VertexOutput {\n    @builtin(position) clip_position: vec4<f32>,\n    @location(0) tex_coords: vec2<f32>,\n    @location(1) view_pos: vec4<f32>,\n    @location(2) view_space_normal: vec4<f32>,\n    @location(3) tint_color: vec4<f32>,\n}\n\nstruct FragmentOutput {\n    @location(0) g_position: vec4<f32>,\n    @location(1) g_normal: vec4<f32>,\n    @location(2) g_albedo_spec: vec4<f32>,\n}\n\n@group(0) @binding(0) \nvar t_diffuse: texture_2d<f32>;\n@group(0) @binding(1) \nvar s_diffuse: sampler;\n@group(1) @binding(0) \nvar<uniform> global_uniforms: GlobalUniformsX_naga_oil_mod_XM5WG6YTBNQX;\n@group(2) @binding(0) \nvar<uniform> view_proj_uniforms: ViewProjectionUniformsX_naga_oil_mod_XM5WG6YTBNQX;\n\n@vertex \nfn vs_main(vertex: VertexInput, instance: InstanceInput) -> VertexOutput {\n    var out: VertexOutput;\n\n    let model_transform = mat4x4<f32>(instance.model_1_, instance.model_2_, instance.model_3_, instance.model_4_);\n    let normal_matrix = mat4x4<f32>(instance.normal_1_, instance.normal_2_, instance.normal_3_, instance.normal_4_);\n    out.tex_coords = (instance.uv_offset + (instance.uv_scale * vertex.tex_coords));\n    let _e21 = view_proj_uniforms.view;\n    let model_view = (_e21 * model_transform);\n    let model_view_pos = (model_view * vertex.position);\n    let _e28 = view_proj_uniforms.projection;\n    out.clip_position = (_e28 * model_view_pos);\n    out.view_space_normal = (normal_matrix * vec4<f32>(vertex.normal, 1f));\n    out.view_pos = model_view_pos;\n    out.tint_color = instance.tint;\n    let _e38 = out;\n    return _e38;\n}\n\n@fragment \nfn fs_main(in: VertexOutput) -> FragmentOutput {\n    var out_1: FragmentOutput;\n\n    out_1.g_position = in.view_pos;\n    out_1.g_normal = normalize(in.view_space_normal);\n    let _e12 = textureSample(t_diffuse, s_diffuse, in.tex_coords);\n    out_1.g_albedo_spec = (in.tint_color * _e12);\n    let _e14 = out_1;\n    return _e14;\n}\n";
    ///Shader module descriptor.
    pub const DESCRIPTOR: ::wgpu::ShaderModuleDescriptor = ::wgpu::ShaderModuleDescriptor {
        label: Some("geometry"),
        source: ::wgpu::ShaderSource::Wgsl(::std::borrow::Cow::Borrowed(SOURCE)),
    };
}
pub mod ssao {
    #[allow(unused)]
    ///Information about the globals within the module, exposed as constants and functions.
    pub mod globals {
        #[allow(unused)]
        use super::*;
        ///Information about the `t_diffuse` global variable within this shader module.
        pub mod t_diffuse {
            #[allow(unused)]
            use super::*;
            pub const NAME: &'static str = "t_diffuse";
            pub const GROUP: u32 = 0u32;
            pub const BINDING: u32 = 0u32;
        }
        ///Information about the `s_diffuse` global variable within this shader module.
        pub mod s_diffuse {
            #[allow(unused)]
            use super::*;
            pub const NAME: &'static str = "s_diffuse";
            pub const GROUP: u32 = 0u32;
            pub const BINDING: u32 = 1u32;
        }
        ///Information about the `global_uniforms` global variable within this shader module.
        pub mod global_uniforms {
            #[allow(unused)]
            use super::*;
            pub const NAME: &'static str = "global_uniforms";
            pub type Ty = super::super::super::global::types::GlobalUniforms;
            pub const GROUP: u32 = 1u32;
            pub const BINDING: u32 = 0u32;
            pub fn layout() -> ::wgpu::BindGroupLayoutDescriptor<'static> {
                ::wgpu::BindGroupLayoutDescriptor {
                    label: Some("global_uniforms"),
                    entries: &[
                        ::wgpu::BindGroupLayoutEntry {
                            binding: 0u32,
                            visibility: ::wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: ::wgpu::BindingType::Buffer {
                                ty: ::wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                }
            }
        }
        ///Information about the `view_proj_uniforms` global variable within this shader module.
        pub mod view_proj_uniforms {
            #[allow(unused)]
            use super::*;
            pub const NAME: &'static str = "view_proj_uniforms";
            pub type Ty = super::super::super::global::types::ViewProjectionUniforms;
            pub const GROUP: u32 = 2u32;
            pub const BINDING: u32 = 0u32;
            pub fn layout() -> ::wgpu::BindGroupLayoutDescriptor<'static> {
                ::wgpu::BindGroupLayoutDescriptor {
                    label: Some("view_proj_uniforms"),
                    entries: &[
                        ::wgpu::BindGroupLayoutEntry {
                            binding: 0u32,
                            visibility: ::wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: ::wgpu::BindingType::Buffer {
                                ty: ::wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                }
            }
        }
        ///Information about the `g_position` global variable within this shader module.
        pub mod g_position {
            #[allow(unused)]
            use super::*;
            pub const NAME: &'static str = "g_position";
            pub const GROUP: u32 = 3u32;
            pub const BINDING: u32 = 0u32;
        }
        ///Information about the `g_position_sampler` global variable within this shader module.
        pub mod g_position_sampler {
            #[allow(unused)]
            use super::*;
            pub const NAME: &'static str = "g_position_sampler";
            pub const GROUP: u32 = 3u32;
            pub const BINDING: u32 = 1u32;
        }
        ///Information about the `g_normal` global variable within this shader module.
        pub mod g_normal {
            #[allow(unused)]
            use super::*;
            pub const NAME: &'static str = "g_normal";
            pub const GROUP: u32 = 3u32;
            pub const BINDING: u32 = 2u32;
        }
        ///Information about the `g_normal_sampler` global variable within this shader module.
        pub mod g_normal_sampler {
            #[allow(unused)]
            use super::*;
            pub const NAME: &'static str = "g_normal_sampler";
            pub const GROUP: u32 = 3u32;
            pub const BINDING: u32 = 3u32;
        }
        ///Information about the `g_albedo_spec` global variable within this shader module.
        pub mod g_albedo_spec {
            #[allow(unused)]
            use super::*;
            pub const NAME: &'static str = "g_albedo_spec";
            pub const GROUP: u32 = 3u32;
            pub const BINDING: u32 = 4u32;
        }
        ///Information about the `g_albedo_spec_sampler` global variable within this shader module.
        pub mod g_albedo_spec_sampler {
            #[allow(unused)]
            use super::*;
            pub const NAME: &'static str = "g_albedo_spec_sampler";
            pub const GROUP: u32 = 3u32;
            pub const BINDING: u32 = 5u32;
        }
        ///Information about the `kernel` global variable within this shader module.
        pub mod kernel {
            #[allow(unused)]
            use super::*;
            pub const NAME: &'static str = "kernel";
            pub type Ty = Kernel;
            pub const GROUP: u32 = 4u32;
            pub const BINDING: u32 = 0u32;
            pub fn layout() -> ::wgpu::BindGroupLayoutDescriptor<'static> {
                ::wgpu::BindGroupLayoutDescriptor {
                    label: Some("kernel"),
                    entries: &[
                        ::wgpu::BindGroupLayoutEntry {
                            binding: 0u32,
                            visibility: ::wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: ::wgpu::BindingType::Buffer {
                                ty: ::wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                }
            }
        }
        ///Information about the `ssao_noise` global variable within this shader module.
        pub mod ssao_noise {
            #[allow(unused)]
            use super::*;
            pub const NAME: &'static str = "ssao_noise";
            pub const GROUP: u32 = 5u32;
            pub const BINDING: u32 = 0u32;
        }
        ///Information about the `ssao_noise_sampler` global variable within this shader module.
        pub mod ssao_noise_sampler {
            #[allow(unused)]
            use super::*;
            pub const NAME: &'static str = "ssao_noise_sampler";
            pub const GROUP: u32 = 5u32;
            pub const BINDING: u32 = 1u32;
        }
        ///Contains the following bindings: ssao_noise, ssao_noise_sampler
        pub mod group5 {
            #[allow(unused)]
            use super::*;
            pub const GROUP: u32 = 5u32;
            pub fn layout() -> ::wgpu::BindGroupLayoutDescriptor<'static> {
                ::wgpu::BindGroupLayoutDescriptor {
                    label: Some("group5"),
                    entries: &[
                        ::wgpu::BindGroupLayoutEntry {
                            binding: 0u32,
                            visibility: ::wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: ::wgpu::TextureViewDimension::D2,
                                sample_type: ::wgpu::TextureSampleType::Float {
                                    filterable: true,
                                },
                            },
                            count: None,
                        },
                        ::wgpu::BindGroupLayoutEntry {
                            binding: 1u32,
                            visibility: ::wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: ::wgpu::BindingType::Sampler(
                                ::wgpu::SamplerBindingType::Filtering,
                            ),
                            count: None,
                        },
                    ],
                }
            }
        }
        ///Contains the following bindings: g_position, g_position_sampler, g_normal, g_normal_sampler, g_albedo_spec, g_albedo_spec_sampler
        pub mod group3 {
            #[allow(unused)]
            use super::*;
            pub const GROUP: u32 = 3u32;
            pub fn layout() -> ::wgpu::BindGroupLayoutDescriptor<'static> {
                ::wgpu::BindGroupLayoutDescriptor {
                    label: Some("group3"),
                    entries: &[
                        ::wgpu::BindGroupLayoutEntry {
                            binding: 0u32,
                            visibility: ::wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: ::wgpu::TextureViewDimension::D2,
                                sample_type: ::wgpu::TextureSampleType::Float {
                                    filterable: true,
                                },
                            },
                            count: None,
                        },
                        ::wgpu::BindGroupLayoutEntry {
                            binding: 1u32,
                            visibility: ::wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: ::wgpu::BindingType::Sampler(
                                ::wgpu::SamplerBindingType::Filtering,
                            ),
                            count: None,
                        },
                        ::wgpu::BindGroupLayoutEntry {
                            binding: 2u32,
                            visibility: ::wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: ::wgpu::TextureViewDimension::D2,
                                sample_type: ::wgpu::TextureSampleType::Float {
                                    filterable: true,
                                },
                            },
                            count: None,
                        },
                        ::wgpu::BindGroupLayoutEntry {
                            binding: 3u32,
                            visibility: ::wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: ::wgpu::BindingType::Sampler(
                                ::wgpu::SamplerBindingType::Filtering,
                            ),
                            count: None,
                        },
                        ::wgpu::BindGroupLayoutEntry {
                            binding: 4u32,
                            visibility: ::wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: ::wgpu::TextureViewDimension::D2,
                                sample_type: ::wgpu::TextureSampleType::Float {
                                    filterable: true,
                                },
                            },
                            count: None,
                        },
                        ::wgpu::BindGroupLayoutEntry {
                            binding: 5u32,
                            visibility: ::wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: ::wgpu::BindingType::Sampler(
                                ::wgpu::SamplerBindingType::Filtering,
                            ),
                            count: None,
                        },
                    ],
                }
            }
        }
        ///Contains the following bindings: t_diffuse, s_diffuse
        pub mod group0 {
            #[allow(unused)]
            use super::*;
            pub const GROUP: u32 = 0u32;
            pub fn layout() -> ::wgpu::BindGroupLayoutDescriptor<'static> {
                ::wgpu::BindGroupLayoutDescriptor {
                    label: Some("group0"),
                    entries: &[
                        ::wgpu::BindGroupLayoutEntry {
                            binding: 0u32,
                            visibility: ::wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: ::wgpu::TextureViewDimension::D2,
                                sample_type: ::wgpu::TextureSampleType::Float {
                                    filterable: true,
                                },
                            },
                            count: None,
                        },
                        ::wgpu::BindGroupLayoutEntry {
                            binding: 1u32,
                            visibility: ::wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: ::wgpu::BindingType::Sampler(
                                ::wgpu::SamplerBindingType::Filtering,
                            ),
                            count: None,
                        },
                    ],
                }
            }
        }
    }
    #[allow(unused)]
    ///Information about the constants within the module, exposed as constants and functions.
    pub mod constants {
        #[allow(unused)]
        use super::*;
    }
    #[allow(unused)]
    ///Information about the entry points within the module, exposed as constants and functions.
    pub mod entry_points {
        #[allow(unused)]
        use super::*;
        pub mod vs_main {
            pub const NAME: &'static str = "vs_main";
            ///The sourcecode for the shader, as a constant string, excluding any other entry points. This is useful when the `minify` feature is enabled for this crate, as it allows more aggressive minification to be performed with the knowledge of the specific entry point that will be used.
            pub const EXCLUSIVE_SOURCE: &'static str = "struct GlobalUniformsX_naga_oil_mod_XM5WG6YTBNQX {\n    time: f32,\n    screen_size: vec2<f32>,\n}\n\nstruct ViewProjectionUniformsX_naga_oil_mod_XM5WG6YTBNQX {\n    view: mat4x4<f32>,\n    projection: mat4x4<f32>,\n    camera_pos: vec3<f32>,\n    inverse_view: mat4x4<f32>,\n}\n\nstruct VertexInput {\n    @location(0) position: vec4<f32>,\n    @location(1) tex_coords: vec2<f32>,\n}\n\nstruct InstanceInput {\n    @location(2) uv_scale: vec2<f32>,\n    @location(3) uv_offset: vec2<f32>,\n    @location(4) tint: vec4<f32>,\n    @location(5) model_1_: vec4<f32>,\n    @location(6) model_2_: vec4<f32>,\n    @location(7) model_3_: vec4<f32>,\n    @location(8) model_4_: vec4<f32>,\n}\n\nstruct VertexOutput {\n    @builtin(position) clip_position: vec4<f32>,\n    @location(0) tex_coords: vec2<f32>,\n    @location(1) tint_color: vec4<f32>,\n}\n\nstruct Kernel {\n    items: array<vec4<f32>, 64>,\n    count: u32,\n    radius: f32,\n    bias: f32,\n    noise_texture_scale: vec2<f32>,\n}\n\n@group(0) @binding(0) \nvar t_diffuse: texture_2d<f32>;\n@group(0) @binding(1) \nvar s_diffuse: sampler;\n@group(1) @binding(0) \nvar<uniform> global_uniforms: GlobalUniformsX_naga_oil_mod_XM5WG6YTBNQX;\n@group(2) @binding(0) \nvar<uniform> view_proj_uniforms: ViewProjectionUniformsX_naga_oil_mod_XM5WG6YTBNQX;\n@group(3) @binding(0) \nvar g_position: texture_2d<f32>;\n@group(3) @binding(1) \nvar g_position_sampler: sampler;\n@group(3) @binding(2) \nvar g_normal: texture_2d<f32>;\n@group(3) @binding(3) \nvar g_normal_sampler: sampler;\n@group(3) @binding(4) \nvar g_albedo_spec: texture_2d<f32>;\n@group(3) @binding(5) \nvar g_albedo_spec_sampler: sampler;\n@group(4) @binding(0) \nvar<uniform> kernel: Kernel;\n@group(5) @binding(0) \nvar ssao_noise: texture_2d<f32>;\n@group(5) @binding(1) \nvar ssao_noise_sampler: sampler;\n\n@vertex \nfn vs_main(vertex: VertexInput, instance: InstanceInput) -> VertexOutput {\n    var out: VertexOutput;\n    var model: vec4<f32>;\n\n    let model_transform = mat4x4<f32>(instance.model_1_, instance.model_2_, instance.model_3_, instance.model_4_);\n    out.tex_coords = (instance.uv_offset + (instance.uv_scale * vertex.tex_coords));\n    model = vertex.position;\n    let _e18 = model.x;\n    model.x = ((_e18 * 2f) - 1f);\n    let _e25 = model.y;\n    model.y = ((_e25 * 2f) - 1f);\n    let model_view = model;\n    out.clip_position = model_view;\n    out.tint_color = instance.tint;\n    let _e34 = out;\n    return _e34;\n}\n";
        }
        pub mod fs_main {
            pub const NAME: &'static str = "fs_main";
            ///The sourcecode for the shader, as a constant string, excluding any other entry points. This is useful when the `minify` feature is enabled for this crate, as it allows more aggressive minification to be performed with the knowledge of the specific entry point that will be used.
            pub const EXCLUSIVE_SOURCE: &'static str = "struct GlobalUniformsX_naga_oil_mod_XM5WG6YTBNQX {\n    time: f32,\n    screen_size: vec2<f32>,\n}\n\nstruct ViewProjectionUniformsX_naga_oil_mod_XM5WG6YTBNQX {\n    view: mat4x4<f32>,\n    projection: mat4x4<f32>,\n    camera_pos: vec3<f32>,\n    inverse_view: mat4x4<f32>,\n}\n\nstruct VertexInput {\n    @location(0) position: vec4<f32>,\n    @location(1) tex_coords: vec2<f32>,\n}\n\nstruct InstanceInput {\n    @location(2) uv_scale: vec2<f32>,\n    @location(3) uv_offset: vec2<f32>,\n    @location(4) tint: vec4<f32>,\n    @location(5) model_1_: vec4<f32>,\n    @location(6) model_2_: vec4<f32>,\n    @location(7) model_3_: vec4<f32>,\n    @location(8) model_4_: vec4<f32>,\n}\n\nstruct VertexOutput {\n    @builtin(position) clip_position: vec4<f32>,\n    @location(0) tex_coords: vec2<f32>,\n    @location(1) tint_color: vec4<f32>,\n}\n\nstruct Kernel {\n    items: array<vec4<f32>, 64>,\n    count: u32,\n    radius: f32,\n    bias: f32,\n    noise_texture_scale: vec2<f32>,\n}\n\n@group(0) @binding(0) \nvar t_diffuse: texture_2d<f32>;\n@group(0) @binding(1) \nvar s_diffuse: sampler;\n@group(1) @binding(0) \nvar<uniform> global_uniforms: GlobalUniformsX_naga_oil_mod_XM5WG6YTBNQX;\n@group(2) @binding(0) \nvar<uniform> view_proj_uniforms: ViewProjectionUniformsX_naga_oil_mod_XM5WG6YTBNQX;\n@group(3) @binding(0) \nvar g_position: texture_2d<f32>;\n@group(3) @binding(1) \nvar g_position_sampler: sampler;\n@group(3) @binding(2) \nvar g_normal: texture_2d<f32>;\n@group(3) @binding(3) \nvar g_normal_sampler: sampler;\n@group(3) @binding(4) \nvar g_albedo_spec: texture_2d<f32>;\n@group(3) @binding(5) \nvar g_albedo_spec_sampler: sampler;\n@group(4) @binding(0) \nvar<uniform> kernel: Kernel;\n@group(5) @binding(0) \nvar ssao_noise: texture_2d<f32>;\n@group(5) @binding(1) \nvar ssao_noise_sampler: sampler;\n\n@fragment \nfn fs_main(in: VertexOutput) -> @location(0) f32 {\n    var view_space_pos: vec4<f32>;\n    var view_space_normal: vec3<f32>;\n    var random_vec: vec3<f32>;\n    var occlusion: f32 = 0f;\n    var i: i32 = 0i;\n    var sample: vec3<f32>;\n    var offset: vec4<f32>;\n    var sample_depth: f32;\n    var range_check: f32;\n\n    let _e7 = textureSample(g_position, g_position_sampler, in.tex_coords.xy);\n    view_space_pos = _e7;\n    let _e13 = textureSample(g_normal, g_normal_sampler, in.tex_coords.xy);\n    view_space_normal = normalize(_e13.xyz);\n    let _e21 = kernel.noise_texture_scale;\n    let _e25 = textureSample(ssao_noise, ssao_noise_sampler, (_e21 * in.tex_coords.xy));\n    random_vec = _e25.xyz;\n    let _e28 = random_vec;\n    let _e29 = view_space_normal;\n    let _e30 = random_vec;\n    let _e31 = view_space_normal;\n    let tangent = normalize((_e28 - (_e29 * dot(_e30, _e31))));\n    let _e36 = view_space_normal;\n    let bitangent = cross(_e36, tangent);\n    let _e38 = view_space_normal;\n    let TBN = mat3x3<f32>(tangent, bitangent, _e38);\n    loop {\n        let _e41 = i;\n        let _e44 = kernel.count;\n        if (_e41 < i32(_e44)) {\n        } else {\n            break;\n        }\n        {\n            let _e47 = view_space_pos;\n            let _e51 = kernel.radius;\n            let _e55 = i;\n            let _e57 = kernel.items[_e55];\n            sample = (_e47.xyz + ((_e51 * TBN) * _e57.xyz));\n            let _e64 = view_proj_uniforms.projection;\n            let _e65 = sample;\n            offset = (_e64 * vec4<f32>(_e65, 1f));\n            let _e72 = offset.w;\n            let _e73 = offset.x;\n            offset.x = (_e73 / _e72);\n            let _e77 = offset.w;\n            let _e78 = offset.y;\n            offset.y = (_e78 / _e77);\n            let _e82 = offset.x;\n            offset.x = ((_e82 * 0.5f) + 0.5f);\n            let _e89 = offset.y;\n            offset.y = ((_e89 * 0.5f) + 0.5f);\n            let _e96 = offset.y;\n            offset.y = (1f - _e96);\n            let _e101 = offset;\n            let _e103 = textureSample(g_position, g_position_sampler, _e101.xy);\n            sample_depth = _e103.z;\n            let _e110 = kernel.radius;\n            let _e112 = view_space_pos.z;\n            let _e113 = sample_depth;\n            range_check = smoothstep(0f, 1f, (_e110 / abs((_e112 - _e113))));\n            let _e119 = sample_depth;\n            let _e121 = sample.z;\n            let _e124 = kernel.bias;\n            if (_e119 >= (_e121 + _e124)) {\n                let _e128 = range_check;\n                let _e129 = occlusion;\n                occlusion = (_e129 + _e128);\n            }\n        }\n        continuing {\n            let _e132 = i;\n            i = (_e132 + 1i);\n        }\n    }\n    let _e134 = occlusion;\n    let _e137 = kernel.count;\n    occlusion = (1f - (_e134 / f32(_e137)));\n    let _e142 = occlusion;\n    occlusion = pow(_e142, 2f);\n    let _e145 = occlusion;\n    return _e145;\n}\n";
        }
    }
    #[allow(unused)]
    ///Equivalent Rust definitions of the types defined in this module.
    pub mod types {
        #[allow(unused, non_camel_case_types)]
        #[repr(C)]
        #[derive(Debug, PartialEq, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
        pub struct Kernel {
            pub items: [glam::f32::Vec4; 64u32 as usize],
            pub count: u32,
            pub radius: f32,
            pub bias: f32,
            pub _pad_bias: [u8; 4u32 as usize],
            pub noise_texture_scale: glam::f32::Vec2,
            pub _pad: [u8; 8u32 as usize],
        }
    }
    #[allow(unused)]
    use types::*;
    ///The sourcecode for the shader, as a constant string.
    pub const SOURCE: &'static str = "struct GlobalUniformsX_naga_oil_mod_XM5WG6YTBNQX {\n    time: f32,\n    screen_size: vec2<f32>,\n}\n\nstruct ViewProjectionUniformsX_naga_oil_mod_XM5WG6YTBNQX {\n    view: mat4x4<f32>,\n    projection: mat4x4<f32>,\n    camera_pos: vec3<f32>,\n    inverse_view: mat4x4<f32>,\n}\n\nstruct VertexInput {\n    @location(0) position: vec4<f32>,\n    @location(1) tex_coords: vec2<f32>,\n}\n\nstruct InstanceInput {\n    @location(2) uv_scale: vec2<f32>,\n    @location(3) uv_offset: vec2<f32>,\n    @location(4) tint: vec4<f32>,\n    @location(5) model_1_: vec4<f32>,\n    @location(6) model_2_: vec4<f32>,\n    @location(7) model_3_: vec4<f32>,\n    @location(8) model_4_: vec4<f32>,\n}\n\nstruct VertexOutput {\n    @builtin(position) clip_position: vec4<f32>,\n    @location(0) tex_coords: vec2<f32>,\n    @location(1) tint_color: vec4<f32>,\n}\n\nstruct Kernel {\n    items: array<vec4<f32>, 64>,\n    count: u32,\n    radius: f32,\n    bias: f32,\n    noise_texture_scale: vec2<f32>,\n}\n\n@group(0) @binding(0) \nvar t_diffuse: texture_2d<f32>;\n@group(0) @binding(1) \nvar s_diffuse: sampler;\n@group(1) @binding(0) \nvar<uniform> global_uniforms: GlobalUniformsX_naga_oil_mod_XM5WG6YTBNQX;\n@group(2) @binding(0) \nvar<uniform> view_proj_uniforms: ViewProjectionUniformsX_naga_oil_mod_XM5WG6YTBNQX;\n@group(3) @binding(0) \nvar g_position: texture_2d<f32>;\n@group(3) @binding(1) \nvar g_position_sampler: sampler;\n@group(3) @binding(2) \nvar g_normal: texture_2d<f32>;\n@group(3) @binding(3) \nvar g_normal_sampler: sampler;\n@group(3) @binding(4) \nvar g_albedo_spec: texture_2d<f32>;\n@group(3) @binding(5) \nvar g_albedo_spec_sampler: sampler;\n@group(4) @binding(0) \nvar<uniform> kernel: Kernel;\n@group(5) @binding(0) \nvar ssao_noise: texture_2d<f32>;\n@group(5) @binding(1) \nvar ssao_noise_sampler: sampler;\n\n@vertex \nfn vs_main(vertex: VertexInput, instance: InstanceInput) -> VertexOutput {\n    var out: VertexOutput;\n    var model: vec4<f32>;\n\n    let model_transform = mat4x4<f32>(instance.model_1_, instance.model_2_, instance.model_3_, instance.model_4_);\n    out.tex_coords = (instance.uv_offset + (instance.uv_scale * vertex.tex_coords));\n    model = vertex.position;\n    let _e18 = model.x;\n    model.x = ((_e18 * 2f) - 1f);\n    let _e25 = model.y;\n    model.y = ((_e25 * 2f) - 1f);\n    let model_view = model;\n    out.clip_position = model_view;\n    out.tint_color = instance.tint;\n    let _e34 = out;\n    return _e34;\n}\n\n@fragment \nfn fs_main(in: VertexOutput) -> @location(0) f32 {\n    var view_space_pos: vec4<f32>;\n    var view_space_normal: vec3<f32>;\n    var random_vec: vec3<f32>;\n    var occlusion: f32 = 0f;\n    var i: i32 = 0i;\n    var sample: vec3<f32>;\n    var offset: vec4<f32>;\n    var sample_depth: f32;\n    var range_check: f32;\n\n    let _e7 = textureSample(g_position, g_position_sampler, in.tex_coords.xy);\n    view_space_pos = _e7;\n    let _e13 = textureSample(g_normal, g_normal_sampler, in.tex_coords.xy);\n    view_space_normal = normalize(_e13.xyz);\n    let _e21 = kernel.noise_texture_scale;\n    let _e25 = textureSample(ssao_noise, ssao_noise_sampler, (_e21 * in.tex_coords.xy));\n    random_vec = _e25.xyz;\n    let _e28 = random_vec;\n    let _e29 = view_space_normal;\n    let _e30 = random_vec;\n    let _e31 = view_space_normal;\n    let tangent = normalize((_e28 - (_e29 * dot(_e30, _e31))));\n    let _e36 = view_space_normal;\n    let bitangent = cross(_e36, tangent);\n    let _e38 = view_space_normal;\n    let TBN = mat3x3<f32>(tangent, bitangent, _e38);\n    loop {\n        let _e41 = i;\n        let _e44 = kernel.count;\n        if (_e41 < i32(_e44)) {\n        } else {\n            break;\n        }\n        {\n            let _e47 = view_space_pos;\n            let _e51 = kernel.radius;\n            let _e55 = i;\n            let _e57 = kernel.items[_e55];\n            sample = (_e47.xyz + ((_e51 * TBN) * _e57.xyz));\n            let _e64 = view_proj_uniforms.projection;\n            let _e65 = sample;\n            offset = (_e64 * vec4<f32>(_e65, 1f));\n            let _e72 = offset.w;\n            let _e73 = offset.x;\n            offset.x = (_e73 / _e72);\n            let _e77 = offset.w;\n            let _e78 = offset.y;\n            offset.y = (_e78 / _e77);\n            let _e82 = offset.x;\n            offset.x = ((_e82 * 0.5f) + 0.5f);\n            let _e89 = offset.y;\n            offset.y = ((_e89 * 0.5f) + 0.5f);\n            let _e96 = offset.y;\n            offset.y = (1f - _e96);\n            let _e101 = offset;\n            let _e103 = textureSample(g_position, g_position_sampler, _e101.xy);\n            sample_depth = _e103.z;\n            let _e110 = kernel.radius;\n            let _e112 = view_space_pos.z;\n            let _e113 = sample_depth;\n            range_check = smoothstep(0f, 1f, (_e110 / abs((_e112 - _e113))));\n            let _e119 = sample_depth;\n            let _e121 = sample.z;\n            let _e124 = kernel.bias;\n            if (_e119 >= (_e121 + _e124)) {\n                let _e128 = range_check;\n                let _e129 = occlusion;\n                occlusion = (_e129 + _e128);\n            }\n        }\n        continuing {\n            let _e132 = i;\n            i = (_e132 + 1i);\n        }\n    }\n    let _e134 = occlusion;\n    let _e137 = kernel.count;\n    occlusion = (1f - (_e134 / f32(_e137)));\n    let _e142 = occlusion;\n    occlusion = pow(_e142, 2f);\n    let _e145 = occlusion;\n    return _e145;\n}\n";
    ///Shader module descriptor.
    pub const DESCRIPTOR: ::wgpu::ShaderModuleDescriptor = ::wgpu::ShaderModuleDescriptor {
        label: Some("ssao"),
        source: ::wgpu::ShaderSource::Wgsl(::std::borrow::Cow::Borrowed(SOURCE)),
    };
}
pub mod inputs {
    #[allow(unused)]
    ///Information about the globals within the module, exposed as constants and functions.
    pub mod globals {
        #[allow(unused)]
        use super::*;
    }
    #[allow(unused)]
    ///Information about the constants within the module, exposed as constants and functions.
    pub mod constants {
        #[allow(unused)]
        use super::*;
    }
    #[allow(unused)]
    ///Information about the entry points within the module, exposed as constants and functions.
    pub mod entry_points {
        #[allow(unused)]
        use super::*;
    }
    #[allow(unused)]
    ///Equivalent Rust definitions of the types defined in this module.
    pub mod types {
        #[allow(unused, non_camel_case_types)]
        #[repr(C)]
        #[derive(Debug, PartialEq, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
        pub struct VertexInput {
            pub position: glam::f32::Vec4,
            pub tex_coords: glam::f32::Vec2,
            pub normal: glam::f32::Vec3,
        }
        impl VertexInput {
            pub const VERTEX_ATTRIBUTES: [::wgpu::VertexAttribute; 3usize] = [
                ::wgpu::VertexAttribute {
                    format: ::wgpu::VertexFormat::Float32x4,
                    offset: 0u64,
                    shader_location: 0u32,
                },
                ::wgpu::VertexAttribute {
                    format: ::wgpu::VertexFormat::Float32x2,
                    offset: 16u64,
                    shader_location: 1u32,
                },
                ::wgpu::VertexAttribute {
                    format: ::wgpu::VertexFormat::Float32x3,
                    offset: 24u64,
                    shader_location: 2u32,
                },
            ];
            pub fn vertex_buffer_layout() -> ::wgpu::VertexBufferLayout<'static> {
                ::wgpu::VertexBufferLayout {
                    array_stride: ::std::mem::size_of::<Self>() as ::wgpu::BufferAddress,
                    step_mode: ::wgpu::VertexStepMode::Vertex,
                    attributes: &Self::VERTEX_ATTRIBUTES,
                }
            }
        }
        #[allow(unused, non_camel_case_types)]
        #[repr(C)]
        #[derive(Debug, PartialEq, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
        pub struct InstanceInput {
            pub uv_scale: glam::f32::Vec2,
            pub uv_offset: glam::f32::Vec2,
            pub tint: glam::f32::Vec4,
            pub model_1x: glam::f32::Vec4,
            pub model_2x: glam::f32::Vec4,
            pub model_3x: glam::f32::Vec4,
            pub model_4x: glam::f32::Vec4,
            pub normal_1x: glam::f32::Vec4,
            pub normal_2x: glam::f32::Vec4,
            pub normal_3x: glam::f32::Vec4,
            pub normal_4x: glam::f32::Vec4,
        }
        impl InstanceInput {
            pub const VERTEX_ATTRIBUTES: [::wgpu::VertexAttribute; 11usize] = [
                ::wgpu::VertexAttribute {
                    format: ::wgpu::VertexFormat::Float32x2,
                    offset: 0u64,
                    shader_location: 0u32,
                },
                ::wgpu::VertexAttribute {
                    format: ::wgpu::VertexFormat::Float32x2,
                    offset: 8u64,
                    shader_location: 1u32,
                },
                ::wgpu::VertexAttribute {
                    format: ::wgpu::VertexFormat::Float32x4,
                    offset: 16u64,
                    shader_location: 2u32,
                },
                ::wgpu::VertexAttribute {
                    format: ::wgpu::VertexFormat::Float32x4,
                    offset: 32u64,
                    shader_location: 3u32,
                },
                ::wgpu::VertexAttribute {
                    format: ::wgpu::VertexFormat::Float32x4,
                    offset: 48u64,
                    shader_location: 4u32,
                },
                ::wgpu::VertexAttribute {
                    format: ::wgpu::VertexFormat::Float32x4,
                    offset: 64u64,
                    shader_location: 5u32,
                },
                ::wgpu::VertexAttribute {
                    format: ::wgpu::VertexFormat::Float32x4,
                    offset: 80u64,
                    shader_location: 6u32,
                },
                ::wgpu::VertexAttribute {
                    format: ::wgpu::VertexFormat::Float32x4,
                    offset: 96u64,
                    shader_location: 7u32,
                },
                ::wgpu::VertexAttribute {
                    format: ::wgpu::VertexFormat::Float32x4,
                    offset: 112u64,
                    shader_location: 8u32,
                },
                ::wgpu::VertexAttribute {
                    format: ::wgpu::VertexFormat::Float32x4,
                    offset: 128u64,
                    shader_location: 9u32,
                },
                ::wgpu::VertexAttribute {
                    format: ::wgpu::VertexFormat::Float32x4,
                    offset: 144u64,
                    shader_location: 10u32,
                },
            ];
            pub fn vertex_buffer_layout() -> ::wgpu::VertexBufferLayout<'static> {
                ::wgpu::VertexBufferLayout {
                    array_stride: ::std::mem::size_of::<Self>() as ::wgpu::BufferAddress,
                    step_mode: ::wgpu::VertexStepMode::Vertex,
                    attributes: &Self::VERTEX_ATTRIBUTES,
                }
            }
        }
    }
    #[allow(unused)]
    use types::*;
    ///The sourcecode for the shader, as a constant string.
    pub const SOURCE: &'static str = "struct VertexInput {\n    @location(0) position: vec4<f32>,\n    @location(1) tex_coords: vec2<f32>,\n    @location(2) normal: vec3<f32>,\n}\n\nstruct InstanceInput {\n    @location(3) uv_scale: vec2<f32>,\n    @location(4) uv_offset: vec2<f32>,\n    @location(5) tint: vec4<f32>,\n    @location(6) model_1x: vec4<f32>,\n    @location(7) model_2x: vec4<f32>,\n    @location(8) model_3x: vec4<f32>,\n    @location(9) model_4x: vec4<f32>,\n    @location(10) normal_1x: vec4<f32>,\n    @location(11) normal_2x: vec4<f32>,\n    @location(12) normal_3x: vec4<f32>,\n    @location(13) normal_4x: vec4<f32>,\n}\n\n";
    ///Shader module descriptor.
    pub const DESCRIPTOR: ::wgpu::ShaderModuleDescriptor = ::wgpu::ShaderModuleDescriptor {
        label: Some("inputs"),
        source: ::wgpu::ShaderSource::Wgsl(::std::borrow::Cow::Borrowed(SOURCE)),
    };
}
pub mod global {
    #[allow(unused)]
    ///Information about the globals within the module, exposed as constants and functions.
    pub mod globals {
        #[allow(unused)]
        use super::*;
    }
    #[allow(unused)]
    ///Information about the constants within the module, exposed as constants and functions.
    pub mod constants {
        #[allow(unused)]
        use super::*;
    }
    #[allow(unused)]
    ///Information about the entry points within the module, exposed as constants and functions.
    pub mod entry_points {
        #[allow(unused)]
        use super::*;
        pub mod main {
            pub const NAME: &'static str = "main";
            ///The sourcecode for the shader, as a constant string, excluding any other entry points. This is useful when the `minify` feature is enabled for this crate, as it allows more aggressive minification to be performed with the knowledge of the specific entry point that will be used.
            pub const EXCLUSIVE_SOURCE: &'static str = "struct GlobalUniforms {\n    time: f32,\n    screen_size: vec2<f32>,\n}\n\nstruct ViewProjectionUniforms {\n    view: mat4x4<f32>,\n    projection: mat4x4<f32>,\n    camera_pos: vec3<f32>,\n    inverse_view: mat4x4<f32>,\n}\n\nstruct ModelVertexData {\n    @location(0) position: vec4<f32>,\n    @location(1) tex_coords: vec2<f32>,\n    @location(2) normal: vec3<f32>,\n}\n\n@fragment \nfn main() {\n    return;\n}\n";
        }
    }
    #[allow(unused)]
    ///Equivalent Rust definitions of the types defined in this module.
    pub mod types {
        #[allow(unused, non_camel_case_types)]
        #[repr(C)]
        #[derive(Debug, PartialEq, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
        pub struct GlobalUniforms {
            pub time: f32,
            pub _pad_time: [u8; 4u32 as usize],
            pub screen_size: glam::f32::Vec2,
        }
        #[allow(unused, non_camel_case_types)]
        #[repr(C)]
        #[derive(Debug, PartialEq, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
        pub struct ViewProjectionUniforms {
            pub view: glam::f32::Mat4,
            pub projection: glam::f32::Mat4,
            pub camera_pos: glam::f32::Vec3,
            pub _pad_camera_pos: [u8; 4u32 as usize],
            pub inverse_view: glam::f32::Mat4,
        }
        #[allow(unused, non_camel_case_types)]
        #[repr(C)]
        #[derive(Debug, PartialEq, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
        pub struct ModelVertexData {
            pub position: glam::f32::Vec4,
            pub tex_coords: glam::f32::Vec2,
            pub normal: glam::f32::Vec3,
        }
        impl ModelVertexData {
            pub const VERTEX_ATTRIBUTES: [::wgpu::VertexAttribute; 3usize] = [
                ::wgpu::VertexAttribute {
                    format: ::wgpu::VertexFormat::Float32x4,
                    offset: 0u64,
                    shader_location: 0u32,
                },
                ::wgpu::VertexAttribute {
                    format: ::wgpu::VertexFormat::Float32x2,
                    offset: 16u64,
                    shader_location: 1u32,
                },
                ::wgpu::VertexAttribute {
                    format: ::wgpu::VertexFormat::Float32x3,
                    offset: 24u64,
                    shader_location: 2u32,
                },
            ];
            pub fn vertex_buffer_layout() -> ::wgpu::VertexBufferLayout<'static> {
                ::wgpu::VertexBufferLayout {
                    array_stride: ::std::mem::size_of::<Self>() as ::wgpu::BufferAddress,
                    step_mode: ::wgpu::VertexStepMode::Vertex,
                    attributes: &Self::VERTEX_ATTRIBUTES,
                }
            }
        }
    }
    #[allow(unused)]
    use types::*;
    ///The sourcecode for the shader, as a constant string.
    pub const SOURCE: &'static str = "struct GlobalUniforms {\n    time: f32,\n    screen_size: vec2<f32>,\n}\n\nstruct ViewProjectionUniforms {\n    view: mat4x4<f32>,\n    projection: mat4x4<f32>,\n    camera_pos: vec3<f32>,\n    inverse_view: mat4x4<f32>,\n}\n\nstruct ModelVertexData {\n    @location(0) position: vec4<f32>,\n    @location(1) tex_coords: vec2<f32>,\n    @location(2) normal: vec3<f32>,\n}\n\n@fragment \nfn main() {\n    return;\n}\n";
    ///Shader module descriptor.
    pub const DESCRIPTOR: ::wgpu::ShaderModuleDescriptor = ::wgpu::ShaderModuleDescriptor {
        label: Some("global"),
        source: ::wgpu::ShaderSource::Wgsl(::std::borrow::Cow::Borrowed(SOURCE)),
    };
}
pub mod shadow_map {
    #[allow(unused)]
    ///Information about the globals within the module, exposed as constants and functions.
    pub mod globals {
        #[allow(unused)]
        use super::*;
        ///Information about the `t_diffuse` global variable within this shader module.
        pub mod t_diffuse {
            #[allow(unused)]
            use super::*;
            pub const NAME: &'static str = "t_diffuse";
            pub const GROUP: u32 = 0u32;
            pub const BINDING: u32 = 0u32;
        }
        ///Information about the `s_diffuse` global variable within this shader module.
        pub mod s_diffuse {
            #[allow(unused)]
            use super::*;
            pub const NAME: &'static str = "s_diffuse";
            pub const GROUP: u32 = 0u32;
            pub const BINDING: u32 = 1u32;
        }
        ///Information about the `global_uniforms` global variable within this shader module.
        pub mod global_uniforms {
            #[allow(unused)]
            use super::*;
            pub const NAME: &'static str = "global_uniforms";
            pub type Ty = super::super::super::global::types::GlobalUniforms;
            pub const GROUP: u32 = 1u32;
            pub const BINDING: u32 = 0u32;
            pub fn layout() -> ::wgpu::BindGroupLayoutDescriptor<'static> {
                ::wgpu::BindGroupLayoutDescriptor {
                    label: Some("global_uniforms"),
                    entries: &[
                        ::wgpu::BindGroupLayoutEntry {
                            binding: 0u32,
                            visibility: ::wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: ::wgpu::BindingType::Buffer {
                                ty: ::wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                }
            }
        }
        ///Information about the `view_proj_uniforms` global variable within this shader module.
        pub mod view_proj_uniforms {
            #[allow(unused)]
            use super::*;
            pub const NAME: &'static str = "view_proj_uniforms";
            pub type Ty = super::super::super::global::types::ViewProjectionUniforms;
            pub const GROUP: u32 = 2u32;
            pub const BINDING: u32 = 0u32;
            pub fn layout() -> ::wgpu::BindGroupLayoutDescriptor<'static> {
                ::wgpu::BindGroupLayoutDescriptor {
                    label: Some("view_proj_uniforms"),
                    entries: &[
                        ::wgpu::BindGroupLayoutEntry {
                            binding: 0u32,
                            visibility: ::wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: ::wgpu::BindingType::Buffer {
                                ty: ::wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                }
            }
        }
        ///Contains the following bindings: t_diffuse, s_diffuse
        pub mod group0 {
            #[allow(unused)]
            use super::*;
            pub const GROUP: u32 = 0u32;
            pub fn layout() -> ::wgpu::BindGroupLayoutDescriptor<'static> {
                ::wgpu::BindGroupLayoutDescriptor {
                    label: Some("group0"),
                    entries: &[
                        ::wgpu::BindGroupLayoutEntry {
                            binding: 0u32,
                            visibility: ::wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: ::wgpu::TextureViewDimension::D2,
                                sample_type: ::wgpu::TextureSampleType::Float {
                                    filterable: true,
                                },
                            },
                            count: None,
                        },
                        ::wgpu::BindGroupLayoutEntry {
                            binding: 1u32,
                            visibility: ::wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: ::wgpu::BindingType::Sampler(
                                ::wgpu::SamplerBindingType::Filtering,
                            ),
                            count: None,
                        },
                    ],
                }
            }
        }
    }
    #[allow(unused)]
    ///Information about the constants within the module, exposed as constants and functions.
    pub mod constants {
        #[allow(unused)]
        use super::*;
    }
    #[allow(unused)]
    ///Information about the entry points within the module, exposed as constants and functions.
    pub mod entry_points {
        #[allow(unused)]
        use super::*;
        pub mod vs_main {
            pub const NAME: &'static str = "vs_main";
            ///The sourcecode for the shader, as a constant string, excluding any other entry points. This is useful when the `minify` feature is enabled for this crate, as it allows more aggressive minification to be performed with the knowledge of the specific entry point that will be used.
            pub const EXCLUSIVE_SOURCE: &'static str = "struct GlobalUniformsX_naga_oil_mod_XM5WG6YTBNQX {\n    time: f32,\n    screen_size: vec2<f32>,\n}\n\nstruct ViewProjectionUniformsX_naga_oil_mod_XM5WG6YTBNQX {\n    view: mat4x4<f32>,\n    projection: mat4x4<f32>,\n    camera_pos: vec3<f32>,\n    inverse_view: mat4x4<f32>,\n}\n\nstruct VertexInputX_naga_oil_mod_XNFXHA5LUOMX {\n    @location(0) position: vec4<f32>,\n    @location(1) tex_coords: vec2<f32>,\n    @location(2) normal: vec3<f32>,\n}\n\nstruct InstanceInputX_naga_oil_mod_XNFXHA5LUOMX {\n    @location(3) uv_scale: vec2<f32>,\n    @location(4) uv_offset: vec2<f32>,\n    @location(5) tint: vec4<f32>,\n    @location(6) model_1x: vec4<f32>,\n    @location(7) model_2x: vec4<f32>,\n    @location(8) model_3x: vec4<f32>,\n    @location(9) model_4x: vec4<f32>,\n    @location(10) normal_1x: vec4<f32>,\n    @location(11) normal_2x: vec4<f32>,\n    @location(12) normal_3x: vec4<f32>,\n    @location(13) normal_4x: vec4<f32>,\n}\n\n@group(0) @binding(0) \nvar t_diffuse: texture_2d<f32>;\n@group(0) @binding(1) \nvar s_diffuse: sampler;\n@group(1) @binding(0) \nvar<uniform> global_uniforms: GlobalUniformsX_naga_oil_mod_XM5WG6YTBNQX;\n@group(2) @binding(0) \nvar<uniform> view_proj_uniforms: ViewProjectionUniformsX_naga_oil_mod_XM5WG6YTBNQX;\n\n@vertex \nfn vs_main(vertex: VertexInputX_naga_oil_mod_XNFXHA5LUOMX, instance: InstanceInputX_naga_oil_mod_XNFXHA5LUOMX) -> @builtin(position) vec4<f32> {\n    let model_transform = mat4x4<f32>(instance.model_1x, instance.model_2x, instance.model_3x, instance.model_4x);\n    let model = (model_transform * vertex.position);\n    let _e11 = view_proj_uniforms.view;\n    let model_view = (_e11 * model);\n    let _e15 = view_proj_uniforms.projection;\n    return (_e15 * model_view);\n}\n";
        }
        pub mod fs_main {
            pub const NAME: &'static str = "fs_main";
            ///The sourcecode for the shader, as a constant string, excluding any other entry points. This is useful when the `minify` feature is enabled for this crate, as it allows more aggressive minification to be performed with the knowledge of the specific entry point that will be used.
            pub const EXCLUSIVE_SOURCE: &'static str = "struct GlobalUniformsX_naga_oil_mod_XM5WG6YTBNQX {\n    time: f32,\n    screen_size: vec2<f32>,\n}\n\nstruct ViewProjectionUniformsX_naga_oil_mod_XM5WG6YTBNQX {\n    view: mat4x4<f32>,\n    projection: mat4x4<f32>,\n    camera_pos: vec3<f32>,\n    inverse_view: mat4x4<f32>,\n}\n\nstruct VertexInputX_naga_oil_mod_XNFXHA5LUOMX {\n    @location(0) position: vec4<f32>,\n    @location(1) tex_coords: vec2<f32>,\n    @location(2) normal: vec3<f32>,\n}\n\nstruct InstanceInputX_naga_oil_mod_XNFXHA5LUOMX {\n    @location(3) uv_scale: vec2<f32>,\n    @location(4) uv_offset: vec2<f32>,\n    @location(5) tint: vec4<f32>,\n    @location(6) model_1x: vec4<f32>,\n    @location(7) model_2x: vec4<f32>,\n    @location(8) model_3x: vec4<f32>,\n    @location(9) model_4x: vec4<f32>,\n    @location(10) normal_1x: vec4<f32>,\n    @location(11) normal_2x: vec4<f32>,\n    @location(12) normal_3x: vec4<f32>,\n    @location(13) normal_4x: vec4<f32>,\n}\n\n@group(0) @binding(0) \nvar t_diffuse: texture_2d<f32>;\n@group(0) @binding(1) \nvar s_diffuse: sampler;\n@group(1) @binding(0) \nvar<uniform> global_uniforms: GlobalUniformsX_naga_oil_mod_XM5WG6YTBNQX;\n@group(2) @binding(0) \nvar<uniform> view_proj_uniforms: ViewProjectionUniformsX_naga_oil_mod_XM5WG6YTBNQX;\n\n@fragment \nfn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {\n    return vec4<f32>(position.z, position.z, position.z, 1f);\n}\n";
        }
    }
    #[allow(unused)]
    ///Equivalent Rust definitions of the types defined in this module.
    pub mod types {}
    #[allow(unused)]
    use types::*;
    ///The sourcecode for the shader, as a constant string.
    pub const SOURCE: &'static str = "struct GlobalUniformsX_naga_oil_mod_XM5WG6YTBNQX {\n    time: f32,\n    screen_size: vec2<f32>,\n}\n\nstruct ViewProjectionUniformsX_naga_oil_mod_XM5WG6YTBNQX {\n    view: mat4x4<f32>,\n    projection: mat4x4<f32>,\n    camera_pos: vec3<f32>,\n    inverse_view: mat4x4<f32>,\n}\n\nstruct VertexInputX_naga_oil_mod_XNFXHA5LUOMX {\n    @location(0) position: vec4<f32>,\n    @location(1) tex_coords: vec2<f32>,\n    @location(2) normal: vec3<f32>,\n}\n\nstruct InstanceInputX_naga_oil_mod_XNFXHA5LUOMX {\n    @location(3) uv_scale: vec2<f32>,\n    @location(4) uv_offset: vec2<f32>,\n    @location(5) tint: vec4<f32>,\n    @location(6) model_1x: vec4<f32>,\n    @location(7) model_2x: vec4<f32>,\n    @location(8) model_3x: vec4<f32>,\n    @location(9) model_4x: vec4<f32>,\n    @location(10) normal_1x: vec4<f32>,\n    @location(11) normal_2x: vec4<f32>,\n    @location(12) normal_3x: vec4<f32>,\n    @location(13) normal_4x: vec4<f32>,\n}\n\n@group(0) @binding(0) \nvar t_diffuse: texture_2d<f32>;\n@group(0) @binding(1) \nvar s_diffuse: sampler;\n@group(1) @binding(0) \nvar<uniform> global_uniforms: GlobalUniformsX_naga_oil_mod_XM5WG6YTBNQX;\n@group(2) @binding(0) \nvar<uniform> view_proj_uniforms: ViewProjectionUniformsX_naga_oil_mod_XM5WG6YTBNQX;\n\n@vertex \nfn vs_main(vertex: VertexInputX_naga_oil_mod_XNFXHA5LUOMX, instance: InstanceInputX_naga_oil_mod_XNFXHA5LUOMX) -> @builtin(position) vec4<f32> {\n    let model_transform = mat4x4<f32>(instance.model_1x, instance.model_2x, instance.model_3x, instance.model_4x);\n    let model = (model_transform * vertex.position);\n    let _e11 = view_proj_uniforms.view;\n    let model_view = (_e11 * model);\n    let _e15 = view_proj_uniforms.projection;\n    return (_e15 * model_view);\n}\n\n@fragment \nfn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {\n    return vec4<f32>(position.z, position.z, position.z, 1f);\n}\n";
    ///Shader module descriptor.
    pub const DESCRIPTOR: ::wgpu::ShaderModuleDescriptor = ::wgpu::ShaderModuleDescriptor {
        label: Some("shadow_map"),
        source: ::wgpu::ShaderSource::Wgsl(::std::borrow::Cow::Borrowed(SOURCE)),
    };
}
pub mod text {
    #[allow(unused)]
    ///Information about the globals within the module, exposed as constants and functions.
    pub mod globals {
        #[allow(unused)]
        use super::*;
        ///Information about the `t_diffuse` global variable within this shader module.
        pub mod t_diffuse {
            #[allow(unused)]
            use super::*;
            pub const NAME: &'static str = "t_diffuse";
            pub const GROUP: u32 = 0u32;
            pub const BINDING: u32 = 0u32;
        }
        ///Information about the `s_diffuse` global variable within this shader module.
        pub mod s_diffuse {
            #[allow(unused)]
            use super::*;
            pub const NAME: &'static str = "s_diffuse";
            pub const GROUP: u32 = 0u32;
            pub const BINDING: u32 = 1u32;
        }
        ///Information about the `global_uniforms` global variable within this shader module.
        pub mod global_uniforms {
            #[allow(unused)]
            use super::*;
            pub const NAME: &'static str = "global_uniforms";
            pub const GROUP: u32 = 1u32;
            pub const BINDING: u32 = 0u32;
            pub fn layout() -> ::wgpu::BindGroupLayoutDescriptor<'static> {
                ::wgpu::BindGroupLayoutDescriptor {
                    label: Some("global_uniforms"),
                    entries: &[
                        ::wgpu::BindGroupLayoutEntry {
                            binding: 0u32,
                            visibility: ::wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: ::wgpu::BindingType::Buffer {
                                ty: ::wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                }
            }
        }
        ///Information about the `view_proj_uniforms` global variable within this shader module.
        pub mod view_proj_uniforms {
            #[allow(unused)]
            use super::*;
            pub const NAME: &'static str = "view_proj_uniforms";
            pub const GROUP: u32 = 2u32;
            pub const BINDING: u32 = 0u32;
            pub fn layout() -> ::wgpu::BindGroupLayoutDescriptor<'static> {
                ::wgpu::BindGroupLayoutDescriptor {
                    label: Some("view_proj_uniforms"),
                    entries: &[
                        ::wgpu::BindGroupLayoutEntry {
                            binding: 0u32,
                            visibility: ::wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: ::wgpu::BindingType::Buffer {
                                ty: ::wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                }
            }
        }
        ///Contains the following bindings: t_diffuse, s_diffuse
        pub mod group0 {
            #[allow(unused)]
            use super::*;
            pub const GROUP: u32 = 0u32;
            pub fn layout() -> ::wgpu::BindGroupLayoutDescriptor<'static> {
                ::wgpu::BindGroupLayoutDescriptor {
                    label: Some("group0"),
                    entries: &[
                        ::wgpu::BindGroupLayoutEntry {
                            binding: 0u32,
                            visibility: ::wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: ::wgpu::TextureViewDimension::D2,
                                sample_type: ::wgpu::TextureSampleType::Float {
                                    filterable: true,
                                },
                            },
                            count: None,
                        },
                        ::wgpu::BindGroupLayoutEntry {
                            binding: 1u32,
                            visibility: ::wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: ::wgpu::BindingType::Sampler(
                                ::wgpu::SamplerBindingType::Filtering,
                            ),
                            count: None,
                        },
                    ],
                }
            }
        }
    }
    #[allow(unused)]
    ///Information about the constants within the module, exposed as constants and functions.
    pub mod constants {
        #[allow(unused)]
        use super::*;
    }
    #[allow(unused)]
    ///Information about the entry points within the module, exposed as constants and functions.
    pub mod entry_points {
        #[allow(unused)]
        use super::*;
        pub mod vs_main {
            pub const NAME: &'static str = "vs_main";
            ///The sourcecode for the shader, as a constant string, excluding any other entry points. This is useful when the `minify` feature is enabled for this crate, as it allows more aggressive minification to be performed with the knowledge of the specific entry point that will be used.
            pub const EXCLUSIVE_SOURCE: &'static str = "struct GlobalUniforms {\n    time: f32,\n}\n\nstruct ViewProjectionUniforms {\n    view: mat4x4<f32>,\n    projection: mat4x4<f32>,\n}\n\nstruct VertexInput {\n    @location(0) position: vec4<f32>,\n    @location(1) tex_coords: vec2<f32>,\n}\n\nstruct InstanceInput {\n    @location(2) uv_scale: vec2<f32>,\n    @location(3) uv_offset: vec2<f32>,\n    @location(4) tint: vec4<f32>,\n    @location(5) model_1_: vec4<f32>,\n    @location(6) model_2_: vec4<f32>,\n    @location(7) model_3_: vec4<f32>,\n    @location(8) model_4_: vec4<f32>,\n}\n\nstruct VertexOutput {\n    @builtin(position) clip_position: vec4<f32>,\n    @location(0) tex_coords: vec2<f32>,\n    @location(1) screen_pos: vec2<f32>,\n    @location(2) tint_color: vec4<f32>,\n}\n\n@group(0) @binding(0) \nvar t_diffuse: texture_2d<f32>;\n@group(0) @binding(1) \nvar s_diffuse: sampler;\n@group(1) @binding(0) \nvar<uniform> global_uniforms: GlobalUniforms;\n@group(2) @binding(0) \nvar<uniform> view_proj_uniforms: ViewProjectionUniforms;\n\nfn median(r: f32, g: f32, b: f32) -> f32 {\n    return max(min(r, g), min(max(r, g), b));\n}\n\n@vertex \nfn vs_main(vertex: VertexInput, instance: InstanceInput) -> VertexOutput {\n    var out: VertexOutput;\n\n    let model_transform = mat4x4<f32>(instance.model_1_, instance.model_2_, instance.model_3_, instance.model_4_);\n    out.tex_coords = (instance.uv_offset + (instance.uv_scale * vertex.tex_coords));\n    let model = (model_transform * vertex.position);\n    let _e18 = view_proj_uniforms.view;\n    let model_view = (_e18 * model);\n    let _e23 = view_proj_uniforms.projection;\n    out.clip_position = (_e23 * model_view);\n    out.screen_pos = model_view.xy;\n    out.tint_color = instance.tint;\n    let _e29 = out;\n    return _e29;\n}\n";
        }
        pub mod fs_main {
            pub const NAME: &'static str = "fs_main";
            ///The sourcecode for the shader, as a constant string, excluding any other entry points. This is useful when the `minify` feature is enabled for this crate, as it allows more aggressive minification to be performed with the knowledge of the specific entry point that will be used.
            pub const EXCLUSIVE_SOURCE: &'static str = "struct GlobalUniforms {\n    time: f32,\n}\n\nstruct ViewProjectionUniforms {\n    view: mat4x4<f32>,\n    projection: mat4x4<f32>,\n}\n\nstruct VertexInput {\n    @location(0) position: vec4<f32>,\n    @location(1) tex_coords: vec2<f32>,\n}\n\nstruct InstanceInput {\n    @location(2) uv_scale: vec2<f32>,\n    @location(3) uv_offset: vec2<f32>,\n    @location(4) tint: vec4<f32>,\n    @location(5) model_1_: vec4<f32>,\n    @location(6) model_2_: vec4<f32>,\n    @location(7) model_3_: vec4<f32>,\n    @location(8) model_4_: vec4<f32>,\n}\n\nstruct VertexOutput {\n    @builtin(position) clip_position: vec4<f32>,\n    @location(0) tex_coords: vec2<f32>,\n    @location(1) screen_pos: vec2<f32>,\n    @location(2) tint_color: vec4<f32>,\n}\n\n@group(0) @binding(0) \nvar t_diffuse: texture_2d<f32>;\n@group(0) @binding(1) \nvar s_diffuse: sampler;\n@group(1) @binding(0) \nvar<uniform> global_uniforms: GlobalUniforms;\n@group(2) @binding(0) \nvar<uniform> view_proj_uniforms: ViewProjectionUniforms;\n\nfn median(r: f32, g: f32, b: f32) -> f32 {\n    return max(min(r, g), min(max(r, g), b));\n}\n\n@fragment \nfn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {\n    let msd = textureSample(t_diffuse, s_diffuse, in.tex_coords);\n    let _e8 = median(msd.x, msd.y, msd.z);\n    let _e9 = fwidth(_e8);\n    let w = (_e9 * 0.5f);\n    let opacity = smoothstep((0.5f - w), (0.5f + w), _e8);\n    return vec4<f32>(in.tint_color.x, in.tint_color.y, in.tint_color.z, (opacity * in.tint_color.w));\n}\n";
        }
    }
    #[allow(unused)]
    ///Equivalent Rust definitions of the types defined in this module.
    pub mod types {}
    #[allow(unused)]
    use types::*;
    ///The sourcecode for the shader, as a constant string.
    pub const SOURCE: &'static str = "struct GlobalUniforms {\n    time: f32,\n}\n\nstruct ViewProjectionUniforms {\n    view: mat4x4<f32>,\n    projection: mat4x4<f32>,\n}\n\nstruct VertexInput {\n    @location(0) position: vec4<f32>,\n    @location(1) tex_coords: vec2<f32>,\n}\n\nstruct InstanceInput {\n    @location(2) uv_scale: vec2<f32>,\n    @location(3) uv_offset: vec2<f32>,\n    @location(4) tint: vec4<f32>,\n    @location(5) model_1_: vec4<f32>,\n    @location(6) model_2_: vec4<f32>,\n    @location(7) model_3_: vec4<f32>,\n    @location(8) model_4_: vec4<f32>,\n}\n\nstruct VertexOutput {\n    @builtin(position) clip_position: vec4<f32>,\n    @location(0) tex_coords: vec2<f32>,\n    @location(1) screen_pos: vec2<f32>,\n    @location(2) tint_color: vec4<f32>,\n}\n\n@group(0) @binding(0) \nvar t_diffuse: texture_2d<f32>;\n@group(0) @binding(1) \nvar s_diffuse: sampler;\n@group(1) @binding(0) \nvar<uniform> global_uniforms: GlobalUniforms;\n@group(2) @binding(0) \nvar<uniform> view_proj_uniforms: ViewProjectionUniforms;\n\nfn median(r: f32, g: f32, b: f32) -> f32 {\n    return max(min(r, g), min(max(r, g), b));\n}\n\n@vertex \nfn vs_main(vertex: VertexInput, instance: InstanceInput) -> VertexOutput {\n    var out: VertexOutput;\n\n    let model_transform = mat4x4<f32>(instance.model_1_, instance.model_2_, instance.model_3_, instance.model_4_);\n    out.tex_coords = (instance.uv_offset + (instance.uv_scale * vertex.tex_coords));\n    let model = (model_transform * vertex.position);\n    let _e18 = view_proj_uniforms.view;\n    let model_view = (_e18 * model);\n    let _e23 = view_proj_uniforms.projection;\n    out.clip_position = (_e23 * model_view);\n    out.screen_pos = model_view.xy;\n    out.tint_color = instance.tint;\n    let _e29 = out;\n    return _e29;\n}\n\n@fragment \nfn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {\n    let msd = textureSample(t_diffuse, s_diffuse, in.tex_coords);\n    let _e8 = median(msd.x, msd.y, msd.z);\n    let _e9 = fwidth(_e8);\n    let w = (_e9 * 0.5f);\n    let opacity = smoothstep((0.5f - w), (0.5f + w), _e8);\n    return vec4<f32>(in.tint_color.x, in.tint_color.y, in.tint_color.z, (opacity * in.tint_color.w));\n}\n";
    ///Shader module descriptor.
    pub const DESCRIPTOR: ::wgpu::ShaderModuleDescriptor = ::wgpu::ShaderModuleDescriptor {
        label: Some("text"),
        source: ::wgpu::ShaderSource::Wgsl(::std::borrow::Cow::Borrowed(SOURCE)),
    };
}
pub mod flat {
    #[allow(unused)]
    ///Information about the globals within the module, exposed as constants and functions.
    pub mod globals {
        #[allow(unused)]
        use super::*;
        ///Information about the `t_diffuse` global variable within this shader module.
        pub mod t_diffuse {
            #[allow(unused)]
            use super::*;
            pub const NAME: &'static str = "t_diffuse";
            pub const GROUP: u32 = 0u32;
            pub const BINDING: u32 = 0u32;
        }
        ///Information about the `s_diffuse` global variable within this shader module.
        pub mod s_diffuse {
            #[allow(unused)]
            use super::*;
            pub const NAME: &'static str = "s_diffuse";
            pub const GROUP: u32 = 0u32;
            pub const BINDING: u32 = 1u32;
        }
        ///Information about the `global_uniforms` global variable within this shader module.
        pub mod global_uniforms {
            #[allow(unused)]
            use super::*;
            pub const NAME: &'static str = "global_uniforms";
            pub const GROUP: u32 = 1u32;
            pub const BINDING: u32 = 0u32;
            pub fn layout() -> ::wgpu::BindGroupLayoutDescriptor<'static> {
                ::wgpu::BindGroupLayoutDescriptor {
                    label: Some("global_uniforms"),
                    entries: &[
                        ::wgpu::BindGroupLayoutEntry {
                            binding: 0u32,
                            visibility: ::wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: ::wgpu::BindingType::Buffer {
                                ty: ::wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                }
            }
        }
        ///Information about the `view_proj_uniforms` global variable within this shader module.
        pub mod view_proj_uniforms {
            #[allow(unused)]
            use super::*;
            pub const NAME: &'static str = "view_proj_uniforms";
            pub const GROUP: u32 = 2u32;
            pub const BINDING: u32 = 0u32;
            pub fn layout() -> ::wgpu::BindGroupLayoutDescriptor<'static> {
                ::wgpu::BindGroupLayoutDescriptor {
                    label: Some("view_proj_uniforms"),
                    entries: &[
                        ::wgpu::BindGroupLayoutEntry {
                            binding: 0u32,
                            visibility: ::wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: ::wgpu::BindingType::Buffer {
                                ty: ::wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                }
            }
        }
        ///Contains the following bindings: t_diffuse, s_diffuse
        pub mod group0 {
            #[allow(unused)]
            use super::*;
            pub const GROUP: u32 = 0u32;
            pub fn layout() -> ::wgpu::BindGroupLayoutDescriptor<'static> {
                ::wgpu::BindGroupLayoutDescriptor {
                    label: Some("group0"),
                    entries: &[
                        ::wgpu::BindGroupLayoutEntry {
                            binding: 0u32,
                            visibility: ::wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: ::wgpu::TextureViewDimension::D2,
                                sample_type: ::wgpu::TextureSampleType::Float {
                                    filterable: true,
                                },
                            },
                            count: None,
                        },
                        ::wgpu::BindGroupLayoutEntry {
                            binding: 1u32,
                            visibility: ::wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: ::wgpu::BindingType::Sampler(
                                ::wgpu::SamplerBindingType::Filtering,
                            ),
                            count: None,
                        },
                    ],
                }
            }
        }
    }
    #[allow(unused)]
    ///Information about the constants within the module, exposed as constants and functions.
    pub mod constants {
        #[allow(unused)]
        use super::*;
    }
    #[allow(unused)]
    ///Information about the entry points within the module, exposed as constants and functions.
    pub mod entry_points {
        #[allow(unused)]
        use super::*;
        pub mod vs_main {
            pub const NAME: &'static str = "vs_main";
            ///The sourcecode for the shader, as a constant string, excluding any other entry points. This is useful when the `minify` feature is enabled for this crate, as it allows more aggressive minification to be performed with the knowledge of the specific entry point that will be used.
            pub const EXCLUSIVE_SOURCE: &'static str = "struct GlobalUniforms {\n    time: f32,\n}\n\nstruct ViewProjectionUniforms {\n    view: mat4x4<f32>,\n    projection: mat4x4<f32>,\n}\n\nstruct VertexInput {\n    @location(0) position: vec4<f32>,\n    @location(1) tex_coords: vec2<f32>,\n}\n\nstruct InstanceInput {\n    @location(2) uv_scale: vec2<f32>,\n    @location(3) uv_offset: vec2<f32>,\n    @location(4) tint: vec4<f32>,\n    @location(5) model_1_: vec4<f32>,\n    @location(6) model_2_: vec4<f32>,\n    @location(7) model_3_: vec4<f32>,\n    @location(8) model_4_: vec4<f32>,\n}\n\nstruct VertexOutput {\n    @builtin(position) clip_position: vec4<f32>,\n    @location(0) tex_coords: vec2<f32>,\n    @location(1) screen_pos: vec2<f32>,\n    @location(2) tint_color: vec4<f32>,\n}\n\n@group(0) @binding(0) \nvar t_diffuse: texture_2d<f32>;\n@group(0) @binding(1) \nvar s_diffuse: sampler;\n@group(1) @binding(0) \nvar<uniform> global_uniforms: GlobalUniforms;\n@group(2) @binding(0) \nvar<uniform> view_proj_uniforms: ViewProjectionUniforms;\n\n@vertex \nfn vs_main(vertex: VertexInput, instance: InstanceInput) -> VertexOutput {\n    var out: VertexOutput;\n\n    let model_transform = mat4x4<f32>(instance.model_1_, instance.model_2_, instance.model_3_, instance.model_4_);\n    out.tex_coords = (instance.uv_offset + (instance.uv_scale * vertex.tex_coords));\n    let model = (model_transform * vertex.position);\n    let _e18 = view_proj_uniforms.view;\n    let model_view = (_e18 * model);\n    let _e23 = view_proj_uniforms.projection;\n    out.clip_position = (_e23 * model_view);\n    out.screen_pos = model_view.xy;\n    out.tint_color = instance.tint;\n    let _e29 = out;\n    return _e29;\n}\n";
        }
        pub mod fs_main {
            pub const NAME: &'static str = "fs_main";
            ///The sourcecode for the shader, as a constant string, excluding any other entry points. This is useful when the `minify` feature is enabled for this crate, as it allows more aggressive minification to be performed with the knowledge of the specific entry point that will be used.
            pub const EXCLUSIVE_SOURCE: &'static str = "struct GlobalUniforms {\n    time: f32,\n}\n\nstruct ViewProjectionUniforms {\n    view: mat4x4<f32>,\n    projection: mat4x4<f32>,\n}\n\nstruct VertexInput {\n    @location(0) position: vec4<f32>,\n    @location(1) tex_coords: vec2<f32>,\n}\n\nstruct InstanceInput {\n    @location(2) uv_scale: vec2<f32>,\n    @location(3) uv_offset: vec2<f32>,\n    @location(4) tint: vec4<f32>,\n    @location(5) model_1_: vec4<f32>,\n    @location(6) model_2_: vec4<f32>,\n    @location(7) model_3_: vec4<f32>,\n    @location(8) model_4_: vec4<f32>,\n}\n\nstruct VertexOutput {\n    @builtin(position) clip_position: vec4<f32>,\n    @location(0) tex_coords: vec2<f32>,\n    @location(1) screen_pos: vec2<f32>,\n    @location(2) tint_color: vec4<f32>,\n}\n\n@group(0) @binding(0) \nvar t_diffuse: texture_2d<f32>;\n@group(0) @binding(1) \nvar s_diffuse: sampler;\n@group(1) @binding(0) \nvar<uniform> global_uniforms: GlobalUniforms;\n@group(2) @binding(0) \nvar<uniform> view_proj_uniforms: ViewProjectionUniforms;\n\n@fragment \nfn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {\n    let _e5 = textureSample(t_diffuse, s_diffuse, in.tex_coords);\n    return (in.tint_color * _e5);\n}\n";
        }
    }
    #[allow(unused)]
    ///Equivalent Rust definitions of the types defined in this module.
    pub mod types {}
    #[allow(unused)]
    use types::*;
    ///The sourcecode for the shader, as a constant string.
    pub const SOURCE: &'static str = "struct GlobalUniforms {\n    time: f32,\n}\n\nstruct ViewProjectionUniforms {\n    view: mat4x4<f32>,\n    projection: mat4x4<f32>,\n}\n\nstruct VertexInput {\n    @location(0) position: vec4<f32>,\n    @location(1) tex_coords: vec2<f32>,\n}\n\nstruct InstanceInput {\n    @location(2) uv_scale: vec2<f32>,\n    @location(3) uv_offset: vec2<f32>,\n    @location(4) tint: vec4<f32>,\n    @location(5) model_1_: vec4<f32>,\n    @location(6) model_2_: vec4<f32>,\n    @location(7) model_3_: vec4<f32>,\n    @location(8) model_4_: vec4<f32>,\n}\n\nstruct VertexOutput {\n    @builtin(position) clip_position: vec4<f32>,\n    @location(0) tex_coords: vec2<f32>,\n    @location(1) screen_pos: vec2<f32>,\n    @location(2) tint_color: vec4<f32>,\n}\n\n@group(0) @binding(0) \nvar t_diffuse: texture_2d<f32>;\n@group(0) @binding(1) \nvar s_diffuse: sampler;\n@group(1) @binding(0) \nvar<uniform> global_uniforms: GlobalUniforms;\n@group(2) @binding(0) \nvar<uniform> view_proj_uniforms: ViewProjectionUniforms;\n\n@vertex \nfn vs_main(vertex: VertexInput, instance: InstanceInput) -> VertexOutput {\n    var out: VertexOutput;\n\n    let model_transform = mat4x4<f32>(instance.model_1_, instance.model_2_, instance.model_3_, instance.model_4_);\n    out.tex_coords = (instance.uv_offset + (instance.uv_scale * vertex.tex_coords));\n    let model = (model_transform * vertex.position);\n    let _e18 = view_proj_uniforms.view;\n    let model_view = (_e18 * model);\n    let _e23 = view_proj_uniforms.projection;\n    out.clip_position = (_e23 * model_view);\n    out.screen_pos = model_view.xy;\n    out.tint_color = instance.tint;\n    let _e29 = out;\n    return _e29;\n}\n\n@fragment \nfn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {\n    let _e5 = textureSample(t_diffuse, s_diffuse, in.tex_coords);\n    return (in.tint_color * _e5);\n}\n";
    ///Shader module descriptor.
    pub const DESCRIPTOR: ::wgpu::ShaderModuleDescriptor = ::wgpu::ShaderModuleDescriptor {
        label: Some("flat"),
        source: ::wgpu::ShaderSource::Wgsl(::std::borrow::Cow::Borrowed(SOURCE)),
    };
}
pub mod ssao_blur {
    #[allow(unused)]
    ///Information about the globals within the module, exposed as constants and functions.
    pub mod globals {
        #[allow(unused)]
        use super::*;
        ///Information about the `t_diffuse` global variable within this shader module.
        pub mod t_diffuse {
            #[allow(unused)]
            use super::*;
            pub const NAME: &'static str = "t_diffuse";
            pub const GROUP: u32 = 0u32;
            pub const BINDING: u32 = 0u32;
        }
        ///Information about the `s_diffuse` global variable within this shader module.
        pub mod s_diffuse {
            #[allow(unused)]
            use super::*;
            pub const NAME: &'static str = "s_diffuse";
            pub const GROUP: u32 = 0u32;
            pub const BINDING: u32 = 1u32;
        }
        ///Information about the `global_uniforms` global variable within this shader module.
        pub mod global_uniforms {
            #[allow(unused)]
            use super::*;
            pub const NAME: &'static str = "global_uniforms";
            pub type Ty = super::super::super::global::types::GlobalUniforms;
            pub const GROUP: u32 = 1u32;
            pub const BINDING: u32 = 0u32;
            pub fn layout() -> ::wgpu::BindGroupLayoutDescriptor<'static> {
                ::wgpu::BindGroupLayoutDescriptor {
                    label: Some("global_uniforms"),
                    entries: &[
                        ::wgpu::BindGroupLayoutEntry {
                            binding: 0u32,
                            visibility: ::wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: ::wgpu::BindingType::Buffer {
                                ty: ::wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                }
            }
        }
        ///Information about the `view_proj_uniforms` global variable within this shader module.
        pub mod view_proj_uniforms {
            #[allow(unused)]
            use super::*;
            pub const NAME: &'static str = "view_proj_uniforms";
            pub type Ty = super::super::super::global::types::ViewProjectionUniforms;
            pub const GROUP: u32 = 2u32;
            pub const BINDING: u32 = 0u32;
            pub fn layout() -> ::wgpu::BindGroupLayoutDescriptor<'static> {
                ::wgpu::BindGroupLayoutDescriptor {
                    label: Some("view_proj_uniforms"),
                    entries: &[
                        ::wgpu::BindGroupLayoutEntry {
                            binding: 0u32,
                            visibility: ::wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: ::wgpu::BindingType::Buffer {
                                ty: ::wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                }
            }
        }
        ///Information about the `blur_settings` global variable within this shader module.
        pub mod blur_settings {
            #[allow(unused)]
            use super::*;
            pub const NAME: &'static str = "blur_settings";
            pub type Ty = BlurUniforms;
            pub const GROUP: u32 = 4u32;
            pub const BINDING: u32 = 0u32;
            pub fn layout() -> ::wgpu::BindGroupLayoutDescriptor<'static> {
                ::wgpu::BindGroupLayoutDescriptor {
                    label: Some("blur_settings"),
                    entries: &[
                        ::wgpu::BindGroupLayoutEntry {
                            binding: 0u32,
                            visibility: ::wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: ::wgpu::BindingType::Buffer {
                                ty: ::wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                }
            }
        }
        ///Information about the `depth_buffer` global variable within this shader module.
        pub mod depth_buffer {
            #[allow(unused)]
            use super::*;
            pub const NAME: &'static str = "depth_buffer";
            pub const GROUP: u32 = 3u32;
            pub const BINDING: u32 = 0u32;
        }
        ///Information about the `depth_buffer_sampler` global variable within this shader module.
        pub mod depth_buffer_sampler {
            #[allow(unused)]
            use super::*;
            pub const NAME: &'static str = "depth_buffer_sampler";
            pub const GROUP: u32 = 3u32;
            pub const BINDING: u32 = 1u32;
        }
        ///Contains the following bindings: t_diffuse, s_diffuse
        pub mod group0 {
            #[allow(unused)]
            use super::*;
            pub const GROUP: u32 = 0u32;
            pub fn layout() -> ::wgpu::BindGroupLayoutDescriptor<'static> {
                ::wgpu::BindGroupLayoutDescriptor {
                    label: Some("group0"),
                    entries: &[
                        ::wgpu::BindGroupLayoutEntry {
                            binding: 0u32,
                            visibility: ::wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: ::wgpu::TextureViewDimension::D2,
                                sample_type: ::wgpu::TextureSampleType::Float {
                                    filterable: true,
                                },
                            },
                            count: None,
                        },
                        ::wgpu::BindGroupLayoutEntry {
                            binding: 1u32,
                            visibility: ::wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: ::wgpu::BindingType::Sampler(
                                ::wgpu::SamplerBindingType::Filtering,
                            ),
                            count: None,
                        },
                    ],
                }
            }
        }
        ///Contains the following bindings: depth_buffer, depth_buffer_sampler
        pub mod group3 {
            #[allow(unused)]
            use super::*;
            pub const GROUP: u32 = 3u32;
            pub fn layout() -> ::wgpu::BindGroupLayoutDescriptor<'static> {
                ::wgpu::BindGroupLayoutDescriptor {
                    label: Some("group3"),
                    entries: &[
                        ::wgpu::BindGroupLayoutEntry {
                            binding: 0u32,
                            visibility: ::wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: ::wgpu::TextureViewDimension::D2,
                                sample_type: ::wgpu::TextureSampleType::Depth,
                            },
                            count: None,
                        },
                        ::wgpu::BindGroupLayoutEntry {
                            binding: 1u32,
                            visibility: ::wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: ::wgpu::BindingType::Sampler(
                                ::wgpu::SamplerBindingType::Filtering,
                            ),
                            count: None,
                        },
                    ],
                }
            }
        }
    }
    #[allow(unused)]
    ///Information about the constants within the module, exposed as constants and functions.
    pub mod constants {
        #[allow(unused)]
        use super::*;
    }
    #[allow(unused)]
    ///Information about the entry points within the module, exposed as constants and functions.
    pub mod entry_points {
        #[allow(unused)]
        use super::*;
        pub mod vs_main {
            pub const NAME: &'static str = "vs_main";
            ///The sourcecode for the shader, as a constant string, excluding any other entry points. This is useful when the `minify` feature is enabled for this crate, as it allows more aggressive minification to be performed with the knowledge of the specific entry point that will be used.
            pub const EXCLUSIVE_SOURCE: &'static str = "struct GlobalUniformsX_naga_oil_mod_XM5WG6YTBNQX {\n    time: f32,\n    screen_size: vec2<f32>,\n}\n\nstruct ViewProjectionUniformsX_naga_oil_mod_XM5WG6YTBNQX {\n    view: mat4x4<f32>,\n    projection: mat4x4<f32>,\n    camera_pos: vec3<f32>,\n    inverse_view: mat4x4<f32>,\n}\n\nstruct VertexInput {\n    @location(0) position: vec4<f32>,\n    @location(1) tex_coords: vec2<f32>,\n}\n\nstruct InstanceInput {\n    @location(2) uv_scale: vec2<f32>,\n    @location(3) uv_offset: vec2<f32>,\n    @location(4) tint: vec4<f32>,\n    @location(5) model_1_: vec4<f32>,\n    @location(6) model_2_: vec4<f32>,\n    @location(7) model_3_: vec4<f32>,\n    @location(8) model_4_: vec4<f32>,\n}\n\nstruct VertexOutput {\n    @builtin(position) clip_position: vec4<f32>,\n    @location(0) tex_coords: vec2<f32>,\n    @location(1) tint_color: vec4<f32>,\n}\n\nstruct BlurUniforms {\n    half_kernel_size: i32,\n    sharpness: f32,\n    step: vec2<f32>,\n}\n\n@group(0) @binding(0) \nvar t_diffuse: texture_2d<f32>;\n@group(0) @binding(1) \nvar s_diffuse: sampler;\n@group(1) @binding(0) \nvar<uniform> global_uniforms: GlobalUniformsX_naga_oil_mod_XM5WG6YTBNQX;\n@group(2) @binding(0) \nvar<uniform> view_proj_uniforms: ViewProjectionUniformsX_naga_oil_mod_XM5WG6YTBNQX;\n@group(4) @binding(0) \nvar<uniform> blur_settings: BlurUniforms;\n@group(3) @binding(0) \nvar depth_buffer: texture_depth_2d;\n@group(3) @binding(1) \nvar depth_buffer_sampler: sampler;\n\nfn blur_weight(radius: f32, center_depth: f32, sample_depth: f32) -> f32 {\n    let _e2 = blur_settings.half_kernel_size;\n    let blur_sigma = ((f32(_e2) + 1f) * 0.5f);\n    let blur_falloff = (1f / ((2f * blur_sigma) * blur_sigma));\n    let _e18 = blur_settings.sharpness;\n    let depth_diff = ((sample_depth - center_depth) * _e18);\n    let weight = exp2((((-(radius) * radius) * blur_falloff) - (depth_diff * depth_diff)));\n    return weight;\n}\n\n@vertex \nfn vs_main(vertex: VertexInput, instance: InstanceInput) -> VertexOutput {\n    var out: VertexOutput;\n    var model: vec4<f32>;\n\n    let model_transform = mat4x4<f32>(instance.model_1_, instance.model_2_, instance.model_3_, instance.model_4_);\n    out.tex_coords = (instance.uv_offset + (instance.uv_scale * vertex.tex_coords));\n    model = vertex.position;\n    let _e18 = model.x;\n    model.x = ((_e18 * 2f) - 1f);\n    let _e25 = model.y;\n    model.y = ((_e25 * 2f) - 1f);\n    let model_view = model;\n    out.clip_position = model_view;\n    out.tint_color = instance.tint;\n    let _e34 = out;\n    return _e34;\n}\n";
        }
        pub mod fs_main {
            pub const NAME: &'static str = "fs_main";
            ///The sourcecode for the shader, as a constant string, excluding any other entry points. This is useful when the `minify` feature is enabled for this crate, as it allows more aggressive minification to be performed with the knowledge of the specific entry point that will be used.
            pub const EXCLUSIVE_SOURCE: &'static str = "struct GlobalUniformsX_naga_oil_mod_XM5WG6YTBNQX {\n    time: f32,\n    screen_size: vec2<f32>,\n}\n\nstruct ViewProjectionUniformsX_naga_oil_mod_XM5WG6YTBNQX {\n    view: mat4x4<f32>,\n    projection: mat4x4<f32>,\n    camera_pos: vec3<f32>,\n    inverse_view: mat4x4<f32>,\n}\n\nstruct VertexInput {\n    @location(0) position: vec4<f32>,\n    @location(1) tex_coords: vec2<f32>,\n}\n\nstruct InstanceInput {\n    @location(2) uv_scale: vec2<f32>,\n    @location(3) uv_offset: vec2<f32>,\n    @location(4) tint: vec4<f32>,\n    @location(5) model_1_: vec4<f32>,\n    @location(6) model_2_: vec4<f32>,\n    @location(7) model_3_: vec4<f32>,\n    @location(8) model_4_: vec4<f32>,\n}\n\nstruct VertexOutput {\n    @builtin(position) clip_position: vec4<f32>,\n    @location(0) tex_coords: vec2<f32>,\n    @location(1) tint_color: vec4<f32>,\n}\n\nstruct BlurUniforms {\n    half_kernel_size: i32,\n    sharpness: f32,\n    step: vec2<f32>,\n}\n\n@group(0) @binding(0) \nvar t_diffuse: texture_2d<f32>;\n@group(0) @binding(1) \nvar s_diffuse: sampler;\n@group(1) @binding(0) \nvar<uniform> global_uniforms: GlobalUniformsX_naga_oil_mod_XM5WG6YTBNQX;\n@group(2) @binding(0) \nvar<uniform> view_proj_uniforms: ViewProjectionUniformsX_naga_oil_mod_XM5WG6YTBNQX;\n@group(4) @binding(0) \nvar<uniform> blur_settings: BlurUniforms;\n@group(3) @binding(0) \nvar depth_buffer: texture_depth_2d;\n@group(3) @binding(1) \nvar depth_buffer_sampler: sampler;\n\nfn blur_weight(radius: f32, center_depth_1: f32, sample_depth: f32) -> f32 {\n    let _e2 = blur_settings.half_kernel_size;\n    let blur_sigma = ((f32(_e2) + 1f) * 0.5f);\n    let blur_falloff = (1f / ((2f * blur_sigma) * blur_sigma));\n    let _e18 = blur_settings.sharpness;\n    let depth_diff = ((sample_depth - center_depth_1) * _e18);\n    let weight_1 = exp2((((-(radius) * radius) * blur_falloff) - (depth_diff * depth_diff)));\n    return weight_1;\n}\n\n@fragment \nfn fs_main(in: VertexOutput) -> @location(0) f32 {\n    var result: f32;\n    var center_depth: f32;\n    var weight: f32 = 1f;\n    var i: i32 = 1i;\n    var i_1: i32 = 1i;\n\n    let _e7 = textureDimensions(t_diffuse, 0i);\n    let texelSize = (vec2<f32>(1f, 1f) / vec2<f32>(_e7));\n    let _e14 = textureSample(t_diffuse, s_diffuse, in.tex_coords);\n    result = _e14.x;\n    let _e20 = textureSample(depth_buffer, depth_buffer_sampler, in.tex_coords);\n    center_depth = _e20;\n    loop {\n        let _e23 = i;\n        let _e26 = blur_settings.half_kernel_size;\n        if (_e23 <= _e26) {\n        } else {\n            break;\n        }\n        {\n            let _e28 = i;\n            let r = f32(_e28);\n            let _e33 = blur_settings.step;\n            let uv = (in.tex_coords + (r * _e33));\n            let _e38 = textureSample(t_diffuse, s_diffuse, uv);\n            let sample_color = _e38.x;\n            let sample_depth_1 = textureSample(depth_buffer, depth_buffer_sampler, in.tex_coords);\n            let _e44 = center_depth;\n            let _e45 = blur_weight(r, _e44, sample_depth_1);\n            let _e47 = weight;\n            weight = (_e47 + _e45);\n            let _e50 = result;\n            result = (_e50 + (sample_color * _e45));\n        }\n        continuing {\n            let _e53 = i;\n            i = (_e53 + 1i);\n        }\n    }\n    loop {\n        let _e56 = i_1;\n        let _e59 = blur_settings.half_kernel_size;\n        if (_e56 <= _e59) {\n        } else {\n            break;\n        }\n        {\n            let _e61 = i_1;\n            let r_1 = f32(_e61);\n            let _e66 = blur_settings.step;\n            let uv_1 = (in.tex_coords - (r_1 * _e66));\n            let _e71 = textureSample(t_diffuse, s_diffuse, uv_1);\n            let sample_color_1 = _e71.x;\n            let sample_depth_2 = textureSample(depth_buffer, depth_buffer_sampler, in.tex_coords);\n            let _e77 = center_depth;\n            let _e78 = blur_weight(r_1, _e77, sample_depth_2);\n            let _e79 = weight;\n            weight = (_e79 + _e78);\n            let _e82 = result;\n            result = (_e82 + (sample_color_1 * _e78));\n        }\n        continuing {\n            let _e85 = i_1;\n            i_1 = (_e85 + 1i);\n        }\n    }\n    let _e87 = weight;\n    let _e88 = result;\n    result = (_e88 / _e87);\n    let _e90 = result;\n    return _e90;\n}\n";
        }
    }
    #[allow(unused)]
    ///Equivalent Rust definitions of the types defined in this module.
    pub mod types {
        #[allow(unused, non_camel_case_types)]
        #[repr(C)]
        #[derive(Debug, PartialEq, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
        pub struct BlurUniforms {
            pub half_kernel_size: i32,
            pub sharpness: f32,
            pub step: glam::f32::Vec2,
        }
    }
    #[allow(unused)]
    use types::*;
    ///The sourcecode for the shader, as a constant string.
    pub const SOURCE: &'static str = "struct GlobalUniformsX_naga_oil_mod_XM5WG6YTBNQX {\n    time: f32,\n    screen_size: vec2<f32>,\n}\n\nstruct ViewProjectionUniformsX_naga_oil_mod_XM5WG6YTBNQX {\n    view: mat4x4<f32>,\n    projection: mat4x4<f32>,\n    camera_pos: vec3<f32>,\n    inverse_view: mat4x4<f32>,\n}\n\nstruct VertexInput {\n    @location(0) position: vec4<f32>,\n    @location(1) tex_coords: vec2<f32>,\n}\n\nstruct InstanceInput {\n    @location(2) uv_scale: vec2<f32>,\n    @location(3) uv_offset: vec2<f32>,\n    @location(4) tint: vec4<f32>,\n    @location(5) model_1_: vec4<f32>,\n    @location(6) model_2_: vec4<f32>,\n    @location(7) model_3_: vec4<f32>,\n    @location(8) model_4_: vec4<f32>,\n}\n\nstruct VertexOutput {\n    @builtin(position) clip_position: vec4<f32>,\n    @location(0) tex_coords: vec2<f32>,\n    @location(1) tint_color: vec4<f32>,\n}\n\nstruct BlurUniforms {\n    half_kernel_size: i32,\n    sharpness: f32,\n    step: vec2<f32>,\n}\n\n@group(0) @binding(0) \nvar t_diffuse: texture_2d<f32>;\n@group(0) @binding(1) \nvar s_diffuse: sampler;\n@group(1) @binding(0) \nvar<uniform> global_uniforms: GlobalUniformsX_naga_oil_mod_XM5WG6YTBNQX;\n@group(2) @binding(0) \nvar<uniform> view_proj_uniforms: ViewProjectionUniformsX_naga_oil_mod_XM5WG6YTBNQX;\n@group(4) @binding(0) \nvar<uniform> blur_settings: BlurUniforms;\n@group(3) @binding(0) \nvar depth_buffer: texture_depth_2d;\n@group(3) @binding(1) \nvar depth_buffer_sampler: sampler;\n\nfn blur_weight(radius: f32, center_depth_1: f32, sample_depth: f32) -> f32 {\n    let _e2 = blur_settings.half_kernel_size;\n    let blur_sigma = ((f32(_e2) + 1f) * 0.5f);\n    let blur_falloff = (1f / ((2f * blur_sigma) * blur_sigma));\n    let _e18 = blur_settings.sharpness;\n    let depth_diff = ((sample_depth - center_depth_1) * _e18);\n    let weight_1 = exp2((((-(radius) * radius) * blur_falloff) - (depth_diff * depth_diff)));\n    return weight_1;\n}\n\n@vertex \nfn vs_main(vertex: VertexInput, instance: InstanceInput) -> VertexOutput {\n    var out: VertexOutput;\n    var model: vec4<f32>;\n\n    let model_transform = mat4x4<f32>(instance.model_1_, instance.model_2_, instance.model_3_, instance.model_4_);\n    out.tex_coords = (instance.uv_offset + (instance.uv_scale * vertex.tex_coords));\n    model = vertex.position;\n    let _e18 = model.x;\n    model.x = ((_e18 * 2f) - 1f);\n    let _e25 = model.y;\n    model.y = ((_e25 * 2f) - 1f);\n    let model_view = model;\n    out.clip_position = model_view;\n    out.tint_color = instance.tint;\n    let _e34 = out;\n    return _e34;\n}\n\n@fragment \nfn fs_main(in: VertexOutput) -> @location(0) f32 {\n    var result: f32;\n    var center_depth: f32;\n    var weight: f32 = 1f;\n    var i: i32 = 1i;\n    var i_1: i32 = 1i;\n\n    let _e7 = textureDimensions(t_diffuse, 0i);\n    let texelSize = (vec2<f32>(1f, 1f) / vec2<f32>(_e7));\n    let _e14 = textureSample(t_diffuse, s_diffuse, in.tex_coords);\n    result = _e14.x;\n    let _e20 = textureSample(depth_buffer, depth_buffer_sampler, in.tex_coords);\n    center_depth = _e20;\n    loop {\n        let _e23 = i;\n        let _e26 = blur_settings.half_kernel_size;\n        if (_e23 <= _e26) {\n        } else {\n            break;\n        }\n        {\n            let _e28 = i;\n            let r = f32(_e28);\n            let _e33 = blur_settings.step;\n            let uv = (in.tex_coords + (r * _e33));\n            let _e38 = textureSample(t_diffuse, s_diffuse, uv);\n            let sample_color = _e38.x;\n            let sample_depth_1 = textureSample(depth_buffer, depth_buffer_sampler, in.tex_coords);\n            let _e44 = center_depth;\n            let _e45 = blur_weight(r, _e44, sample_depth_1);\n            let _e47 = weight;\n            weight = (_e47 + _e45);\n            let _e50 = result;\n            result = (_e50 + (sample_color * _e45));\n        }\n        continuing {\n            let _e53 = i;\n            i = (_e53 + 1i);\n        }\n    }\n    loop {\n        let _e56 = i_1;\n        let _e59 = blur_settings.half_kernel_size;\n        if (_e56 <= _e59) {\n        } else {\n            break;\n        }\n        {\n            let _e61 = i_1;\n            let r_1 = f32(_e61);\n            let _e66 = blur_settings.step;\n            let uv_1 = (in.tex_coords - (r_1 * _e66));\n            let _e71 = textureSample(t_diffuse, s_diffuse, uv_1);\n            let sample_color_1 = _e71.x;\n            let sample_depth_2 = textureSample(depth_buffer, depth_buffer_sampler, in.tex_coords);\n            let _e77 = center_depth;\n            let _e78 = blur_weight(r_1, _e77, sample_depth_2);\n            let _e79 = weight;\n            weight = (_e79 + _e78);\n            let _e82 = result;\n            result = (_e82 + (sample_color_1 * _e78));\n        }\n        continuing {\n            let _e85 = i_1;\n            i_1 = (_e85 + 1i);\n        }\n    }\n    let _e87 = weight;\n    let _e88 = result;\n    result = (_e88 / _e87);\n    let _e90 = result;\n    return _e90;\n}\n";
    ///Shader module descriptor.
    pub const DESCRIPTOR: ::wgpu::ShaderModuleDescriptor = ::wgpu::ShaderModuleDescriptor {
        label: Some("ssao_blur"),
        source: ::wgpu::ShaderSource::Wgsl(::std::borrow::Cow::Borrowed(SOURCE)),
    };
}
pub mod forward {
    #[allow(unused)]
    ///Information about the globals within the module, exposed as constants and functions.
    pub mod globals {
        #[allow(unused)]
        use super::*;
        ///Information about the `t_diffuse` global variable within this shader module.
        pub mod t_diffuse {
            #[allow(unused)]
            use super::*;
            pub const NAME: &'static str = "t_diffuse";
            pub const GROUP: u32 = 0u32;
            pub const BINDING: u32 = 0u32;
        }
        ///Information about the `s_diffuse` global variable within this shader module.
        pub mod s_diffuse {
            #[allow(unused)]
            use super::*;
            pub const NAME: &'static str = "s_diffuse";
            pub const GROUP: u32 = 0u32;
            pub const BINDING: u32 = 1u32;
        }
        ///Information about the `global_uniforms` global variable within this shader module.
        pub mod global_uniforms {
            #[allow(unused)]
            use super::*;
            pub const NAME: &'static str = "global_uniforms";
            pub type Ty = super::super::super::global::types::GlobalUniforms;
            pub const GROUP: u32 = 1u32;
            pub const BINDING: u32 = 0u32;
            pub fn layout() -> ::wgpu::BindGroupLayoutDescriptor<'static> {
                ::wgpu::BindGroupLayoutDescriptor {
                    label: Some("global_uniforms"),
                    entries: &[
                        ::wgpu::BindGroupLayoutEntry {
                            binding: 0u32,
                            visibility: ::wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: ::wgpu::BindingType::Buffer {
                                ty: ::wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                }
            }
        }
        ///Information about the `view_proj_uniforms` global variable within this shader module.
        pub mod view_proj_uniforms {
            #[allow(unused)]
            use super::*;
            pub const NAME: &'static str = "view_proj_uniforms";
            pub type Ty = super::super::super::global::types::ViewProjectionUniforms;
            pub const GROUP: u32 = 2u32;
            pub const BINDING: u32 = 0u32;
            pub fn layout() -> ::wgpu::BindGroupLayoutDescriptor<'static> {
                ::wgpu::BindGroupLayoutDescriptor {
                    label: Some("view_proj_uniforms"),
                    entries: &[
                        ::wgpu::BindGroupLayoutEntry {
                            binding: 0u32,
                            visibility: ::wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: ::wgpu::BindingType::Buffer {
                                ty: ::wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                }
            }
        }
        ///Information about the `lights` global variable within this shader module.
        pub mod lights {
            #[allow(unused)]
            use super::*;
            pub const NAME: &'static str = "lights";
            pub type Ty = LightsUniform;
            pub const GROUP: u32 = 3u32;
            pub const BINDING: u32 = 0u32;
        }
        ///Information about the `shadow_map` global variable within this shader module.
        pub mod shadow_map {
            #[allow(unused)]
            use super::*;
            pub const NAME: &'static str = "shadow_map";
            pub const GROUP: u32 = 3u32;
            pub const BINDING: u32 = 1u32;
        }
        ///Information about the `shadow_map_sampler` global variable within this shader module.
        pub mod shadow_map_sampler {
            #[allow(unused)]
            use super::*;
            pub const NAME: &'static str = "shadow_map_sampler";
            pub const GROUP: u32 = 3u32;
            pub const BINDING: u32 = 2u32;
        }
        ///Information about the `occlusion_map` global variable within this shader module.
        pub mod occlusion_map {
            #[allow(unused)]
            use super::*;
            pub const NAME: &'static str = "occlusion_map";
            pub const GROUP: u32 = 4u32;
            pub const BINDING: u32 = 0u32;
        }
        ///Information about the `occlusion_map_sampler` global variable within this shader module.
        pub mod occlusion_map_sampler {
            #[allow(unused)]
            use super::*;
            pub const NAME: &'static str = "occlusion_map_sampler";
            pub const GROUP: u32 = 4u32;
            pub const BINDING: u32 = 1u32;
        }
        ///Contains the following bindings: lights, shadow_map, shadow_map_sampler
        pub mod group3 {
            #[allow(unused)]
            use super::*;
            pub const GROUP: u32 = 3u32;
            pub fn layout() -> ::wgpu::BindGroupLayoutDescriptor<'static> {
                ::wgpu::BindGroupLayoutDescriptor {
                    label: Some("group3"),
                    entries: &[
                        ::wgpu::BindGroupLayoutEntry {
                            binding: 0u32,
                            visibility: ::wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: ::wgpu::BindingType::Buffer {
                                ty: ::wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        ::wgpu::BindGroupLayoutEntry {
                            binding: 1u32,
                            visibility: ::wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: ::wgpu::TextureViewDimension::D2Array,
                                sample_type: ::wgpu::TextureSampleType::Depth,
                            },
                            count: None,
                        },
                        ::wgpu::BindGroupLayoutEntry {
                            binding: 2u32,
                            visibility: ::wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: ::wgpu::BindingType::Sampler(
                                ::wgpu::SamplerBindingType::Comparison,
                            ),
                            count: None,
                        },
                    ],
                }
            }
        }
        ///Contains the following bindings: t_diffuse, s_diffuse
        pub mod group0 {
            #[allow(unused)]
            use super::*;
            pub const GROUP: u32 = 0u32;
            pub fn layout() -> ::wgpu::BindGroupLayoutDescriptor<'static> {
                ::wgpu::BindGroupLayoutDescriptor {
                    label: Some("group0"),
                    entries: &[
                        ::wgpu::BindGroupLayoutEntry {
                            binding: 0u32,
                            visibility: ::wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: ::wgpu::TextureViewDimension::D2,
                                sample_type: ::wgpu::TextureSampleType::Float {
                                    filterable: true,
                                },
                            },
                            count: None,
                        },
                        ::wgpu::BindGroupLayoutEntry {
                            binding: 1u32,
                            visibility: ::wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: ::wgpu::BindingType::Sampler(
                                ::wgpu::SamplerBindingType::Filtering,
                            ),
                            count: None,
                        },
                    ],
                }
            }
        }
        ///Contains the following bindings: occlusion_map, occlusion_map_sampler
        pub mod group4 {
            #[allow(unused)]
            use super::*;
            pub const GROUP: u32 = 4u32;
            pub fn layout() -> ::wgpu::BindGroupLayoutDescriptor<'static> {
                ::wgpu::BindGroupLayoutDescriptor {
                    label: Some("group4"),
                    entries: &[
                        ::wgpu::BindGroupLayoutEntry {
                            binding: 0u32,
                            visibility: ::wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: ::wgpu::TextureViewDimension::D2,
                                sample_type: ::wgpu::TextureSampleType::Float {
                                    filterable: true,
                                },
                            },
                            count: None,
                        },
                        ::wgpu::BindGroupLayoutEntry {
                            binding: 1u32,
                            visibility: ::wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: ::wgpu::BindingType::Sampler(
                                ::wgpu::SamplerBindingType::Filtering,
                            ),
                            count: None,
                        },
                    ],
                }
            }
        }
    }
    #[allow(unused)]
    ///Information about the constants within the module, exposed as constants and functions.
    pub mod constants {
        #[allow(unused)]
        use super::*;
    }
    #[allow(unused)]
    ///Information about the entry points within the module, exposed as constants and functions.
    pub mod entry_points {
        #[allow(unused)]
        use super::*;
        pub mod vs_main {
            pub const NAME: &'static str = "vs_main";
            ///The sourcecode for the shader, as a constant string, excluding any other entry points. This is useful when the `minify` feature is enabled for this crate, as it allows more aggressive minification to be performed with the knowledge of the specific entry point that will be used.
            pub const EXCLUSIVE_SOURCE: &'static str = "struct GlobalUniformsX_naga_oil_mod_XM5WG6YTBNQX {\n    time: f32,\n    screen_size: vec2<f32>,\n}\n\nstruct ViewProjectionUniformsX_naga_oil_mod_XM5WG6YTBNQX {\n    view: mat4x4<f32>,\n    projection: mat4x4<f32>,\n    camera_pos: vec3<f32>,\n    inverse_view: mat4x4<f32>,\n}\n\nstruct ModelVertexDataX_naga_oil_mod_XM5WG6YTBNQX {\n    @location(0) position: vec4<f32>,\n    @location(1) tex_coords: vec2<f32>,\n    @location(2) normal: vec3<f32>,\n}\n\nstruct InstanceInput {\n    @location(3) uv_scale: vec2<f32>,\n    @location(4) uv_offset: vec2<f32>,\n    @location(5) tint: vec4<f32>,\n    @location(6) model_1_: vec4<f32>,\n    @location(7) model_2_: vec4<f32>,\n    @location(8) model_3_: vec4<f32>,\n    @location(9) model_4_: vec4<f32>,\n    @location(10) normal_1_: vec4<f32>,\n    @location(11) normal_2_: vec4<f32>,\n    @location(12) normal_3_: vec4<f32>,\n    @location(13) normal_4_: vec4<f32>,\n}\n\nstruct VertexOutput {\n    @builtin(position) clip_position: vec4<f32>,\n    @location(0) tex_coords: vec2<f32>,\n    @location(1) view_pos: vec4<f32>,\n    @location(2) view_space_normal: vec3<f32>,\n    @location(3) tint_color: vec4<f32>,\n    @location(4) world_pos: vec4<f32>,\n}\n\nstruct Light {\n    direction: vec3<f32>,\n    kind: u32,\n    color: vec4<f32>,\n    view_proj: mat4x4<f32>,\n    position: vec3<f32>,\n    radius: f32,\n    reach: f32,\n}\n\nstruct LightsUniform {\n    items: array<Light, 8>,\n    count: u32,\n    shadow_bias_minimum: f32,\n    shadow_bias_factor: f32,\n    shadow_blur_half_kernel_size: i32,\n    ambient_color: vec4<f32>,\n}\n\n@group(0) @binding(0) \nvar t_diffuse: texture_2d<f32>;\n@group(0) @binding(1) \nvar s_diffuse: sampler;\n@group(1) @binding(0) \nvar<uniform> global_uniforms: GlobalUniformsX_naga_oil_mod_XM5WG6YTBNQX;\n@group(2) @binding(0) \nvar<uniform> view_proj_uniforms: ViewProjectionUniformsX_naga_oil_mod_XM5WG6YTBNQX;\n@group(3) @binding(0) \nvar<uniform> lights: LightsUniform;\n@group(3) @binding(1) \nvar shadow_map: texture_depth_2d_array;\n@group(3) @binding(2) \nvar shadow_map_sampler: sampler_comparison;\n@group(4) @binding(0) \nvar occlusion_map: texture_2d<f32>;\n@group(4) @binding(1) \nvar occlusion_map_sampler: sampler;\n\n@vertex \nfn vs_main(vertex: ModelVertexDataX_naga_oil_mod_XM5WG6YTBNQX, instance: InstanceInput) -> VertexOutput {\n    var out: VertexOutput;\n\n    let model_transform = mat4x4<f32>(instance.model_1_, instance.model_2_, instance.model_3_, instance.model_4_);\n    let normal_matrix = mat4x4<f32>(instance.normal_1_, instance.normal_2_, instance.normal_3_, instance.normal_4_);\n    out.tex_coords = (instance.uv_offset + (instance.uv_scale * vertex.tex_coords));\n    let _e21 = view_proj_uniforms.view;\n    let model_view = (_e21 * model_transform);\n    let model_view_pos = (model_view * vertex.position);\n    let _e28 = view_proj_uniforms.projection;\n    out.clip_position = (_e28 * model_view_pos);\n    out.view_space_normal = (normal_matrix * vec4<f32>(vertex.normal, 1f)).xyz;\n    out.view_pos = model_view_pos;\n    out.tint_color = instance.tint;\n    out.world_pos = (model_transform * vertex.position);\n    let _e42 = out;\n    return _e42;\n}\n";
        }
        pub mod fs_main {
            pub const NAME: &'static str = "fs_main";
            ///The sourcecode for the shader, as a constant string, excluding any other entry points. This is useful when the `minify` feature is enabled for this crate, as it allows more aggressive minification to be performed with the knowledge of the specific entry point that will be used.
            pub const EXCLUSIVE_SOURCE: &'static str = "struct GlobalUniformsX_naga_oil_mod_XM5WG6YTBNQX {\n    time: f32,\n    screen_size: vec2<f32>,\n}\n\nstruct ViewProjectionUniformsX_naga_oil_mod_XM5WG6YTBNQX {\n    view: mat4x4<f32>,\n    projection: mat4x4<f32>,\n    camera_pos: vec3<f32>,\n    inverse_view: mat4x4<f32>,\n}\n\nstruct ModelVertexDataX_naga_oil_mod_XM5WG6YTBNQX {\n    @location(0) position: vec4<f32>,\n    @location(1) tex_coords: vec2<f32>,\n    @location(2) normal: vec3<f32>,\n}\n\nstruct InstanceInput {\n    @location(3) uv_scale: vec2<f32>,\n    @location(4) uv_offset: vec2<f32>,\n    @location(5) tint: vec4<f32>,\n    @location(6) model_1_: vec4<f32>,\n    @location(7) model_2_: vec4<f32>,\n    @location(8) model_3_: vec4<f32>,\n    @location(9) model_4_: vec4<f32>,\n    @location(10) normal_1_: vec4<f32>,\n    @location(11) normal_2_: vec4<f32>,\n    @location(12) normal_3_: vec4<f32>,\n    @location(13) normal_4_: vec4<f32>,\n}\n\nstruct VertexOutput {\n    @builtin(position) clip_position: vec4<f32>,\n    @location(0) tex_coords: vec2<f32>,\n    @location(1) view_pos: vec4<f32>,\n    @location(2) view_space_normal: vec3<f32>,\n    @location(3) tint_color: vec4<f32>,\n    @location(4) world_pos: vec4<f32>,\n}\n\nstruct Light {\n    direction: vec3<f32>,\n    kind: u32,\n    color: vec4<f32>,\n    view_proj: mat4x4<f32>,\n    position: vec3<f32>,\n    radius: f32,\n    reach: f32,\n}\n\nstruct LightsUniform {\n    items: array<Light, 8>,\n    count: u32,\n    shadow_bias_minimum: f32,\n    shadow_bias_factor: f32,\n    shadow_blur_half_kernel_size: i32,\n    ambient_color: vec4<f32>,\n}\n\n@group(0) @binding(0) \nvar t_diffuse: texture_2d<f32>;\n@group(0) @binding(1) \nvar s_diffuse: sampler;\n@group(1) @binding(0) \nvar<uniform> global_uniforms: GlobalUniformsX_naga_oil_mod_XM5WG6YTBNQX;\n@group(2) @binding(0) \nvar<uniform> view_proj_uniforms: ViewProjectionUniformsX_naga_oil_mod_XM5WG6YTBNQX;\n@group(3) @binding(0) \nvar<uniform> lights: LightsUniform;\n@group(3) @binding(1) \nvar shadow_map: texture_depth_2d_array;\n@group(3) @binding(2) \nvar shadow_map_sampler: sampler_comparison;\n@group(4) @binding(0) \nvar occlusion_map: texture_2d<f32>;\n@group(4) @binding(1) \nvar occlusion_map_sampler: sampler;\n\n@fragment \nfn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {\n    var total_light: vec3<f32> = vec3<f32>(0f, 0f, 0f);\n    var i: u32 = 0u;\n    var visibility: f32;\n    var bias: f32;\n    var occlusion: f32;\n    var weight: f32;\n    var x: i32;\n    var y: i32;\n\n    let view_pos = in.view_pos;\n    let view_space_normal = in.view_space_normal;\n    let _e10 = textureSample(t_diffuse, s_diffuse, in.tex_coords);\n    let albedo_spec = (in.tint_color * _e10);\n    let _e13 = textureDimensions(shadow_map);\n    let texelSize = (1f / f32(_e13.x));\n    let _e20 = lights.ambient_color;\n    let MaterialAmbientColor = _e20.xyz;\n    let MaterialDiffuseColor = albedo_spec.xyz;\n    const MaterialSpecularColor = vec3<f32>(0.4f, 0.4f, 0.4f);\n    let n = normalize(in.view_space_normal);\n    let _e35 = global_uniforms.screen_size;\n    let _e37 = textureSample(occlusion_map, occlusion_map_sampler, (in.clip_position.xy / _e35));\n    let ao = _e37.x;\n    let _e42 = total_light;\n    total_light = (_e42 + ((ao * MaterialAmbientColor) * MaterialDiffuseColor));\n    loop {\n        let _e45 = i;\n        let _e48 = lights.count;\n        if (_e45 < _e48) {\n        } else {\n            break;\n        }\n        {\n            visibility = 1f;\n            let _e54 = i;\n            let _e57 = lights.items[_e54].kind;\n            if (_e57 == 1u) {\n                let _e64 = i;\n                let _e67 = lights.items[_e64].position;\n                let light_to_fragment = (in.world_pos.xyz - _e67);\n                let light_dist_sqr = dot(light_to_fragment, light_to_fragment);\n                let _e73 = i;\n                let _e76 = lights.items[_e73].direction;\n                let spot_factor = dot(normalize(light_to_fragment), _e76);\n                let _e80 = i;\n                let _e83 = lights.items[_e80].reach;\n                let reach_sqr = pow(_e83, 2f);\n                let _e88 = i;\n                let _e91 = lights.items[_e88].radius;\n                if ((spot_factor > _e91) && (light_dist_sqr < reach_sqr)) {\n                    let _e101 = i;\n                    let _e104 = lights.items[_e101].radius;\n                    visibility = (1f - (((1f - spot_factor) * 1f) / (1f - _e104)));\n                } else {\n                    visibility = 0f;\n                }\n            }\n            let _e113 = i;\n            let _e116 = lights.items[_e113].color;\n            let LightColor = _e116.xyz;\n            let _e120 = i;\n            let LightPower = lights.items[_e120].color.w;\n            let _e127 = view_proj_uniforms.view;\n            let _e130 = i;\n            let _e133 = lights.items[_e130].position;\n            let l = normalize((_e127 * vec4<f32>(_e133, 0f)).xyz);\n            let cosTheta = clamp(dot(n, l), 0f, 1f);\n            let E = normalize(-(in.view_pos.xyz));\n            let R = reflect(-(l), n);\n            let cosAlpha = clamp(dot(E, R), 0f, 1f);\n            bias = 0f;\n            let _e157 = i;\n            let _e160 = lights.items[_e157].view_proj;\n            let shadow_pos = (_e160 * in.world_pos);\n            const flip_correction = vec2<f32>(0.5f, -0.5f);\n            let proj_correction = (1f / shadow_pos.w);\n            let ShadowCoord = (((shadow_pos.xy * flip_correction) * proj_correction) + vec2<f32>(0.5f, 0.5f));\n            occlusion = 0f;\n            weight = 0f;\n            let _e182 = lights.shadow_blur_half_kernel_size;\n            x = -(_e182);\n            loop {\n                let _e185 = x;\n                let _e188 = lights.shadow_blur_half_kernel_size;\n                if (_e185 <= _e188) {\n                } else {\n                    break;\n                }\n                {\n                    let _e192 = lights.shadow_blur_half_kernel_size;\n                    y = -(_e192);\n                    loop {\n                        let _e195 = y;\n                        let _e198 = lights.shadow_blur_half_kernel_size;\n                        if (_e195 <= _e198) {\n                        } else {\n                            break;\n                        }\n                        {\n                            let _e203 = x;\n                            let _e205 = y;\n                            let _e210 = i;\n                            let _e212 = bias;\n                            let _e216 = textureSampleCompare(shadow_map, shadow_map_sampler, (ShadowCoord.xy + (vec2<f32>(f32(_e203), f32(_e205)) * texelSize)), _e210, ((shadow_pos.z - _e212) / shadow_pos.w));\n                            let _e219 = occlusion;\n                            occlusion = (_e219 + (1f - _e216));\n                            let _e222 = weight;\n                            weight = (_e222 + 1f);\n                        }\n                        continuing {\n                            let _e225 = y;\n                            y = (_e225 + 1i);\n                        }\n                    }\n                }\n                continuing {\n                    let _e228 = x;\n                    x = (_e228 + 1i);\n                }\n            }\n            let _e230 = weight;\n            let _e231 = occlusion;\n            occlusion = (_e231 / _e230);\n            let _e233 = visibility;\n            let _e234 = occlusion;\n            visibility = clamp((_e233 - _e234), 0f, 1f);\n            let _e239 = visibility;\n            let _e244 = visibility;\n            let _e252 = total_light;\n            total_light = (_e252 + (((((_e239 * MaterialDiffuseColor) * LightColor) * LightPower) * cosTheta) + ((((_e244 * MaterialSpecularColor) * LightColor) * LightPower) * pow(cosAlpha, 5f))));\n        }\n        continuing {\n            let _e255 = i;\n            i = (_e255 + 1u);\n        }\n    }\n    let _e257 = total_light;\n    return vec4<f32>(_e257.xyz, albedo_spec.w);\n}\n";
        }
    }
    #[allow(unused)]
    ///Equivalent Rust definitions of the types defined in this module.
    pub mod types {
        #[allow(unused, non_camel_case_types)]
        #[repr(C)]
        #[derive(Debug, PartialEq, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
        pub struct Light {
            pub direction: glam::f32::Vec3,
            pub kind: u32,
            pub color: glam::f32::Vec4,
            pub view_proj: glam::f32::Mat4,
            pub position: glam::f32::Vec3,
            pub radius: f32,
            pub reach: f32,
            pub _pad: [u8; 12u32 as usize],
        }
        #[allow(unused, non_camel_case_types)]
        #[repr(C)]
        #[derive(Debug, PartialEq, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
        pub struct LightsUniform {
            pub items: [Light; 8u32 as usize],
            pub count: u32,
            pub shadow_bias_minimum: f32,
            pub shadow_bias_factor: f32,
            pub shadow_blur_half_kernel_size: i32,
            pub ambient_color: glam::f32::Vec4,
        }
    }
    #[allow(unused)]
    use types::*;
    ///The sourcecode for the shader, as a constant string.
    pub const SOURCE: &'static str = "struct GlobalUniformsX_naga_oil_mod_XM5WG6YTBNQX {\n    time: f32,\n    screen_size: vec2<f32>,\n}\n\nstruct ViewProjectionUniformsX_naga_oil_mod_XM5WG6YTBNQX {\n    view: mat4x4<f32>,\n    projection: mat4x4<f32>,\n    camera_pos: vec3<f32>,\n    inverse_view: mat4x4<f32>,\n}\n\nstruct ModelVertexDataX_naga_oil_mod_XM5WG6YTBNQX {\n    @location(0) position: vec4<f32>,\n    @location(1) tex_coords: vec2<f32>,\n    @location(2) normal: vec3<f32>,\n}\n\nstruct InstanceInput {\n    @location(3) uv_scale: vec2<f32>,\n    @location(4) uv_offset: vec2<f32>,\n    @location(5) tint: vec4<f32>,\n    @location(6) model_1_: vec4<f32>,\n    @location(7) model_2_: vec4<f32>,\n    @location(8) model_3_: vec4<f32>,\n    @location(9) model_4_: vec4<f32>,\n    @location(10) normal_1_: vec4<f32>,\n    @location(11) normal_2_: vec4<f32>,\n    @location(12) normal_3_: vec4<f32>,\n    @location(13) normal_4_: vec4<f32>,\n}\n\nstruct VertexOutput {\n    @builtin(position) clip_position: vec4<f32>,\n    @location(0) tex_coords: vec2<f32>,\n    @location(1) view_pos: vec4<f32>,\n    @location(2) view_space_normal: vec3<f32>,\n    @location(3) tint_color: vec4<f32>,\n    @location(4) world_pos: vec4<f32>,\n}\n\nstruct Light {\n    direction: vec3<f32>,\n    kind: u32,\n    color: vec4<f32>,\n    view_proj: mat4x4<f32>,\n    position: vec3<f32>,\n    radius: f32,\n    reach: f32,\n}\n\nstruct LightsUniform {\n    items: array<Light, 8>,\n    count: u32,\n    shadow_bias_minimum: f32,\n    shadow_bias_factor: f32,\n    shadow_blur_half_kernel_size: i32,\n    ambient_color: vec4<f32>,\n}\n\n@group(0) @binding(0) \nvar t_diffuse: texture_2d<f32>;\n@group(0) @binding(1) \nvar s_diffuse: sampler;\n@group(1) @binding(0) \nvar<uniform> global_uniforms: GlobalUniformsX_naga_oil_mod_XM5WG6YTBNQX;\n@group(2) @binding(0) \nvar<uniform> view_proj_uniforms: ViewProjectionUniformsX_naga_oil_mod_XM5WG6YTBNQX;\n@group(3) @binding(0) \nvar<uniform> lights: LightsUniform;\n@group(3) @binding(1) \nvar shadow_map: texture_depth_2d_array;\n@group(3) @binding(2) \nvar shadow_map_sampler: sampler_comparison;\n@group(4) @binding(0) \nvar occlusion_map: texture_2d<f32>;\n@group(4) @binding(1) \nvar occlusion_map_sampler: sampler;\n\n@vertex \nfn vs_main(vertex: ModelVertexDataX_naga_oil_mod_XM5WG6YTBNQX, instance: InstanceInput) -> VertexOutput {\n    var out: VertexOutput;\n\n    let model_transform = mat4x4<f32>(instance.model_1_, instance.model_2_, instance.model_3_, instance.model_4_);\n    let normal_matrix = mat4x4<f32>(instance.normal_1_, instance.normal_2_, instance.normal_3_, instance.normal_4_);\n    out.tex_coords = (instance.uv_offset + (instance.uv_scale * vertex.tex_coords));\n    let _e21 = view_proj_uniforms.view;\n    let model_view = (_e21 * model_transform);\n    let model_view_pos = (model_view * vertex.position);\n    let _e28 = view_proj_uniforms.projection;\n    out.clip_position = (_e28 * model_view_pos);\n    out.view_space_normal = (normal_matrix * vec4<f32>(vertex.normal, 1f)).xyz;\n    out.view_pos = model_view_pos;\n    out.tint_color = instance.tint;\n    out.world_pos = (model_transform * vertex.position);\n    let _e42 = out;\n    return _e42;\n}\n\n@fragment \nfn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {\n    var total_light: vec3<f32> = vec3<f32>(0f, 0f, 0f);\n    var i: u32 = 0u;\n    var visibility: f32;\n    var bias: f32;\n    var occlusion: f32;\n    var weight: f32;\n    var x: i32;\n    var y: i32;\n\n    let view_pos = in.view_pos;\n    let view_space_normal = in.view_space_normal;\n    let _e10 = textureSample(t_diffuse, s_diffuse, in.tex_coords);\n    let albedo_spec = (in.tint_color * _e10);\n    let _e13 = textureDimensions(shadow_map);\n    let texelSize = (1f / f32(_e13.x));\n    let _e20 = lights.ambient_color;\n    let MaterialAmbientColor = _e20.xyz;\n    let MaterialDiffuseColor = albedo_spec.xyz;\n    const MaterialSpecularColor = vec3<f32>(0.4f, 0.4f, 0.4f);\n    let n = normalize(in.view_space_normal);\n    let _e35 = global_uniforms.screen_size;\n    let _e37 = textureSample(occlusion_map, occlusion_map_sampler, (in.clip_position.xy / _e35));\n    let ao = _e37.x;\n    let _e42 = total_light;\n    total_light = (_e42 + ((ao * MaterialAmbientColor) * MaterialDiffuseColor));\n    loop {\n        let _e45 = i;\n        let _e48 = lights.count;\n        if (_e45 < _e48) {\n        } else {\n            break;\n        }\n        {\n            visibility = 1f;\n            let _e54 = i;\n            let _e57 = lights.items[_e54].kind;\n            if (_e57 == 1u) {\n                let _e64 = i;\n                let _e67 = lights.items[_e64].position;\n                let light_to_fragment = (in.world_pos.xyz - _e67);\n                let light_dist_sqr = dot(light_to_fragment, light_to_fragment);\n                let _e73 = i;\n                let _e76 = lights.items[_e73].direction;\n                let spot_factor = dot(normalize(light_to_fragment), _e76);\n                let _e80 = i;\n                let _e83 = lights.items[_e80].reach;\n                let reach_sqr = pow(_e83, 2f);\n                let _e88 = i;\n                let _e91 = lights.items[_e88].radius;\n                if ((spot_factor > _e91) && (light_dist_sqr < reach_sqr)) {\n                    let _e101 = i;\n                    let _e104 = lights.items[_e101].radius;\n                    visibility = (1f - (((1f - spot_factor) * 1f) / (1f - _e104)));\n                } else {\n                    visibility = 0f;\n                }\n            }\n            let _e113 = i;\n            let _e116 = lights.items[_e113].color;\n            let LightColor = _e116.xyz;\n            let _e120 = i;\n            let LightPower = lights.items[_e120].color.w;\n            let _e127 = view_proj_uniforms.view;\n            let _e130 = i;\n            let _e133 = lights.items[_e130].position;\n            let l = normalize((_e127 * vec4<f32>(_e133, 0f)).xyz);\n            let cosTheta = clamp(dot(n, l), 0f, 1f);\n            let E = normalize(-(in.view_pos.xyz));\n            let R = reflect(-(l), n);\n            let cosAlpha = clamp(dot(E, R), 0f, 1f);\n            bias = 0f;\n            let _e157 = i;\n            let _e160 = lights.items[_e157].view_proj;\n            let shadow_pos = (_e160 * in.world_pos);\n            const flip_correction = vec2<f32>(0.5f, -0.5f);\n            let proj_correction = (1f / shadow_pos.w);\n            let ShadowCoord = (((shadow_pos.xy * flip_correction) * proj_correction) + vec2<f32>(0.5f, 0.5f));\n            occlusion = 0f;\n            weight = 0f;\n            let _e182 = lights.shadow_blur_half_kernel_size;\n            x = -(_e182);\n            loop {\n                let _e185 = x;\n                let _e188 = lights.shadow_blur_half_kernel_size;\n                if (_e185 <= _e188) {\n                } else {\n                    break;\n                }\n                {\n                    let _e192 = lights.shadow_blur_half_kernel_size;\n                    y = -(_e192);\n                    loop {\n                        let _e195 = y;\n                        let _e198 = lights.shadow_blur_half_kernel_size;\n                        if (_e195 <= _e198) {\n                        } else {\n                            break;\n                        }\n                        {\n                            let _e203 = x;\n                            let _e205 = y;\n                            let _e210 = i;\n                            let _e212 = bias;\n                            let _e216 = textureSampleCompare(shadow_map, shadow_map_sampler, (ShadowCoord.xy + (vec2<f32>(f32(_e203), f32(_e205)) * texelSize)), _e210, ((shadow_pos.z - _e212) / shadow_pos.w));\n                            let _e219 = occlusion;\n                            occlusion = (_e219 + (1f - _e216));\n                            let _e222 = weight;\n                            weight = (_e222 + 1f);\n                        }\n                        continuing {\n                            let _e225 = y;\n                            y = (_e225 + 1i);\n                        }\n                    }\n                }\n                continuing {\n                    let _e228 = x;\n                    x = (_e228 + 1i);\n                }\n            }\n            let _e230 = weight;\n            let _e231 = occlusion;\n            occlusion = (_e231 / _e230);\n            let _e233 = visibility;\n            let _e234 = occlusion;\n            visibility = clamp((_e233 - _e234), 0f, 1f);\n            let _e239 = visibility;\n            let _e244 = visibility;\n            let _e252 = total_light;\n            total_light = (_e252 + (((((_e239 * MaterialDiffuseColor) * LightColor) * LightPower) * cosTheta) + ((((_e244 * MaterialSpecularColor) * LightColor) * LightPower) * pow(cosAlpha, 5f))));\n        }\n        continuing {\n            let _e255 = i;\n            i = (_e255 + 1u);\n        }\n    }\n    let _e257 = total_light;\n    return vec4<f32>(_e257.xyz, albedo_spec.w);\n}\n";
    ///Shader module descriptor.
    pub const DESCRIPTOR: ::wgpu::ShaderModuleDescriptor = ::wgpu::ShaderModuleDescriptor {
        label: Some("forward"),
        source: ::wgpu::ShaderSource::Wgsl(::std::borrow::Cow::Borrowed(SOURCE)),
    };
}
