#version 450

layout(set = 0, binding = 0) uniform Projection {
    mat4 projection;
};

layout(set = 0, binding = 1) uniform View {
    mat4 view;
};

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 diffuse;

layout(location = 0) out vec3 out_diffuse;

void main() {
    // move vertex to world-space
    mat4 view_model = view;
    vec4 world_position = view * vec4(position, 1.0);

    // TODO: inverse transpose multiply normal to make world_normal

    // finally, move vertex to screen-space
    gl_Position = projection * world_position;

    // Send rest to frag shader
    out_diffuse = diffuse;
}