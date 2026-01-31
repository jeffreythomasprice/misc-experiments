#version 450

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 texture_coordinate;
layout(location = 2) in vec4 color;

layout(location = 0) out vec2 frag_texture_coordinate;
layout(location = 1) out vec4 frag_color;

void main() {
	gl_Position = vec4(position, 0.0, 1.0);
	frag_texture_coordinate = texture_coordinate;
	frag_color = color;
}