use core::f32;

use glam::{Mat4, Vec2, Vec3, vec3};

pub struct Camera {
    // when the angles are both 0, points into the camera along the midpoint of the screen
    default_forward: Vec3,
    // points toawrds the top of the screen
    default_up: Vec3,

    /*
    The actual orientation is a pair of rotations, in this order:
    - Rotate around the local Y axis, i.e. turn the camera left or right. Positive values turn right.
    - Rotate around the new local X axis, i.e. turn the camera up or down. Positive values are up.
    */
    angle_right: f32,
    angle_up: f32,

    position: Vec3,
}

impl Camera {
    pub fn new(position: Vec3, target: Vec3, up: Vec3) -> Self {
        let forward = (target - position).normalize();
        let default_forward =
            if forward.x.abs() > forward.y.abs() && forward.x.abs() > forward.z.abs() {
                if forward.x > 0.0 {
                    vec3(1.0, 0.0, 0.0)
                } else {
                    vec3(-1.0, 0.0, 0.0)
                }
            } else if forward.y.abs() > forward.z.abs() {
                if forward.y > 0.0 {
                    vec3(0.0, 1.0, 0.0)
                } else {
                    vec3(0.0, -1.0, 0.0)
                }
            } else if forward.z > 0.0 {
                vec3(0.0, 0.0, 1.0)
            } else {
                vec3(0.0, 0.0, -1.0)
            };

        let default_up = up.normalize();

        let default_right = default_up.cross(default_forward);

        let q = Plane::new_normal_point(default_up, position).closest_point_to(target);
        let u = q - position;
        let angle_right = angle_between_vectors(default_forward, u);
        let angle_right = fix_angle_right(if default_right.dot(forward) > 0.0 {
            angle_right
        } else {
            f32::consts::TAU - angle_right
        });

        let g = target - position;
        let angle_up = angle_between_vectors(g, u);
        let angle_up = fix_angle_up(if default_up.dot(forward) > 0.0 {
            -angle_up
        } else {
            angle_up
        });

        Self {
            default_forward,
            default_up,
            angle_right,
            angle_up,
            position,
        }
    }

    pub fn get_angle_right(&self) -> f32 {
        self.angle_right
    }

    pub fn set_angle_right(&mut self, angle: f32) {
        self.angle_right = fix_angle_right(angle);
    }

    pub fn get_angle_up(&self) -> f32 {
        self.angle_up
    }

    pub fn set_angle_up(&mut self, angle: f32) {
        self.angle_up = fix_angle_up(angle);
    }

    pub fn get_position(&self) -> Vec3 {
        self.position
    }

    pub fn set_position(&mut self, position: Vec3) {
        self.position = position;
    }

    pub fn turn(&mut self, mouse_movement: Vec2) {
        const SCALE: f32 = f32::consts::FRAC_PI_4 / 700.0;
        let v = mouse_movement * SCALE;
        self.set_angle_right(self.angle_right + v.x);
        self.set_angle_up(self.angle_up + v.y);
    }

    pub fn move_position(&mut self, forward: f32, strafe: f32, up: f32) {
        self.position = self.position
            + self.forward() * forward
            + self.right_right_angle_only() * strafe
            + self.default_up * up;
    }

    pub fn matrix(&self) -> Mat4 {
        // TODO cache
        Mat4::look_to_lh(self.position, self.forward(), self.default_up)
    }

    fn forward(&self) -> Vec3 {
        // TODO cache
        Mat4::from_axis_angle(self.right_right_angle_only(), self.angle_up)
            .transform_vector3(self.forward_right_angle_only())
    }

    fn right_right_angle_only(&self) -> Vec3 {
        // TODO cache
        self.default_up.cross(self.forward_right_angle_only())
    }

    fn forward_right_angle_only(&self) -> Vec3 {
        // TODO cache
        Mat4::from_axis_angle(self.default_up, self.angle_right)
            .transform_vector3(self.default_forward)
    }
}

fn fix_angle_right(angle: f32) -> f32 {
    let x = angle % f32::consts::TAU;
    if x < 0.0 { x + f32::consts::TAU } else { x }
}

fn fix_angle_up(angle: f32) -> f32 {
    const LIMIT: f32 = f32::consts::PI * 0.49;
    angle.clamp(-LIMIT, LIMIT)
}

struct Plane {
    normal: Vec3,
    d: f32,
}

impl Plane {
    pub fn new_normal_point(normal: Vec3, point: Vec3) -> Self {
        let normal = normal.normalize();
        Self {
            normal,
            d: normal.dot(point),
        }
    }

    pub fn signed_distance_to(&self, point: Vec3) -> f32 {
        self.d - self.normal.dot(point)
    }

    pub fn closest_point_to(&self, point: Vec3) -> Vec3 {
        self.normal * self.signed_distance_to(point) + point
    }
}

fn angle_between_vectors(a: Vec3, b: Vec3) -> f32 {
    /*
    https://stackoverflow.com/a/16544330/9290998
    https://stackoverflow.com/a/67719217/9290998
    x = dot(a, b)
    y = dot(n, cross(a, b))
    angle = atan2(y, x)
    */
    a.cross(b).length().atan2(a.dot(b))
}
