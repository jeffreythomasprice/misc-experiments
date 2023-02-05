attribute vec2 position_attribute;
attribute vec4 color_attribute;

uniform mat4 projection_matrix_uniform;
uniform mat4 modelview_matrix_uniform;

varying vec4 color_varying;

void main() {
	gl_Position = projection_matrix_uniform * modelview_matrix_uniform * vec4(position_attribute, 0.0, 1.0);
	color_varying = color_attribute;
}