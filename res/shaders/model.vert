#version 300 es
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