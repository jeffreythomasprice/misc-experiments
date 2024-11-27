attribute vec3 position_attribute;
attribute vec2 texture_coordinate_attribute;
attribute vec4 color_attribute;

varying vec2 texture_coordinate_varying;
varying vec4 color_varying;

uniform mat4 projection_matrix_uniform;
uniform mat4 model_view_matrix_uniform;

void main() {
	gl_Position = projection_matrix_uniform * model_view_matrix_uniform * vec4(position_attribute, 1);
	texture_coordinate_varying = texture_coordinate_attribute;
    color_varying = color_attribute;
}
