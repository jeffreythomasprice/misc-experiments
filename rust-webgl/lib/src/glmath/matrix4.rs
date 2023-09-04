use std::ops::Mul;

use super::{
    angles::Radians,
    numbers::{CouldBeAnAngle, Float},
    vector3::Vector3,
};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Matrix4<T> {
    data: [[T; 4]; 4],
}

impl<T> Matrix4<T> {
    pub fn flatten(&self) -> &[T] {
        self.data.flatten()
    }
}

impl<T> Matrix4<T>
where
    T: Float + Copy,
    Radians<T>: CouldBeAnAngle<Output = T>,
{
    pub fn new_identity() -> Self {
        Self {
            data: [
                [T::ONE, T::ZERO, T::ZERO, T::ZERO],
                [T::ZERO, T::ONE, T::ZERO, T::ZERO],
                [T::ZERO, T::ZERO, T::ONE, T::ZERO],
                [T::ZERO, T::ZERO, T::ZERO, T::ONE],
            ],
        }
    }

    pub fn new_translation(v: Vector3<T>) -> Self {
        Self {
            data: [
                [T::ONE, T::ZERO, T::ZERO, T::ZERO],
                [T::ZERO, T::ONE, T::ZERO, T::ZERO],
                [T::ZERO, T::ZERO, T::ONE, T::ZERO],
                [v.x, v.y, v.z, T::ONE],
            ],
        }
    }

    pub fn new_scale(v: Vector3<T>) -> Self {
        Self {
            data: [
                [v.x, T::ZERO, T::ZERO, T::ZERO],
                [T::ZERO, v.y, T::ZERO, T::ZERO],
                [T::ZERO, T::ZERO, v.z, T::ZERO],
                [T::ZERO, T::ZERO, T::ZERO, T::ONE],
            ],
        }
    }

    pub fn new_rotation(angle: Radians<T>, axis: Vector3<T>) -> Self {
        let c: T = angle.cos();
        let s: T = angle.sin();
        let one_minus_c = T::ONE - c;
        let axis = axis.normalized();
        Self {
            data: [
                [
                    axis.x * axis.x * one_minus_c + c,
                    axis.x * axis.y * one_minus_c - axis.z * s,
                    axis.x * axis.z * one_minus_c + axis.y * s,
                    T::ZERO,
                ],
                [
                    axis.y * axis.x * one_minus_c + axis.z * s,
                    axis.y * axis.y * one_minus_c + c,
                    axis.y * axis.z * one_minus_c - axis.x * s,
                    T::ZERO,
                ],
                [
                    axis.x * axis.z * one_minus_c - axis.y * s,
                    axis.y * axis.z * one_minus_c + axis.x * s,
                    axis.z * axis.z * one_minus_c + c,
                    T::ZERO,
                ],
                [T::ZERO, T::ZERO, T::ZERO, T::ONE],
            ],
        }
    }

    pub fn new_ortho(left: T, right: T, bottom: T, top: T, near: T, far: T) -> Self {
        Self {
            data: [
                [T::TWO / (right - left), T::ZERO, T::ZERO, T::ZERO],
                [T::ZERO, T::TWO / (top - bottom), T::ZERO, T::ZERO],
                [T::ZERO, T::ZERO, -T::TWO / (far - near), T::ZERO],
                [
                    -(right + left) / (right - left),
                    -(top + bottom) / (top - bottom),
                    -(far + near) / (far - near),
                    T::ONE,
                ],
            ],
        }
    }

    pub fn new_perspective(fov: Radians<T>, width: T, height: T, near: T, far: T) -> Self {
        let f = T::ONE / (fov.0 / T::TWO).tan();
        let aspect = width / height;
        Self {
            data: [
                [f / aspect, T::ZERO, T::ZERO, T::ZERO],
                [T::ZERO, f, T::ZERO, T::ZERO],
                [T::ZERO, T::ZERO, (far + near) / (near - far), -T::ONE],
                [T::ZERO, T::ZERO, T::TWO * far * near / (near - far), T::ONE],
            ],
        }
    }

    pub fn new_look_at(position: Vector3<T>, target: Vector3<T>, up: Vector3<T>) -> Self {
        let f = (target - position).normalized();
        let up = up.normalized();
        let s = f.cross_product(up).normalized();
        let u = s.cross_product(f).normalized();
        *Self {
            data: [
                [s.x, u.x, -f.x, T::ZERO],
                [s.y, u.y, -f.y, T::ZERO],
                [s.z, u.z, -f.z, T::ZERO],
                [T::ZERO, T::ZERO, T::ZERO, T::ONE],
            ],
        }
        .translate(-position)
    }

    pub fn append(&mut self, other: Self) -> &mut Self {
        let other = other * *self;
        self.data = other.data;
        self
    }

    pub fn translate(&mut self, v: Vector3<T>) -> &mut Self {
        self.append(Matrix4::new_translation(v))
    }

    pub fn scale(&mut self, v: Vector3<T>) -> &mut Self {
        self.append(Matrix4::new_scale(v))
    }

    pub fn rotate(&mut self, angle: Radians<T>, axis: Vector3<T>) -> &mut Self {
        self.append(Matrix4::new_rotation(angle, axis))
    }
}

