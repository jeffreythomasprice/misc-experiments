use std::cell::RefCell;

use crate::errors::Result;

use super::{
    angles::{Degrees, Radians},
    matrix4::Matrix4,
    numbers::{CouldBeAnAngle, Float},
    vector3::Vector3,
};

#[derive(Clone)]
struct CalculatedState<T> {
    local_x: Vector3<T>,
    local_y: Vector3<T>,
    local_z: Vector3<T>,
    matrix: Matrix4<T>,
}

pub struct FPSCamera<T>
where
    T: Float + Copy,
{
    // the vectors pointing along in the "standard" look direction
    // this matches the default opengl perspective camera, where:
    // positive x = towards the right of the screen
    // positive y = towards the top of the screen
    // positive z = towards the camera (so negative z = in the direction the camera is looking)
    default_local_x: Vector3<T>,
    default_local_y: Vector3<T>,
    default_local_z: Vector3<T>,

    // the range of possible values each angle can take
    min_angle_x: Radians<T>,
    max_angle_x: Radians<T>,
    min_angle_y: Radians<T>,
    max_angle_y: Radians<T>,

    // the rotation around the camera's local x axis, that is the cross product of the look and up vectors
    angle_x: Radians<T>,
    // the rotation around the camera's local y axis, that is the up vector
    angle_y: Radians<T>,

    // the current position
    position: Vector3<T>,

    // a transformation matrix that applies the above
    // TODO should this have a mutex?
    calculated_state: RefCell<Option<CalculatedState<T>>>,
}

impl<T> FPSCamera<T>
where
    T: Float + Copy + PartialOrd,
    Radians<T>: CouldBeAnAngle<Output = T>,
{
    /*
    position is the initial position

    look is the vector pointing in the direction the camera is pointing when the rotation angles are both zero

    up is the vector pointing towards the top of the screen in the camera's space
    */
    pub fn new(position: Vector3<T>, look: Vector3<T>, up: Vector3<T>) -> Result<Self> {
        let local_x = look.cross_product(up);
        let local_y = local_x.cross_product(look);
        let local_z = local_y.cross_product(local_x);

        let max_angle_x = Degrees(T::from_f64(89f64)?);
        let min_angle_x = -max_angle_x;
        let min_angle_y = Degrees(T::ZERO);
        let max_angle_y = Radians(T::PI * T::TWO);

        Ok(Self {
            default_local_x: local_x.normalized(),
            default_local_y: local_y.normalized(),
            default_local_z: local_z.normalized(),

            min_angle_x: min_angle_x.to_radians(),
            max_angle_x: max_angle_x.to_radians(),
            min_angle_y: min_angle_y.to_radians(),
            max_angle_y,

            angle_x: Radians(T::ZERO),
            angle_y: Radians(T::ZERO),
            position,
            calculated_state: RefCell::new(None),
        })
    }

    pub fn position(&self) -> Vector3<T> {
        self.position
    }

    pub fn set_position(&mut self, value: Vector3<T>) {
        self.position = value;
        self.calculated_state.replace(None);
    }

    pub fn angle_x(&self) -> Radians<T> {
        self.angle_x
    }

    pub fn set_angle_x(&mut self, value: Radians<T>) {
        self.angle_x = if value < self.min_angle_x {
            self.min_angle_x
        } else if value > self.max_angle_x {
            self.max_angle_x
        } else {
            value
        };
        self.calculated_state.replace(None);
    }

    pub fn angle_y(&self) -> Radians<T> {
        self.angle_y
    }

    pub fn set_angle_y(&mut self, value: Radians<T>) {
        let range = self.max_angle_y - self.min_angle_y;
        let mut value = (value - self.min_angle_y) % range;
        if value < Radians(T::ZERO) {
            value += range;
        }
        self.angle_y = value + self.min_angle_y;
        self.calculated_state.replace(None);
    }

    pub fn move_by(&mut self, forward: T, right: T, up: T) {
        let state = self.get_or_update_calculated_state();
        self.set_position(
            self.position + state.local_x * right + state.local_y * up + state.local_z * forward,
        );
    }

    /*
    the delta adjusts the x and y angles

    if based on mouse movement:
    the x component should be based on mouse y, because it cooresponds to looking up and down
    the y component should be based on mouse x, because it corresponds to looking left and right
    */
    pub fn turn(&mut self, x: Radians<T>, y: Radians<T>) {
        self.set_angle_x(self.angle_x + x);
        self.set_angle_y(self.angle_y + y);
    }

    pub fn look_at(&mut self, target: Vector3<T>) {
        // the direction we want the camera to be pointing in
        let desired_look = target - self.position;

        // the projection of that vector onto the local y axis
        let look_y = self.default_local_y
            * (desired_look.dot_product(self.default_local_y)
                / self.default_local_y.magnitude_squared());

        // the projection of the look vector onto the camera's local XZ plane
        let look_xz = desired_look - look_y;

        // now the angles are given by the angles between that projection on the XZ plane and other reference angles
        // the angle around the camera's Y axis is given by the amount we have to turn from the default Z axis
        // and then around the camera's X axis by the amount we have to go up or down from the XZ plane to meet the desired look angle
        let directionless_angle_y = Radians(
            (look_xz.dot_product(self.default_local_z)
                / (look_xz.magnitude() * self.default_local_z.magnitude()))
            .acos(),
        );
        let directionless_angle_x = Radians(
            (look_xz.dot_product(desired_look) / (look_xz.magnitude() * desired_look.magnitude()))
                .acos(),
        );

        // that's jsut the shorted angle between those vectors
        // to get the true rotation angles for the camera we need to adjust based on which direction the look vector is relative to the
        // standard directions
        self.set_angle_x(
            if desired_look.dot_product(self.default_local_y) > T::ZERO {
                -directionless_angle_x
            } else {
                directionless_angle_x
            },
        );
        self.set_angle_y(
            if desired_look.dot_product(self.default_local_x) > T::ZERO {
                directionless_angle_y
            } else {
                Radians(T::PI * T::TWO) - directionless_angle_y
            },
        );

        self.calculated_state.replace(None);
    }

    pub fn matrix(&mut self) -> Matrix4<T> {
        self.get_or_update_calculated_state().matrix
    }

    fn get_or_update_calculated_state(&mut self) -> CalculatedState<T> {
        let state = &mut *self.calculated_state.borrow_mut();
        match state {
            Some(result) => result.clone(),
            None => {
                // first rotate around the local y axis
                // so the new y axis is still default, but we need the other two
                let rotation_y = Matrix4::new_rotation(self.angle_y, self.default_local_y);
                let local_x = rotation_y.apply_to_vector(self.default_local_x);
                let local_z = rotation_y.apply_to_vector(self.default_local_z);

                // now apply the x axis rotation
                // technically this moves the y axis too, but look_at matrix is going to fix the vectors to be perpendicular via cross
                // products anyway, so just find the new z
                let rotation_x = Matrix4::new_rotation(self.angle_x, local_x);
                let local_z = rotation_x.apply_to_vector(local_z);

                // update the state var
                let result = CalculatedState {
                    local_x,
                    // we don't actually rotate the up vector, all movement should be parallel to the default up
                    local_y: self.default_local_y,
                    local_z,
                    matrix: Matrix4::new_look_at(
                        self.position,
                        self.position + local_z,
                        self.default_local_y,
                    ),
                };
                state.replace(result.clone());
                result
            }
        }
    }
}
