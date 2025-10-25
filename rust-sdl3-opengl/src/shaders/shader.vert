#version 410 core

in vec2 in_position;
in vec4 in_color;

varying vec4 varying_color;

void main() {
	gl_Position = vec4(in_position, 0.0, 1.0);
	varying_color = in_color;
}
