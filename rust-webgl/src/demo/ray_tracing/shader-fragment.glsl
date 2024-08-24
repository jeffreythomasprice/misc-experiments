precision mediump float;

varying vec3 ray_origin_varying;
varying vec3 ray_delta_varying;

const int CHUNK_SIZE = 16;

struct Ray {
    vec3 origin;
    vec3 delta;
};

struct RayIntersection {
    bool success;
    float distance;
};

bool get_voxel(vec3 point) {
    // TODO voxel data in texture? some kind of uniform?
    ivec3 ipoint = ivec3(
        int(point.x),
        int(point.y),
        int(point.z)
    );
    // are we on the min or max of each axis
    bool x = ipoint.x == 0 || ipoint.x == 15;
    bool y = ipoint.y == 0 || ipoint.y == 15;
    bool z = ipoint.z == 0 || ipoint.z == 15;
    // edges and corners get filled in, interior and chunk faces don't
    return (x ? 1 : 0) + (y ? 1 : 0) + (z ? 1 : 0) >= 2;
}

vec3 get_ray_point(Ray ray, float distance) {
    return ray.origin + ray.delta * distance;
}

RayIntersection ray_x_axis_plane_intersection(Ray ray, float plane) {
    RayIntersection result;
    if (ray.origin.x == 0.0) {
        result.success = false;
    } else {
        result.distance = (plane - ray.origin.x) / ray.delta.x;
        result.success = result.distance >= 0.0;
    }
    return result;
}

RayIntersection ray_y_axis_plane_intersection(Ray ray, float plane) {
    RayIntersection result;
    if (ray.origin.y == 0.0) {
        result.success = false;
    } else {
        result.distance = (plane - ray.origin.y) / ray.delta.y;
        result.success = result.distance >= 0.0;
    }
    return result;
}

RayIntersection ray_z_axis_plane_intersection(Ray ray, float plane) {
    RayIntersection result;
    if (ray.origin.z == 0.0) {
        result.success = false;
    } else {
        result.distance = (plane - ray.origin.z) / ray.delta.z;
        result.success = result.distance >= 0.0;
    }
    return result;
}

RayIntersection ray_x_axis_chunk_intersection(Ray ray) {
    RayIntersection best;
    best.success = false;
    if (ray.delta.x != 0.0) {
        float plane_offset;
        if (ray.delta.x < 0.0) {
            plane_offset = 0.999;
        } else {
            plane_offset = 0.0;
        }
        for (int i=0; i<CHUNK_SIZE; i++) {
            RayIntersection current = ray_x_axis_plane_intersection(ray, float(i) + plane_offset);
            if (
                current.success
                && get_voxel(get_ray_point(ray, current.distance))
                && (!best.success || current.distance < best.distance)
            ) {
                best = current;
            }
        }
    }
    return best;
}

RayIntersection ray_y_axis_chunk_intersection(Ray ray) {
    RayIntersection best;
    best.success = false;
    if (ray.delta.y != 0.0) {
        float plane_offset;
        if (ray.delta.y < 0.0) {
            plane_offset = 0.999;
        } else {
            plane_offset = 0.0;
        }
        for (int i=0; i<CHUNK_SIZE; i++) {
            RayIntersection current = ray_y_axis_plane_intersection(ray, float(i) + plane_offset);
            if (
                current.success
                && get_voxel(get_ray_point(ray, current.distance))
                && (!best.success || current.distance < best.distance)
            ) {
                best = current;
            }
        }
    }
    return best;
}

RayIntersection ray_z_axis_chunk_intersection(Ray ray) {
    RayIntersection best;
    best.success = false;
    if (ray.delta.z != 0.0) {
        float plane_offset;
        if (ray.delta.z < 0.0) {
            plane_offset = 0.999;
        } else {
            plane_offset = 0.0;
        }
        for (int i=0; i<CHUNK_SIZE; i++) {
            RayIntersection current = ray_z_axis_plane_intersection(ray, float(i) + plane_offset);
            if (
                current.success
                && get_voxel(get_ray_point(ray, current.distance))
                && (!best.success || current.distance < best.distance)
            ) {
                best = current;
            }
        }
    }
    return best;
}

RayIntersection get_chunk_intersection(Ray ray) {
    RayIntersection result_x = ray_x_axis_chunk_intersection(ray);
    RayIntersection result_y = ray_y_axis_chunk_intersection(ray);
    RayIntersection result_z = ray_z_axis_chunk_intersection(ray);
    
    RayIntersection best = result_x;
    if (result_y.success && result_y.distance < best.distance) {
        best = result_y;
    }
    if (result_z.success && result_z.distance < best.distance) {
        best = result_z;
    }
    return best;
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