 #version 450

layout(set = 0, binding = 0) uniform sampler2D uniform_sampler;

layout(location = 0) in vec2 frag_texture_coordinate;
layout(location = 1) in vec4 frag_color;

layout(location = 0) out vec4 out_color;

void main() {
	out_color = texture(uniform_sampler, frag_texture_coordinate)*frag_color;
}