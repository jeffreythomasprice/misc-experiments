use std::fmt::Display;

use super::{
    angles::Radians,
    matrix4::Matrix4,
    numbers::{CouldBeAnAngle, Float},
    vector3::Vector3,
};

#[derive(Debug, Clone)]
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

    // the rotation around the camera's local x axis, that is the cross product of the look and up vectors
    angle_x: Radians<T>,
    // the rotation around the camera's local y axis, that is the up vector
    angle_y: Radians<T>,

    // the current position
    position: Vector3<T>,

    // a transformation matrix that applies the above
    matrix: Matrix4<T>,
}

impl<T> FPSCamera<T>
where
    // TODO JEFF doesn't need Display
    T: Float + Copy + PartialOrd + Display,
    Radians<T>: CouldBeAnAngle<Output = T>,
{
    /*
    position is the initial position

    look is the vector pointing in the direction the camera is pointing when the rotation angles are both zero

    up is the vector pointing towards the top of the screen in the camera's space
    */
    pub fn new(position: Vector3<T>, look: Vector3<T>, up: Vector3<T>) -> Self {
        let local_x = look.cross_product(up);
        let local_y = local_x.cross_product(look);
        let local_z = local_y.cross_product(local_x);

        let mut result = Self {
            default_local_x: local_x.normalized(),
            default_local_y: local_y.normalized(),
            default_local_z: local_z.normalized(),
            angle_x: Radians(T::ZERO),
            angle_y: Radians(T::ZERO),
            position,
            matrix: Matrix4::new_identity(),
        };

        // TODO JEFF testing initial conditions
        result.angle_y = Radians(T::FRAC_PI_4);
        result.angle_x = Radians(T::FRAC_PI_4);

        result.update();
        return result;
    }

    // TODO JEFF camera public accessors

    pub fn position(&self) -> Vector3<T> {
        self.position
    }

    pub fn set_position(&mut self, value: Vector3<T>) -> &mut Self {
        self.position = value;
        self.update()
    }

    pub fn angle_x(&self) -> Radians<T> {
        self.angle_x
    }

    pub fn set_angle_x(&mut self, value: Radians<T>) -> &mut Self {
        // TODO JEFF clamp
        let max = Radians(T::PI);
        let min = -max;
        self.angle_x = if value < min {
            min
        } else if value > max {
            max
        } else {
            value
        };
        self.update()
    }

    pub fn angle_y(&self) -> Radians<T> {
        self.angle_y
    }

    pub fn set_angle_y(&mut self, value: Radians<T>) -> &mut Self {
        let mut value = value % Radians(T::PI * T::TWO);
        if value < Radians(T::ZERO) {
            value += Radians(T::PI * T::TWO);
        }
        self.angle_y = value;
        self.update()
    }

    pub fn matrix(&self) -> &Matrix4<T> {
        &self.matrix
    }

    // TODO JEFF helper for moving based on current look direction
    // TODO JEFF helper for turning based on camera local xy (i.e. mouse movement)

    pub fn look_at(&mut self, target: Vector3<T>) -> &mut Self {
        // TODO JEFF this is all wrong

        log::debug!("TODO JEFF look_at, target = {target}");

        // the direction we want the camera to be pointing in
        let desired_look = target - self.position;
        log::debug!("TODO JEFF desired_look = {desired_look}");
        // the projection of that vector onto the local y axis
        let look_y = self.default_local_y
            * (desired_look.dot_product(self.default_local_y)
                / self.default_local_y.magnitude_squared());
        log::debug!("TODO JEFF look_y = {look_y}");
        // the projection of the look vector onto the camera's local XZ plane
        let look_xz = desired_look - look_y;
        log::debug!("TODO JEFF look_xz = {look_xz}");
        // now the angles are given by the angles between that projection on the XZ plane and other reference angles
        // the angle around the camera's Y axis is given by the amount we have to turn from the default Z axis
        // and then around the camera's X axis by the amount we have to go up or down from the XZ plane to meet the desired look angle
        let directionless_angle_y = Radians(
            (look_xz.dot_product(self.default_local_z)
                / (look_xz.magnitude() * self.default_local_z.magnitude()))
            .acos(),
        );
        log::debug!(
            "TODO JEFF directionless_angle_y = {}",
            directionless_angle_y.to_degrees()
        );
        let directionless_angle_x = Radians(
            (look_xz.dot_product(desired_look) / (look_xz.magnitude() * desired_look.magnitude()))
                .acos(),
        );
        log::debug!(
            "TODO JEFF directionless_angle_x = {}",
            directionless_angle_x.to_degrees()
        );
        // that's jsut the shorted angle between those vectors
        // to get the true rotation angles for the camera we need to adjust based on which direction the look vector is relative to the
        // standard directions
        // TODO JEFF avoid updating twice
        self.set_angle_x(
            if desired_look.dot_product(self.default_local_y) > T::ZERO {
                directionless_angle_x
            } else {
                -directionless_angle_x
            },
        );
        self.set_angle_y(
            if desired_look.dot_product(self.default_local_x) > T::ZERO {
                directionless_angle_y
            } else {
                Radians(T::PI * T::TWO) - directionless_angle_y
            },
        );
        log::debug!("TODO JEFF angle_x = {}", self.angle_x.to_degrees());
        log::debug!("TODO JEFF angle_y = {}", self.angle_y.to_degrees());
        self
    }

    fn update(&mut self) -> &mut Self {
        let mut rotation = Matrix4::new_rotation(self.angle_y, self.default_local_y);
        let local_x = rotation.apply_to_vector(self.default_local_x);
        let local_z = rotation.apply_to_vector(self.default_local_z);
        log::debug!("TODO JEFF only applying y\nlocal_x = {local_x}\nlocal_z = {local_z}");

        rotation.rotate(self.angle_x, local_x);
        let local_x = rotation.apply_to_vector(self.default_local_x);
        let local_y = rotation.apply_to_vector(self.default_local_y);
        let local_z = rotation.apply_to_vector(self.default_local_z);
        log::debug!("TODO JEFF both rotations\nlocal_x = {local_x}\nlocal_y = {local_y}\nlocal_z = {local_z}");

        // TODO JEFF fix camera matrix
        self.matrix = Matrix4::new_look_at(self.position, self.position + local_z, local_y);

        self
    }
}
