#version 410 core

uniform mat4 uniform_projection_matrix;
uniform mat4 uniform_modelview_matrix;

in vec2 in_position;
in vec4 in_color;

varying vec4 varying_color;

void main() {
	gl_Position = uniform_projection_matrix * uniform_modelview_matrix * vec4(in_position, 0.0, 1.0);
	varying_color = in_color;
}
