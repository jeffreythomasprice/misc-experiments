use std::ops::Mul;

use super::{
    angles::Radians,
    numbers::{BasicMath, CouldBeAnAngle, Float},
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
        let c: T = angle.cos().into();
        let s: T = angle.sin().into();
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
        for i in 0..4 {
            for j in 0..4 {
                for k in 0..4 {
                    data[i][j] += self.data[i][k] * rhs.data[k][j];
                }
            }
        }
        Self::Output { data }
    }
}
