attribute vec2 position_attribute;

void main() {
	gl_Position = vec4(position_attribute,0,1);
}