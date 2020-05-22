#version 450

layout(location = 0) in vec3 pos;
layout(location = 1) in vec2 tex_coords;
layout(location = 0) out vec2 out_tex_coords;

layout(set = 0, binding = 0) 
uniform Camera {
    mat4 view;
    mat4 projection;
    vec3 view_pos;
};

layout(set = 0, binding = 1) 
uniform ModelInfo {
    mat4 model;
};

const mat4 CONVERSION = mat4(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0
);

void main() {
    out_tex_coords = tex_coords;
    vec3 fragment_position = vec3(model * vec4(pos, 1.0));
    gl_Position = CONVERSION * projection * view  * vec4(fragment_position, 1.0);
}