#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec2 tex_coord;

layout(location = 2) out vec2 out_tex_coord;

void main() {
    gl_Position = vec4(position, 1.0);

    out_tex_coord = tex_coord;
}