use std::ops::{Add, Div, Mul, Rem, Sub};

use super::numbers::{CouldBeAnAngle, Float};

#[derive(Debug, Clone, Copy)]
pub struct Radians<T>(pub T);

#[derive(Debug, Clone, Copy)]
pub struct Degrees<T>(pub T);

impl<T> Radians<T>
where
    T: Float,
{
    pub fn new(value: T) -> Self {
        Self(value)
    }
}

impl<T> Degrees<T>
where
    T: Float,
{
    pub fn new(value: T) -> Self {
        Self(value)
    }
}

impl From<Radians<f32>> for f32 {
    fn from(val: Radians<f32>) -> Self {
        val.0
    }
}

impl From<Radians<f32>> for Degrees<f32> {
    fn from(val: Radians<f32>) -> Self {
        Degrees(val.0 * 180f32 / std::f32::consts::PI)
    }
}

impl From<Radians<f64>> for f64 {
    fn from(val: Radians<f64>) -> Self {
        val.0
    }
}

impl From<Radians<f64>> for Degrees<f64> {
    fn from(val: Radians<f64>) -> Self {
        Degrees(val.0 * 180f64 / std::f64::consts::PI)
    }
}

impl From<Degrees<f32>> for f32 {
    fn from(val: Degrees<f32>) -> Self {
        val.0
    }
}

impl From<Degrees<f32>> for Radians<f32> {
    fn from(val: Degrees<f32>) -> Self {
        Radians(val.0 * std::f32::consts::PI / 180f32)
    }
}

impl From<Degrees<f64>> for f64 {
    fn from(val: Degrees<f64>) -> Self {
        val.0
    }
}

impl From<Degrees<f64>> for Radians<f64> {
    fn from(val: Degrees<f64>) -> Self {
        Radians(val.0 * std::f64::consts::PI / 180f64)
    }
}

impl CouldBeAnAngle for Radians<f32> {
    type Output = f32;

    fn cos(self) -> Self::Output {
        self.0.cos()
    }

    fn sin(self) -> Self::Output {
        self.0.sin()
    }
}

impl CouldBeAnAngle for Radians<f64> {
    type Output = f64;

    fn cos(self) -> Self::Output {
        self.0.cos()
    }

    fn sin(self) -> Self::Output {
        self.0.sin()
    }
}

impl CouldBeAnAngle for Degrees<f32> {
    type Output = f32;

    fn cos(self) -> Self::Output {
        let r: Radians<f32> = self.into();
        r.cos()
    }

    fn sin(self) -> Self::Output {
        let r: Radians<f32> = self.into();
        r.sin()
    }
}

impl CouldBeAnAngle for Degrees<f64> {
    type Output = f64;

    fn cos(self) -> Self::Output {
        let r: Radians<f64> = self.into();
        r.cos()
    }

    fn sin(self) -> Self::Output {
        let r: Radians<f64> = self.into();
        r.sin()
    }
}

impl<T> Add for Radians<T>
where
    T: Float,
{
    type Output = Radians<T>;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output::new(self.0 + rhs.0)
    }
}

impl<T> Sub for Radians<T>
where
    T: Float,
{
    type Output = Radians<T>;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::Output::new(self.0 - rhs.0)
    }
}

impl<T> Mul for Radians<T>
where
    T: Float,
{
    type Output = Radians<T>;

    fn mul(self, rhs: Self) -> Self::Output {
        Self::Output::new(self.0 * rhs.0)
    }
}

impl<T> Div for Radians<T>
where
    T: Float,
{
    type Output = Radians<T>;

    fn div(self, rhs: Self) -> Self::Output {
        Self::Output::new(self.0 / rhs.0)
    }
}

impl<T> Rem for Radians<T>
where
    T: Float,
{
    type Output = Radians<T>;

    fn rem(self, rhs: Self) -> Self::Output {
        Self::Output::new(self.0 % rhs.0)
    }
}

impl<T> Add for Degrees<T>
where
    T: Float,
{
    type Output = Degrees<T>;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output::new(self.0 + rhs.0)
    }
}

impl<T> Sub for Degrees<T>
where
    T: Float,
{
    type Output = Degrees<T>;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::Output::new(self.0 - rhs.0)
    }
}

impl<T> Mul for Degrees<T>
where
    T: Float,
{
    type Output = Degrees<T>;

    fn mul(self, rhs: Self) -> Self::Output {
        Self::Output::new(self.0 * rhs.0)
    }
}

impl<T> Div for Degrees<T>
where
    T: Float,
{
    type Output = Degrees<T>;

    fn div(self, rhs: Self) -> Self::Output {
        Self::Output::new(self.0 / rhs.0)
    }
}

impl<T> Rem for Degrees<T>
where
    T: Float,
{
    type Output = Degrees<T>;

    fn rem(self, rhs: Self) -> Self::Output {
        Self::Output::new(self.0 % rhs.0)
    }
}
