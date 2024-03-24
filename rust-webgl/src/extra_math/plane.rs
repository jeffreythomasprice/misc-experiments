use nalgebra::{Point3, Vector3};

use super::primitives::Float;

pub struct Plane {
    normal: Vector3<Float>,
    d: Float,
}

impl Plane {
    pub fn new_from_normal_and_point(normal: Vector3<Float>, point: &Vector3<Float>) -> Self {
        Self {
            normal,
            d: normal.dot(point),
        }
    }

    /**
    Finds t such that (point + normal*t) is a point on the plane.
    */
    pub fn signed_distance_to_point(&self, point: &Point3<Float>) -> Float {
        self.d - self.normal.dot(&point.coords)
    }

    pub fn closest_point_to_point(&self, point: &Point3<Float>) -> Vector3<Float> {
        self.normal * self.signed_distance_to_point(point) + point.coords
    }
}
