use miniquad::*;

pub const VERTEX: &str = r#"#version 300 es
    precision highp float;

    // per vertex
    in vec4 position;
    in vec2 uv;
    // per instance
    in vec2 uv_scale;
    in vec2 uv_offset;
    in vec4 tint;
    in mat4 model;

    uniform mat4 view;
    uniform mat4 projection;

    out vec2 texCoord;
    out vec2 screenPos;
    out vec4 tintColor;

    void main() {
       texCoord = uv_offset + uv_scale * uv;
       vec4 pos = view * model * position;
       screenPos = pos.xy;
       tintColor = tint / vec4(255, 255, 255, 255);
       gl_Position = projection * pos;
    }
    "#;

pub const FRAGMENT: &str = r#"#version 300 es
    precision highp float;

    in vec2 texCoord;
    in vec2 screenPos;
    in vec4 tintColor;

    uniform sampler2D tex;

    out vec4 fragmentColor;

    void main() {
       fragmentColor = tintColor * texture(tex, texCoord);
    }
    "#;

pub fn meta() -> ShaderMeta {
    ShaderMeta {
        images: vec!["tex".into()],
        uniforms: UniformBlockLayout {
            uniforms: vec![
                UniformDesc::new("view", UniformType::Mat4),
                UniformDesc::new("projection", UniformType::Mat4),
            ],
        },
    }
}

#[repr(C)]
pub struct Uniforms {
    pub view: glam::Mat4,
    pub projection: glam::Mat4,
}
// pub const VERTEX: &str = r#"#version 100
// attribute vec4 pos;
// attribute vec4 color0;

// varying lowp vec4 color;

// uniform mat4 mvp;

// void main() {
//     gl_Position = mvp * pos;
//     color = color0;
// }
// "#;

// pub const FRAGMENT: &str = r#"#version 100

// varying lowp vec4 color;

// void main() {
//     gl_FragColor = color;
// }
// "#;

// pub fn meta() -> ShaderMeta {
//     ShaderMeta {
//         images: vec![],
//         uniforms: UniformBlockLayout {
//             uniforms: vec![UniformDesc::new("mvp", UniformType::Mat4)],
//         },
//     }
// }

// #[repr(C)]
// pub struct Uniforms {
//     pub mvp: glam::Mat4,
// }
