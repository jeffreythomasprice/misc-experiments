attribute vec2 position_attribute;
attribute vec4 color_attribute;

varying vec4 color_varying;

//uniform mat4 projection_matrix;
//uniform mat4 model_view_matrix;

void main() {
	gl_Position = vec4(position_attribute, 0, 1);
    //gl_Position = projection_matrix * model_view_matrix * vec4(position_attribute, 1);
    color_varying = color_attribute;
}
