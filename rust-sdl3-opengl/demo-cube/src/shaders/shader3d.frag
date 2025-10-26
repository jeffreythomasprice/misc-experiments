#version 320 es

precision mediump float;

uniform sampler2D uniform_sampler;

in vec2 varying_texture_coordinate;
in vec4 varying_color;

out vec4 out_color;

void main() {
	out_color = texture(uniform_sampler, varying_texture_coordinate) * varying_color;
}
