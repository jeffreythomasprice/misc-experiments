precision mediump float;

uniform sampler2D sampler_uniform;

varying vec2 texture_coordinate_varying;
varying vec4 color_varying;

void main() {
    gl_FragColor = texture2D(sampler_uniform, texture_coordinate_varying) * color_varying;
}