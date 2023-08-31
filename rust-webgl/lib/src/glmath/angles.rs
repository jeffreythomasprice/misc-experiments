use std::{
    f32::consts::PI,
    ops::{Add, Div, Mul, Rem, Sub},
};

#[derive(Debug, Clone, Copy)]
pub struct Radians<T>(pub T);

#[derive(Debug, Clone, Copy)]
pub struct Degrees<T>(pub T);

impl<T> Radians<T> {
    pub fn new(value: T) -> Self {
        Self(value)
    }
}

impl Radians<f32> {
    pub fn cos(self) -> f32 {
        self.0.cos()
    }

    pub fn sin(self) -> f32 {
        self.0.sin()
    }
}

impl<T> Degrees<T> {
    pub fn new(value: T) -> Self {
        Self(value)
    }
}

impl Into<Degrees<f32>> for Radians<f32> {
    fn into(self) -> Degrees<f32> {
        Degrees(self.0 * 180f32 / PI)
    }
}

impl Into<Radians<f32>> for Degrees<f32> {
    fn into(self) -> Radians<f32> {
        Radians(self.0 * PI / 180f32)
    }
}

impl Add for Radians<f32> {
    type Output = Radians<f32>;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output::new(self.0 + rhs.0)
    }
}

impl Sub for Radians<f32> {
    type Output = Radians<f32>;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::Output::new(self.0 - rhs.0)
    }
}

impl Mul for Radians<f32> {
    type Output = Radians<f32>;

    fn mul(self, rhs: Self) -> Self::Output {
        Self::Output::new(self.0 * rhs.0)
    }
}

impl Mul<f32> for Radians<f32> {
    type Output = Radians<f32>;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::Output::new(self.0 * rhs)
    }
}

impl Div for Radians<f32> {
    type Output = Radians<f32>;

    fn div(self, rhs: Self) -> Self::Output {
        Self::Output::new(self.0 / rhs.0)
    }
}

impl Div<f32> for Radians<f32> {
    type Output = Radians<f32>;

    fn div(self, rhs: f32) -> Self::Output {
        Self::Output::new(self.0 / rhs)
    }
}

impl Rem for Radians<f32> {
    type Output = Radians<f32>;

    fn rem(self, rhs: Self) -> Self::Output {
        Self::Output::new(self.0 % rhs.0)
    }
}

impl Add for Degrees<f32> {
    type Output = Degrees<f32>;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output::new(self.0 + rhs.0)
    }
}

impl Sub for Degrees<f32> {
    type Output = Degrees<f32>;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::Output::new(self.0 - rhs.0)
    }
}

impl Mul for Degrees<f32> {
    type Output = Degrees<f32>;

    fn mul(self, rhs: Self) -> Self::Output {
        Self::Output::new(self.0 * rhs.0)
    }
}

impl Mul<f32> for Degrees<f32> {
    type Output = Degrees<f32>;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::Output::new(self.0 * rhs)
    }
}

impl Div for Degrees<f32> {
    type Output = Degrees<f32>;

    fn div(self, rhs: Self) -> Self::Output {
        Self::Output::new(self.0 / rhs.0)
    }
}

impl Div<f32> for Degrees<f32> {
    type Output = Degrees<f32>;

    fn div(self, rhs: f32) -> Self::Output {
        Self::Output::new(self.0 / rhs)
    }
}

impl Rem for Degrees<f32> {
    type Output = Degrees<f32>;

    fn rem(self, rhs: Self) -> Self::Output {
        Self::Output::new(self.0 % rhs.0)
    }
}
