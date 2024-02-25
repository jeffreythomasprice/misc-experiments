attribute vec3 positionAttribute;
attribute vec4 colorAttribute;

varying vec4 colorVarying;

uniform mat4 projectionMatrix;
uniform mat4 modelViewMatrix;

void main() {
    gl_Position = projectionMatrix * modelViewMatrix * vec4(positionAttribute, 1);
    colorVarying = colorAttribute;
}
