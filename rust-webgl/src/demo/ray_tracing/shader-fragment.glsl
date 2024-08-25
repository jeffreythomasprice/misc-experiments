precision mediump float;

varying vec3 ray_origin_varying;
varying vec3 ray_delta_varying;

const int CHUNK_SIZE = 16;
const float VOXEL_SIZE = 1.0;
const float WRONG_DIRECTION_OFFSET = 0.001;
const float MIN_VOXEL_SPACE_COORD = 0.0;
const float MAX_VOXEL_SPACE_COORD = float(CHUNK_SIZE) * VOXEL_SIZE;

struct Ray {
    vec3 origin;
    vec3 delta;
};

struct RayIntersection {
    bool success;
    float distance;
};

RayIntersection get_best_ray_intersection(RayIntersection a, RayIntersection b) {
    if (a.success) {
        if (b.success && b.distance < a.distance) {
            return b;
        }
        return a;
    }
    return b;
}

bool get_voxel(vec3 point) {
    if (
        point.x < MIN_VOXEL_SPACE_COORD
        || point.x >= MAX_VOXEL_SPACE_COORD
        || point.y <= MIN_VOXEL_SPACE_COORD
        || point.y >= MAX_VOXEL_SPACE_COORD
        || point.z <= MIN_VOXEL_SPACE_COORD
        || point.z >= MAX_VOXEL_SPACE_COORD
    ) {
        return false;
    }
    // TODO voxel data in texture? some kind of uniform?
    ivec3 ipoint = ivec3(
        int(point.x / VOXEL_SIZE),
        int(point.y / VOXEL_SIZE),
        int(point.z / VOXEL_SIZE)
    );
    // are we on the min or max of each axis
    bool x = ipoint.x == 0 || ipoint.x == CHUNK_SIZE-1;
    bool y = ipoint.y == 0 || ipoint.y == CHUNK_SIZE-1;
    bool z = ipoint.z == 0 || ipoint.z == CHUNK_SIZE-1;
    // edges and corners get filled in, interior and chunk faces don't
    return (x ? 1 : 0) + (y ? 1 : 0) + (z ? 1 : 0) >= 2;
}

vec3 get_ray_point(Ray ray, float distance) {
    return ray.origin + ray.delta * distance;
}

// TODO no?
// RayIntersection ray_x_axis_plane_intersection(Ray ray, float plane) {
//     RayIntersection result;
//     if (ray.origin.x == 0.0) {
//         result.success = false;
//     } else {
//         result.distance = (plane - ray.origin.x) / ray.delta.x;
//         result.success = result.distance >= 0.0;
//     }
//     return result;
// }

// TODO no?
// RayIntersection ray_y_axis_plane_intersection(Ray ray, float plane) {
//     RayIntersection result;
//     if (ray.origin.y == 0.0) {
//         result.success = false;
//     } else {
//         result.distance = (plane - ray.origin.y) / ray.delta.y;
//         result.success = result.distance >= 0.0;
//     }
//     return result;
// }

// TODO no?
// RayIntersection ray_z_axis_plane_intersection(Ray ray, float plane) {
//     RayIntersection result;
//     if (ray.origin.z == 0.0) {
//         result.success = false;
//     } else {
//         result.distance = (plane - ray.origin.z) / ray.delta.z;
//         result.success = result.distance >= 0.0;
//     }
//     return result;
// }

RayIntersection ray_x_axis_chunk_intersection(Ray ray) {
    RayIntersection result;
    result.success = false;
    if (ray.delta.x != 0.0) {
        // find the next plane in the direction we're going
        float plane;
        if (ray.origin.x < MIN_VOXEL_SPACE_COORD) {
            if (ray.delta.x < 0.0) {
                return result;
            }
            plane = MIN_VOXEL_SPACE_COORD;
        } else if (ray.origin.x >= MAX_VOXEL_SPACE_COORD) {
            if (ray.delta.x > 0.0) {
                return result;
            }
            plane = MAX_VOXEL_SPACE_COORD - WRONG_DIRECTION_OFFSET;
        } else {
            if (ray.delta.x < 0.0) {
                plane = floor(ray.origin.x) - WRONG_DIRECTION_OFFSET;
            } else {
                plane = floor(ray.origin.x) + VOXEL_SIZE;
            }
        }
        // start were the next plane to either side is
        result.distance = (plane - ray.origin.x) / ray.delta.x;
        // how much we'll have to move each step
        float distance_delta = abs(VOXEL_SIZE / ray.delta.x);
        // check check until we can't possibly be inside the chunk any more
        for (int i=0; i<CHUNK_SIZE; i++) {
            if (get_voxel(get_ray_point(ray, result.distance))) {
                result.success = true;
                return result;
            }
            result.distance+=distance_delta;
        }
    }
    return result;
}

