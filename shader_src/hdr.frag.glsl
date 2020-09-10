#version 450

layout(set = 0, binding = 0) uniform sampler sampler0;
layout(set = 0, binding = 1) uniform texture2D hdr_buffer;
layout(set = 0, binding = 2) uniform texture2D bloom_buffer;

layout(push_constant) uniform Exposure {
    float exposure;
};

layout(location = 0) in vec2 tex_coord;

layout(location = 0) out vec4 out_color;

const float GAMMA = 2.2;

void main() {
    vec3 hdr_color = texture(sampler2D(hdr_buffer, sampler0), tex_coord).rgb;
    vec3 bloom_color = texture(sampler2D(bloom_buffer, sampler0), tex_coord).rgb;
    hdr_color += bloom_color;
    vec3 result = pow(vec3(1.0) - exp(-hdr_color * exposure), vec3(1.0 / GAMMA));

    out_color = vec4(result, 1.0);
}