use std::{
    fmt::Display,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign},
};

use bytemuck::{Pod, Zeroable};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Zeroable)]
#[repr(C)]
pub struct Vec2<T> {
    pub x: T,
    pub y: T,
}

unsafe impl<T> Pod for Vec2<T> where T: Copy + Zeroable + 'static {}

impl<T> Clone for Vec2<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            x: self.x.clone(),
            y: self.y.clone(),
        }
    }
}

impl<T> Copy for Vec2<T> where T: Copy {}

impl<T> Display for Vec2<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl<T> From<nalgebra_glm::TVec2<T>> for Vec2<T>
where
    T: Copy,
{
    fn from(value: nalgebra_glm::TVec2<T>) -> Self {
        Self { x: value[0], y: value[1] }
    }
}

impl<T> Into<nalgebra_glm::TVec2<T>> for Vec2<T> {
    fn into(self) -> nalgebra_glm::TVec2<T> {
        nalgebra_glm::TVec2::new(self.x, self.y)
    }
}

impl<T> From<rusttype::Point<T>> for Vec2<T> {
    fn from(value: rusttype::Point<T>) -> Self {
        Self { x: value.x, y: value.y }
    }
}

impl<T> Vec2<T>
where
    T: Zeroable,
{
    pub fn zeroes() -> Self {
        Self {
            x: T::zeroed(),
            y: T::zeroed(),
        }
    }
}

impl<T> Add for Vec2<T>
where
    T: Add<Output = T>,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl<T> AddAssign for Vec2<T>
where
    T: Copy + Add<Output = T>,
{
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}

impl<T> Sub for Vec2<T>
where
    T: Sub<Output = T>,
{
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl<T> SubAssign for Vec2<T>
where
    T: Copy + Sub<Output = T>,
{
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs
    }
}

impl<T> Mul<T> for Vec2<T>
where
    T: Mul<Output = T> + Copy,
{
    type Output = Self;

    fn mul(self, rhs: T) -> Self {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl<T> MulAssign<T> for Vec2<T>
where
    T: Copy + Mul<Output = T>,
{
    fn mul_assign(&mut self, rhs: T) {
        *self = *self * rhs
    }
}

impl<T> Div<T> for Vec2<T>
where
    T: Div<Output = T> + Copy,
{
    type Output = Self;

    fn div(self, rhs: T) -> Self {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl<T> DivAssign<T> for Vec2<T>
where
    T: Copy + Div<Output = T>,
{
    fn div_assign(&mut self, rhs: T) {
        *self = *self / rhs
    }
}

impl<T> Vec2<T>
where
    T: Copy + Add<Output = T> + Mul<Output = T>,
{
    pub fn magnitude_squared(&self) -> T {
        self.x * self.x + self.y * self.y
    }
}

impl Vec2<f32> {
    pub fn magnitude(&self) -> f32 {
        self.magnitude_squared().sqrt()
    }

    pub fn normalize(&self) -> Self {
        *self / self.magnitude()
    }
}

impl Vec2<f64> {
    pub fn magnitude(&self) -> f64 {
        self.magnitude_squared().sqrt()
    }

    pub fn normalize(&self) -> Self {
        *self / self.magnitude()
    }
}

impl<T> Vec2<T>
where
    T: Copy + Add<Output = T> + Mul<Output = T>,
{
    pub fn dot(&self, rhs: &Self) -> T {
        self.x * rhs.x + self.y * rhs.y
    }
}
