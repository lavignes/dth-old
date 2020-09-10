#version 450

layout(set = 0, binding = 0) uniform sampler sampler0;
layout(set = 0, binding = 1) uniform texture2D image;

layout(push_constant) uniform GaussianBlur {
    uint horizontal;
    float weight[5];
};

layout(location = 0) in vec2 tex_coord;

layout(location = 0) out vec4 out_color;

void main() {
    vec2 tex_offset = 1.0 / textureSize(sampler2D(image, sampler0), 0);
    vec3 result = texture(sampler2D(image, sampler0), tex_coord).rgb * weight[0];

    if (horizontal == 1) {
        for(int i = 1; i < 5; ++i) {
            result += texture(sampler2D(image, sampler0), tex_coord + vec2(tex_offset.x * i, 0.0)).rgb * weight[i];
            result += texture(sampler2D(image, sampler0), tex_coord - vec2(tex_offset.x * i, 0.0)).rgb * weight[i];
        }
    } else {
        for(int i = 1; i < 5; ++i) {
            result += texture(sampler2D(image, sampler0), tex_coord + vec2(0.0, tex_offset.y * i)).rgb * weight[i];
            result += texture(sampler2D(image, sampler0), tex_coord - vec2(0.0, tex_offset.y * i)).rgb * weight[i];
        }
    }
    out_color = vec4(result, 1.0);
}