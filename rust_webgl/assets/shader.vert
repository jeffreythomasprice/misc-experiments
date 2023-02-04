attribute vec2 positionAttribute;

void main() {
	gl_Position = vec4(positionAttribute, 0.0, 1.0);
}