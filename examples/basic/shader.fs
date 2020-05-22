#version 450
layout(location = 0) in vec2 tex_coords;
layout(location = 0) out vec4 f_color;

layout(set = 1, binding = 0) uniform texture2D t_diffues;
layout(set = 1, binding = 1) uniform sampler s_diffuse;

void main() {
    f_color = texture(sampler2D(t_diffues, s_diffuse), tex_coords);
}