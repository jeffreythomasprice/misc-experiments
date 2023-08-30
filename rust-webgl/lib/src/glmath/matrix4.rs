use std::ops::Mul;

use super::{angles::Radians, vector3::Vector3};

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

impl Matrix4<f32> {
    pub fn new_identity() -> Self {
        Self {
            data: [
                [1f32, 0f32, 0f32, 0f32],
                [0f32, 1f32, 0f32, 0f32],
                [0f32, 0f32, 1f32, 0f32],
                [0f32, 0f32, 0f32, 1f32],
            ],
        }
    }

    pub fn new_translation(v: Vector3<f32>) -> Self {
        Self {
            data: [
                [1f32, 0f32, 0f32, 0f32],
                [0f32, 1f32, 0f32, 0f32],
                [0f32, 0f32, 1f32, 0f32],
                [v.x, v.y, v.z, 1f32],
            ],
        }
    }

    pub fn new_scale(v: Vector3<f32>) -> Self {
        Self {
            data: [
                [v.x, 0f32, 0f32, 0f32],
                [0f32, v.y, 0f32, 0f32],
                [0f32, 0f32, v.z, 0f32],
                [0f32, 0f32, 0f32, 1f32],
            ],
        }
    }

    pub fn new_rotation(angle: Radians<f32>, axis: Vector3<f32>) -> Self {
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

    pub fn new_ortho(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Self {
        Self {
            data: [
                [2f32 / (right - left), 0f32, 0f32, 0f32],
                [0f32, 2f32 / (top - bottom), 0f32, 0f32],
                [0f32, 0f32, -2f32 / (far - near), 0f32],
                [
                    -(right + left) / (right - left),
                    -(top + bottom) / (top - bottom),
                    -(far + near) / (far - near),
                    1f32,
                ],
            ],
        }
    }

    // TODO new as perspective
    /*
    const f = 1 / Math.tan(fov / 2);
    const aspect = width / height;
    return new Matrix4(
        f / aspect, 0, 0, 0,
        0, f, 0, 0,
        0, 0, (far + near) / (near - far), 2 * far * near / (near - far),
        0, 0, -1, 0,
    );
    */

    // TODO new as lookat
    /*
    const f = target.sub(position).normalized;
    up = up.normalized;
    const s = f.cross(up).normalized;
    const u = s.cross(f).normalized;
    return new Matrix4(
        s.x, u.x, -f.x, 0,
        s.y, u.y, -f.y, 0,
        s.z, u.z, -f.z, 0,
        0, 0, 0, 1,
    )
        .mul(Matrix4.createTranslation(position.negated));
    */

    pub fn append(&mut self, other: Self) -> &mut Self {
        let other = other * *self;
        self.data = other.data;
        self
    }

    pub fn translate(&mut self, v: Vector3<f32>) -> &mut Self {
        self.append(Matrix4::new_translation(v))
    }

    pub fn scale(&mut self, v: Vector3<f32>) -> &mut Self {
        self.append(Matrix4::new_scale(v))
    }

    pub fn rotate(&mut self, angle: Radians<f32>, axis: Vector3<f32>) -> &mut Self {
        self.append(Matrix4::new_rotation(angle, axis))
    }
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
