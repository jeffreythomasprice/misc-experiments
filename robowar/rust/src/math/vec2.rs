use crate::math::sqrt::Sqrt;
use std::ops::{Add, Div, Mul, Sub};

#[derive(Debug, Clone)]
pub struct Vec2<T> {
    pub x: T,
    pub y: T,
}

impl<T> Copy for Vec2<T> where T: Copy {}

impl<T> Vec2<T> {
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

impl<T> Vec2<T>
where
    T: Add<T, Output = T> + Mul<T, Output = T> + Copy,
{
    pub fn magnitude_squared(&self) -> T {
        self.x * self.x + self.y * self.y
    }
}

impl<T> Vec2<T>
where
    T: Add<T, Output = T> + Mul<T, Output = T> + Copy + Sqrt,
{
    pub fn magnitude(&self) -> T {
        self.magnitude_squared().sqrt()
    }
}

impl<T> Add for Vec2<T>
where
    T: Add<Output = T>,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl<T> Sub for Vec2<T>
where
    T: Sub<Output = T>,
{
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl<T> Mul<T> for Vec2<T>
where
    T: Mul<Output = T> + Copy,
{
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        Self::Output {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl<T> Div<T> for Vec2<T>
where
    T: Div<Output = T> + Copy,
{
    type Output = Self;

    fn div(self, rhs: T) -> Self::Output {
        Self::Output {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}
