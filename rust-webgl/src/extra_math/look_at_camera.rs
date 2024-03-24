use nalgebra::{clamp, Matrix4, Point3, Unit, Vector2, Vector3, Vector4};

use super::{
    plane::Plane,
    primitives::{Float, PI},
};

pub struct LookAtCamera {
    default_forward: Unit<Vector3<Float>>,
    default_up: Unit<Vector3<Float>>,

    angle_right: Float,
    angle_up: Float,

    position: Point3<Float>,
}

impl LookAtCamera {
    pub fn new(position: Point3<Float>, target: Point3<Float>, up: Vector3<Float>) -> Self {
        let forward = Unit::new_normalize(target - position);
        let forward_abs = forward.abs();
        let default_forward = if forward_abs.x > forward_abs.y && forward_abs.x > forward_abs.z {
            if forward.x > 0.0 {
                Vector3::x_axis()
            } else {
                -Vector3::x_axis()
            }
        } else if forward_abs.y > forward_abs.z {
            if forward.y > 0.0 {
                Vector3::y_axis()
            } else {
                -Vector3::y_axis()
            }
        } else {
            if forward.z > 0.0 {
                Vector3::z_axis()
            } else {
                -Vector3::z_axis()
            }
        };

        let default_up = Unit::new_normalize(up);

        let default_right = default_up.cross(&default_forward);

        let q = Plane::new_from_normal_and_point(*default_up, &position.coords)
            .closest_point_to_point(&target);
        let u = q - position.coords;
        let a = default_forward.angle(&u);
        let angle_right = Self::fix_angle_right(if default_right.dot(&forward) > 0.0 {
            a
        } else {
            PI - a
        });

        let g = target - position;
        let a = g.angle(&u);
        let angle_up = Self::fix_angle_up(if default_up.angle(&forward) > 0.0 {
            a
        } else {
            -a
        });

        Self {
            default_forward,
            default_up,
            angle_right,
            angle_up,
            position,
        }
    }

    pub fn angle_right(&self) -> f32 {
        self.angle_right
    }

    pub fn set_angle_right(&mut self, value: f32) {
        self.angle_right = Self::fix_angle_right(value);
    }

    pub fn angle_up(&self) -> f32 {
        self.angle_up
    }

    pub fn set_angle_up(&mut self, value: f32) {
        self.angle_up = Self::fix_angle_up(value);
    }

    pub fn position(&self) -> Point3<Float> {
        self.position
    }

    pub fn set_position(&mut self, value: Point3<Float>) {
        self.position = value
    }

    pub fn turn(&mut self, mouse_movement: Vector2<Float>) {
        // TODO put constants somewhere
        let v = mouse_movement / 700.0;
        let rotation_constant = (45.0 as Float).to_radians();
        let v = v * rotation_constant;
        self.set_angle_right(self.angle_right + v.x);
        self.set_angle_up(self.angle_up + v.y);
    }

    pub fn move_position(&mut self, forward: Float, strafe: Float, up: Float) {
        self.position += self.forward() * forward;
        self.position += *self.right_right_angle_only() * strafe;
        self.position += *self.default_up * up;
    }

    pub fn transform_matrix(&self) -> Matrix4<Float> {
        // TODO cache
        Matrix4::look_at_lh(
            &self.position,
            &Point3::from(self.position.coords + self.forward()),
            &self.default_up,
        )
    }

    fn forward(&self) -> Vector3<Float> {
        // TODO cache
        Matrix4::from_axis_angle(&self.right_right_angle_only(), self.angle_up)
            .transform_vector(&self.forward_right_angle_only())
    }

    fn right_right_angle_only(&self) -> Unit<Vector3<Float>> {
        // TODO cache
        Unit::new_normalize(self.default_up.cross(&self.forward_right_angle_only()))
    }

    fn forward_right_angle_only(&self) -> Unit<Vector3<Float>> {
        // TODO cache
        Unit::new_normalize(
            Matrix4::from_axis_angle(&self.default_up, self.angle_right)
                .transform_vector(&self.default_forward),
        )
    }

    fn fix_angle_right(value: f32) -> f32 {
        let x = value % (PI * 2.0);
        if x < 0.0 {
            x + PI * 2.0
        } else {
            x
        }
    }

    fn fix_angle_up(value: f32) -> f32 {
        // TODO put constants somewhere
        let limit = PI * 0.49;
        clamp(value, -limit, limit)
    }
}
