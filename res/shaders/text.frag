#version 300 es
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