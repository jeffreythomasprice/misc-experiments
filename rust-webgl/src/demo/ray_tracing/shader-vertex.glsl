attribute vec2 position_attribute;
attribute vec3 ray_origin_attribute;
attribute vec3 ray_delta_attribute;

varying vec3 ray_origin_varying;
varying vec3 ray_delta_varying;

void main() {
	gl_Position = vec4(position_attribute, 0, 1);
    ray_origin_varying = ray_origin_attribute;
	ray_delta_varying = ray_delta_attribute;
}
