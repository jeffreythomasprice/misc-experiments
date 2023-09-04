use std::{
    fmt::Display,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

use super::numbers::{BasicMath, Float};

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector3<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T> Vector3<T> {
    pub fn new(x: T, y: T, z: T) -> Self {
        Self { x, y, z }
    }
}

impl<T> Display for Vector3<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

// TODO JEFF clean up types, doesn't all have to be Float
impl<T> Vector3<T>
where
    T: Float + Copy,
{
    pub fn magnitude_squared(self) -> T {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    pub fn magnitude(self) -> T {
        self.magnitude_squared().sqrt()
    }

    pub fn normalized(self) -> Vector3<T> {
        self / self.magnitude()
    }

    pub fn dot_product(self, rhs: Self) -> T {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }

    pub fn cross_product(self, rhs: Self) -> Self {
        Self {
            x: self.y * rhs.z - self.z * rhs.y,
            y: self.z * rhs.x - self.x * rhs.z,
            z: self.x * rhs.y - self.y * rhs.x,
        }
    }
}

impl<T> Add for Vector3<T>
where
    T: BasicMath + Copy,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl<T> AddAssign for Vector3<T>
where
    T: BasicMath + Copy,
{
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}

impl<T> Sub for Vector3<T>
where
    T: BasicMath + Copy,
{
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl<T> SubAssign for Vector3<T>
where
    T: BasicMath + Copy,
{
    fn sub_assign(&mut self, rhs: Self) {
        *self += rhs
    }
}

impl<T> Mul<T> for Vector3<T>
where
    T: BasicMath + Copy,
{
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        Self::Output {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl<T> MulAssign<T> for Vector3<T>
where
    T: BasicMath + Copy,
{
    fn mul_assign(&mut self, rhs: T) {
        *self = *self * rhs
    }
}

impl<T> Div<T> for Vector3<T>
where
    T: BasicMath + Copy,
{
    type Output = Self;

    fn div(self, rhs: T) -> Self::Output {
        Self::Output {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}

impl<T> DivAssign<T> for Vector3<T>
where
    T: BasicMath + Copy,
{
    fn div_assign(&mut self, rhs: T) {
        *self = *self / rhs
    }
}

impl<T> Neg for Vector3<T>
where
    T: BasicMath + Copy,
{
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::Output {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}
