precision mediump float;

varying vec3 ray_origin_varying;
varying vec3 ray_delta_varying;

float ray_sphere_intersection(vec3 ray_origin, vec3 ray_delta, vec3 sphere_origin, float sphere_radius) {
    // https://gist.github.com/wwwtyro/beecc31d65d1004f5a9d
    float a = dot(ray_delta, ray_delta);
    vec3 s0_r0 = ray_origin - sphere_origin;
    float b = 2.0 * dot(ray_delta, s0_r0);
    float c = dot(s0_r0, s0_r0) - (sphere_radius * sphere_radius);
    if (b*b - 4.0*a*c < 0.0) {
        return -1.0;
    }
    return (-b - sqrt((b*b) - 4.0*a*c))/(2.0*a);
}

void main() {
    if (ray_sphere_intersection(ray_origin_varying, ray_delta_varying, vec3(0, 0, 0), 1.0) >= 0.0) {
        gl_FragColor = vec4(1, 1, 1, 1);
    } else {
        gl_FragColor = vec4(0.5, 0.5, 0.5, 1);
    }
}