RayIntersection ray_y_axis_chunk_intersection(Ray ray) {
    RayIntersection result;
    result.success = false;
    if (ray.delta.y != 0.0) {
        // find the next plane in the direction we're going
        float plane;
        if (ray.origin.y < MIN_VOXEL_SPACE_COORD) {
            if (ray.delta.y < 0.0) {
                return result;
            }
            plane = MIN_VOXEL_SPACE_COORD;
        } else if (ray.origin.y >= MAX_VOXEL_SPACE_COORD) {
            if (ray.delta.y > 0.0) {
                return result;
            }
            plane = MAX_VOXEL_SPACE_COORD - WRONG_DIRECTION_OFFSET;
        } else {
            if (ray.delta.y < 0.0) {
                plane = floor(ray.origin.y) - WRONG_DIRECTION_OFFSET;
            } else {
                plane = floor(ray.origin.y) + VOXEL_SIZE;
            }
        }
        // start were the next plane to either side is
        result.distance = (plane - ray.origin.y) / ray.delta.y;
        // how much we'll have to move each step
        float distance_delta = abs(VOXEL_SIZE / ray.delta.y);
        // check check until we can't possibly be inside the chunk any more
        for (int i=0; i<CHUNK_SIZE; i++) {
            if (get_voxel(get_ray_point(ray, result.distance))) {
                result.success = true;
                return result;
            }
            result.distance+=distance_delta;
        }
    }
    return result;
}

RayIntersection ray_z_axis_chunk_intersection(Ray ray) {
    RayIntersection result;
    result.success = false;
    if (ray.delta.z != 0.0) {
        // find the next plane in the direction we're going
        float plane;
        if (ray.origin.z < MIN_VOXEL_SPACE_COORD) {
            plane = MIN_VOXEL_SPACE_COORD;
            if (ray.delta.z < 0.0) {
                return result;
            }
        } else if (ray.origin.z >= MAX_VOXEL_SPACE_COORD) {
            plane = MAX_VOXEL_SPACE_COORD - WRONG_DIRECTION_OFFSET;
            if (ray.delta.z > 0.0) {
                return result;
            }
        } else {
            if (ray.delta.z < 0.0) {
                plane = floor(ray.origin.z) - WRONG_DIRECTION_OFFSET;
            } else {
                plane = floor(ray.origin.z) + VOXEL_SIZE;
            }
        }
        // start were the next plane to either side is
        result.distance = (plane - ray.origin.z) / ray.delta.z;
        // how much we'll have to move each step
        float distance_delta = abs(VOXEL_SIZE / ray.delta.z);
        // check check until we can't possibly be inside the chunk any more
        for (int i=0; i<CHUNK_SIZE; i++) {
            if (get_voxel(get_ray_point(ray, result.distance))) {
                result.success = true;
                return result;
            }
            result.distance+=distance_delta;
        }
    }
    return result;
}

RayIntersection get_chunk_intersection(Ray ray) {
    return get_best_ray_intersection(
        get_best_ray_intersection(
            ray_x_axis_chunk_intersection(ray),
            ray_y_axis_chunk_intersection(ray)
        ),
        ray_z_axis_chunk_intersection(ray)
    );
}

void main() {
    Ray ray;
    ray.origin = ray_origin_varying;
    ray.delta = ray_delta_varying;
    RayIntersection result = get_chunk_intersection(ray);
    if (result.success) {
        gl_FragColor = vec4(1, 1, 1, 1);
    } else {
        gl_FragColor = vec4(0.5, 0.5, 0.5, 1);
    }
}