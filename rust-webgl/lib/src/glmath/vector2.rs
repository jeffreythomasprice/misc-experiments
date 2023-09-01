use std::ops::{Add, Div, Mul, Neg, Sub};

use super::numbers::Float;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Vector2<T> {
    pub x: T,
    pub y: T,
}

impl<T> Vector2<T> {
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

impl<T> Vector2<T>
where
    T: Float + Copy,
{
    pub fn magnitude_squared(self) -> T {
        self.x * self.x + self.y * self.y
    }

    pub fn magnitude(self) -> T {
        self.magnitude_squared().sqrt()
    }

    pub fn normalized(self) -> Vector2<T> {
        self / self.magnitude()
    }

    pub fn dot_product(self, rhs: Self) -> T {
        self.x * rhs.x + self.y * rhs.y
    }
}

impl<T> Add for Vector2<T>
where
    T: Float + Copy,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl<T> Sub for Vector2<T>
where
    T: Float + Copy,
{
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl<T> Mul<T> for Vector2<T>
where
    T: Float + Copy,
{
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        Self::Output {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl<T> Div<T> for Vector2<T>
where
    T: Float + Copy,
{
    type Output = Self;

    fn div(self, rhs: T) -> Self::Output {
        Self::Output {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl<T> Neg for Vector2<T>
where
    T: Float + Copy,
{
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::Output {
            x: -self.x,
            y: -self.y,
        }
    }
}