impl<T> Matrix4<T>
where
    T: Float + Copy,
{
    pub fn apply_to_point(&self, input: Vector3<T>) -> Vector3<T> {
        Vector3::new(
            self.data[0][0] * input.x
                + self.data[1][0] * input.y
                + self.data[2][0] * input.z
                + self.data[3][0],
            self.data[0][1] * input.x
                + self.data[1][1] * input.y
                + self.data[2][1] * input.z
                + self.data[3][1],
            self.data[0][2] * input.x
                + self.data[1][2] * input.y
                + self.data[2][2] * input.z
                + self.data[3][2],
        )
    }

    pub fn apply_to_vector(&self, input: Vector3<T>) -> Vector3<T> {
        Vector3::new(
            self.data[0][0] * input.x + self.data[1][0] * input.y + self.data[2][0] * input.z,
            self.data[0][1] * input.x + self.data[1][1] * input.y + self.data[2][1] * input.z,
            self.data[0][2] * input.x + self.data[1][2] * input.y + self.data[2][2] * input.z,
        )
    }
}

impl<T> Mul for Matrix4<T>
where
    T: Float + Copy,
{
    type Output = Matrix4<T>;

    fn mul(self, rhs: Self) -> Self::Output {
        let mut data = [
            [T::ZERO, T::ZERO, T::ZERO, T::ZERO],
            [T::ZERO, T::ZERO, T::ZERO, T::ZERO],
            [T::ZERO, T::ZERO, T::ZERO, T::ZERO],
            [T::ZERO, T::ZERO, T::ZERO, T::ZERO],
        ];
        for (i, row) in data.iter_mut().enumerate() {
            for (j, col) in row.iter_mut().enumerate() {
                for k in 0..4 {
                    *col += self.data[i][k] * rhs.data[k][j];
                }
            }
        }
        Self::Output { data }
    }
}

#[cfg(test)]
pub mod tests {
    use std::fmt::Display;

    use crate::glmath::angles::Degrees;

    use super::*;

    #[test]
    fn identity() {
        let m = Matrix4::new_identity();
        assert_vector3_close_to(
            m.apply_to_point(Vector3::new(1f64, 2f64, 3f64)),
            Vector3::new(1f64, 2f64, 3f64),
            1e-8f64,
        );
        assert_vector3_close_to(
            m.apply_to_vector(Vector3::new(1f64, 2f64, 3f64)),
            Vector3::new(1f64, 2f64, 3f64),
            1e-8f64,
        );
    }

    #[test]
    fn translation() {
        let m = Matrix4::new_translation(Vector3::new(1f64, 2f64, 3f64));
        assert_vector3_close_to(
            m.apply_to_point(Vector3::new(1f64, 1f64, 1f64)),
            Vector3::new(2f64, 3f64, 4f64),
            1e-8f64,
        );
        assert_vector3_close_to(
            m.apply_to_vector(Vector3::new(1f64, 1f64, 1f64)),
            Vector3::new(1f64, 1f64, 1f64),
            1e-8f64,
        );
    }

    #[test]
    fn scale() {
        let m = Matrix4::new_scale(Vector3::new(2f64, 3f64, 4f64));
        assert_vector3_close_to(
            m.apply_to_point(Vector3::new(2f64, 2f64, 2f64)),
            Vector3::new(4f64, 6f64, 8f64),
            1e-8f64,
        );
        assert_vector3_close_to(
            m.apply_to_vector(Vector3::new(2f64, 2f64, 2f64)),
            Vector3::new(4f64, 6f64, 8f64),
            1e-8f64,
        );
    }

    #[test]
    fn rotation_x() {
        let m = Matrix4::new_rotation(Degrees(90f64).into(), Vector3::new(1f64, 0f64, 0f64));
        assert_vector3_close_to(
            m.apply_to_point(Vector3::new(1f64, 1f64, 1f64)),
            Vector3::new(1f64, 1f64, -1f64),
            1e-8f64,
        );
        assert_vector3_close_to(
            m.apply_to_vector(Vector3::new(1f64, 1f64, 1f64)),
            Vector3::new(1f64, 1f64, -1f64),
            1e-8f64,
        );

        let m = Matrix4::new_rotation(Degrees(-90f64).into(), Vector3::new(1f64, 0f64, 0f64));
        assert_vector3_close_to(
            m.apply_to_point(Vector3::new(1f64, 1f64, 1f64)),
            Vector3::new(1f64, -1f64, 1f64),
            1e-8f64,
        );
        assert_vector3_close_to(
            m.apply_to_vector(Vector3::new(1f64, 1f64, 1f64)),
            Vector3::new(1f64, -1f64, 1f64),
            1e-8f64,
        );

        let m = Matrix4::new_rotation(Degrees(90f64).into(), Vector3::new(-1f64, 0f64, 0f64));
        assert_vector3_close_to(
            m.apply_to_point(Vector3::new(1f64, 1f64, 1f64)),
            Vector3::new(1f64, -1f64, 1f64),
            1e-8f64,
        );
        assert_vector3_close_to(
            m.apply_to_vector(Vector3::new(1f64, 1f64, 1f64)),
            Vector3::new(1f64, -1f64, 1f64),
            1e-8f64,
        );
    }

