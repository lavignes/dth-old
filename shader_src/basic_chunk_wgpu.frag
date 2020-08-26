#version 450

layout(location = 0) in vec3 diffuse;

layout(location = 0) out vec4 color;

void main() {
    color = vec4(diffuse, 1.0);
}