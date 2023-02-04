attribute vec2 positionAttribute;
attribute vec4 colorAttribute;

varying vec4 colorVarying;

void main() {
	gl_Position = vec4(positionAttribute, 0.0, 1.0);
	colorVarying = colorAttribute;
}