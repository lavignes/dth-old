#version 450

layout(set = 0, binding = 0) uniform Projection {
    mat4 projection;
};

layout(set = 0, binding = 1) uniform View {
    mat4 view;
    vec3 view_position;
};

layout(push_constant) uniform Model {
    mat4 model;
    mat3 inverse_normal;
    uint tex_indices;
};

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 tex_coord;
layout(location = 3) in vec4 color;

layout(location = 0) out vec3 out_position;
layout(location = 1) out vec3 out_normal;
layout(location = 2) out vec2 out_tex_coord;
layout(location = 3) out vec4 out_color;

void main() {
    vec4 world_position = model * vec4(position, 1.0);

    gl_Position = projection * view * world_position;

    out_position = world_position.xyz;
    out_normal = inverse_normal * normal;
    out_tex_coord = tex_coord;
    out_color = color;
}
