#version 410 core

varying vec4 varying_color;

out vec4 out_color;

void main() {
	out_color = varying_color;
}
