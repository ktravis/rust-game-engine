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
    in vec4 tintColor;

    uniform sampler2D tex;

    out vec4 fragmentColor;

    float median(float r, float g, float b) {
        return max(min(r, g), min(max(r, g), b));
    }

    void main() {
        vec4 msd = texture(tex, texCoord);
        float sd = median(msd.r, msd.g, msd.b);
        float w = fwidth(sd) * 0.5;
        float opacity = smoothstep(0.5 - w, 0.5 + w, sd);
        fragmentColor = vec4(tintColor.rgb, opacity);
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
