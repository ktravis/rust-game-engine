#version 300 es
precision highp float;

in vec4 position;
in vec2 uv;

uniform mat4 view;
uniform mat4 projection;

out vec2 texCoord;
out vec2 screenPos;

void main() {
   texCoord = uv;
   vec4 pos = view * position;
   screenPos = pos.xy;
   gl_Position = projection * pos;
}