    #[test]
    fn rotation_y() {
        let m = Matrix4::new_rotation(Degrees(90f64).into(), Vector3::new(0f64, 1f64, 0f64));
        assert_vector3_close_to(
            m.apply_to_point(Vector3::new(1f64, 1f64, 1f64)),
            Vector3::new(-1f64, 1f64, 1f64),
            1e-8f64,
        );
        assert_vector3_close_to(
            m.apply_to_vector(Vector3::new(1f64, 1f64, 1f64)),
            Vector3::new(-1f64, 1f64, 1f64),
            1e-8f64,
        );

        let m = Matrix4::new_rotation(Degrees(-90f64).into(), Vector3::new(0f64, 1f64, 0f64));
        assert_vector3_close_to(
            m.apply_to_point(Vector3::new(1f64, 1f64, 1f64)),
            Vector3::new(1f64, 1f64, -1f64),
            1e-8f64,
        );
        assert_vector3_close_to(
            m.apply_to_vector(Vector3::new(1f64, 1f64, 1f64)),
            Vector3::new(1f64, 1f64, -1f64),
            1e-8f64,
        );

        let m = Matrix4::new_rotation(Degrees(90f64).into(), Vector3::new(0f64, -1f64, 0f64));
        assert_vector3_close_to(
            m.apply_to_point(Vector3::new(1f64, 1f64, 1f64)),
            Vector3::new(1f64, 1f64, -1f64),
            1e-8f64,
        );
        assert_vector3_close_to(
            m.apply_to_vector(Vector3::new(1f64, 1f64, 1f64)),
            Vector3::new(1f64, 1f64, -1f64),
            1e-8f64,
        );
    }

    #[test]
    fn rotation_z() {
        let m = Matrix4::new_rotation(Degrees(90f64).into(), Vector3::new(0f64, 0f64, 1f64));
        assert_vector3_close_to(
            m.apply_to_point(Vector3::new(1f64, 1f64, 1f64)),
            Vector3::new(1f64, -1f64, 1f64),
            1e-8f64,
        );
        assert_vector3_close_to(
            m.apply_to_vector(Vector3::new(1f64, 1f64, 1f64)),
            Vector3::new(1f64, -1f64, 1f64),
            1e-8f64,
        );

        let m = Matrix4::new_rotation(Degrees(-90f64).into(), Vector3::new(0f64, 0f64, 1f64));
        assert_vector3_close_to(
            m.apply_to_point(Vector3::new(1f64, 1f64, 1f64)),
            Vector3::new(-1f64, 1f64, 1f64),
            1e-8f64,
        );
        assert_vector3_close_to(
            m.apply_to_vector(Vector3::new(1f64, 1f64, 1f64)),
            Vector3::new(-1f64, 1f64, 1f64),
            1e-8f64,
        );

        let m = Matrix4::new_rotation(Degrees(90f64).into(), Vector3::new(0f64, 0f64, -1f64));
        assert_vector3_close_to(
            m.apply_to_point(Vector3::new(1f64, 1f64, 1f64)),
            Vector3::new(-1f64, 1f64, 1f64),
            1e-8f64,
        );
        assert_vector3_close_to(
            m.apply_to_vector(Vector3::new(1f64, 1f64, 1f64)),
            Vector3::new(-1f64, 1f64, 1f64),
            1e-8f64,
        );
    }

    #[test]
    fn translation_and_rotation() {
        let m = *Matrix4::new_identity()
            .rotate(Degrees(90f64).into(), Vector3::new(0f64, 1f64, 0f64))
            .translate(Vector3::new(1f64, 2f64, 3f64));
        assert_vector3_close_to(
            m.apply_to_point(Vector3::new(1f64, 1f64, 1f64)),
            Vector3::new(-4f64, 3f64, 2f64),
            1e-8f64,
        );
        assert_vector3_close_to(
            m.apply_to_vector(Vector3::new(1f64, 1f64, 1f64)),
            Vector3::new(-1f64, 1f64, 1f64),
            1e-8f64,
        );
    }

    fn assert_vector3_close_to<T>(a: Vector3<T>, b: Vector3<T>, max_distance: T)
    where
        T: Float + Copy + PartialOrd + Display,
    {
        let actual_distance = (a - b).magnitude();
        assert!(
            actual_distance < max_distance,
            "expected {} and {} to be within {}, actual distance {}",
            a,
            b,
            max_distance,
            actual_distance
        );
    }
}
