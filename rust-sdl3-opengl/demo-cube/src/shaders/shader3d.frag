#version 410 core

uniform sampler2D uniform_sampler;

varying vec2 varying_texture_coordinate;
varying vec4 varying_color;

out vec4 out_color;

void main() {
	out_color = texture(uniform_sampler, varying_texture_coordinate) * varying_color;
}
