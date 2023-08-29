use std::ops::Mul;

use super::{angles::Radians, vector3::Vector3};

#[repr(C)]
#[derive(Debug)]
pub struct Matrix4<T> {
    data: [[T; 4]; 4],
}

impl<T> Matrix4<T> {
    pub fn flatten(&self) -> &[T] {
        self.data.flatten()
    }
}

impl Matrix4<f32> {
    pub fn identity() -> Self {
        Self {
            data: [
                [1f32, 0f32, 0f32, 0f32],
                [0f32, 1f32, 0f32, 0f32],
                [0f32, 0f32, 1f32, 0f32],
                [0f32, 0f32, 0f32, 1f32],
            ],
        }
    }

    pub fn translation(v: Vector3<f32>) -> Self {
        Self {
            data: [
                [1f32, 0f32, 0f32, 0f32],
                [0f32, 1f32, 0f32, 0f32],
                [0f32, 0f32, 1f32, 0f32],
                [v.x, v.y, v.z, 1f32],
            ],
        }
    }

    pub fn scale(v: Vector3<f32>) -> Self {
        Self {
            data: [
                [v.x, 0f32, 0f32, 0f32],
                [0f32, v.y, 0f32, 0f32],
                [0f32, 0f32, v.z, 0f32],
                [0f32, 0f32, 0f32, 1f32],
            ],
        }
    }

    pub fn rotation(angle: Radians<f32>, axis: Vector3<f32>) -> Self {
        let c = angle.cos();
        let s = angle.sin();
        let one_minus_c = 1f32 - c;
        let axis = axis.normalized();
        Self {
            data: [
                [
                    axis.x * axis.x * one_minus_c + c,
                    axis.x * axis.y * one_minus_c - axis.z * s,
                    axis.x * axis.z * one_minus_c + axis.y * s,
                    0f32,
                ],
                [
                    axis.y * axis.x * one_minus_c + axis.z * s,
                    axis.y * axis.y * one_minus_c + c,
                    axis.y * axis.z * one_minus_c - axis.x * s,
                    0f32,
                ],
                [
                    axis.x * axis.z * one_minus_c - axis.y * s,
                    axis.y * axis.z * one_minus_c + axis.x * s,
                    axis.z * axis.z * one_minus_c + c,
                    0f32,
                ],
                [0f32, 0f32, 0f32, 1f32],
            ],
        }
    }

    // TODO JEFF not right, looks wrong, looks right in shader if we do vert * mat, but it's supposed to be mat * vert and that looks wrong
    pub fn ortho(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Self {
        Self {
            data: [
                [
                    2f32 / (right - left),
                    0f32,
                    0f32,
                    -(right + left) / (right - left),
                ],
                [
                    0f32,
                    2f32 / (top - bottom),
                    0f32,
                    -(top + bottom) / (top - bottom),
                ],
                [
                    0f32,
                    0f32,
                    -2f32 / (far - near),
                    -(far + near) / (far - near),
                ],
                [0f32, 0f32, 0f32, 1f32],
            ],
        }
    }

    // TODO new as perspective
    // TODO new as lookat
    // TODO append a translation
    // TODO append a scale
    // TODO append a rotation
}

impl Mul for Matrix4<f32> {
    type Output = Matrix4<f32>;

    fn mul(self, rhs: Self) -> Self::Output {
        let mut data = [
            [0f32, 0f32, 0f32, 0f32],
            [0f32, 0f32, 0f32, 0f32],
            [0f32, 0f32, 0f32, 0f32],
            [0f32, 0f32, 0f32, 0f32],
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
