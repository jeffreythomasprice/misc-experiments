use std::ops::{Add, Div, Mul, Sub};

use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Vec2u32 {
    pub x: u32,
    pub y: u32,
}

impl Vec2u32 {
    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }
}

impl Add<Self> for Vec2u32 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub<Self> for Vec2u32 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Mul<u32> for Vec2u32 {
    type Output = Self;

    fn mul(self, rhs: u32) -> Self::Output {
        Self::Output {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Div<u32> for Vec2u32 {
    type Output = Self;

    fn div(self, rhs: u32) -> Self::Output {
        Self::Output {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}
