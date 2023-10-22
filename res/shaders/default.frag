#version 300 es
precision highp float;

in vec2 texCoord;
in vec2 screenPos;

uniform sampler2D tex;
uniform float time;

out vec4 fragmentColor;

void main() {
   // fragmentColor = tintColor * texture(tex, texCoord);
   fragmentColor = texture(tex, texCoord);
}