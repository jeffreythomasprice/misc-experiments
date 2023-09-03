use std::{
    fmt::Display,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Rem, RemAssign, Sub, SubAssign},
};

use super::numbers::{CouldBeAnAngle, Float};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Radians<T>(pub T);

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Degrees<T>(pub T);

impl<T> Radians<T>
where
    T: Float,
{
    pub fn new(value: T) -> Self {
        Self(value)
    }

    pub fn to_degrees(self) -> Degrees<T> {
        Degrees(self.0 * T::_180 / T::PI)
    }
}

impl<T> Degrees<T>
where
    T: Float,
{
    pub fn new(value: T) -> Self {
        Self(value)
    }

    pub fn to_radians(self) -> Radians<T> {
        Radians(self.0 * T::PI / T::_180)
    }
}

impl<T> Display for Radians<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Radians({})", self.0)
    }
}

impl<T> Display for Degrees<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Degrees({})", self.0)
    }
}

impl From<Radians<f32>> for f32 {
    fn from(val: Radians<f32>) -> Self {
        val.0
    }
}

impl From<Radians<f64>> for f64 {
    fn from(val: Radians<f64>) -> Self {
        val.0
    }
}

impl<T> From<Radians<T>> for Degrees<T>
where
    T: Float,
{
    fn from(val: Radians<T>) -> Self {
        val.to_degrees()
    }
}

impl From<Degrees<f32>> for f32 {
    fn from(val: Degrees<f32>) -> Self {
        val.0
    }
}

impl From<Degrees<f64>> for f64 {
    fn from(val: Degrees<f64>) -> Self {
        val.0
    }
}

impl<T> From<Degrees<T>> for Radians<T>
where
    T: Float,
{
    fn from(val: Degrees<T>) -> Self {
        val.to_radians()
    }
}

impl<T> CouldBeAnAngle for Radians<T>
where
    T: CouldBeAnAngle<Output = T>,
{
    type Output = T;

    fn cos(self) -> Self::Output {
        self.0.cos()
    }

    fn sin(self) -> Self::Output {
        self.0.sin()
    }
}

impl<T> CouldBeAnAngle for Degrees<T>
where
    T: Float + CouldBeAnAngle<Output = T>,
    Radians<T>: From<Degrees<T>>,
{
    type Output = T;

    fn cos(self) -> Self::Output {
        self.to_radians().cos()
    }

    fn sin(self) -> Self::Output {
        self.to_radians().sin()
    }
}

impl<T> Add for Radians<T>
where
    T: Float,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output::new(self.0 + rhs.0)
    }
}

impl<T> AddAssign for Radians<T>
where
    T: Float + Copy,
{
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}

impl<T> Sub for Radians<T>
where
    T: Float,
{
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::Output::new(self.0 - rhs.0)
    }
}

impl<T> SubAssign for Radians<T>
where
    T: Float + Copy,
{
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs
    }
}

impl<T> Mul for Radians<T>
where
    T: Float,
{
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self::Output::new(self.0 * rhs.0)
    }
}

impl<T> MulAssign for Radians<T>
where
    T: Float + Copy,
{
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs
    }
}

impl<T> Div for Radians<T>
where
    T: Float,
{
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self::Output::new(self.0 / rhs.0)
    }
}

impl<T> DivAssign for Radians<T>
where
    T: Float + Copy,
{
    fn div_assign(&mut self, rhs: Self) {
        *self = *self / rhs
    }
}

impl<T> Rem for Radians<T>
where
    T: Float,
{
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        Self::Output::new(self.0 % rhs.0)
    }
}

impl<T> RemAssign for Radians<T>
where
    T: Float + Copy,
{
    fn rem_assign(&mut self, rhs: Self) {
        *self = *self % rhs
    }
}

impl<T> Neg for Radians<T>
where
    T: Float,
{
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::Output::new(-self.0)
    }
}

impl<T> Add for Degrees<T>
where
    T: Float,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output::new(self.0 + rhs.0)
    }
}

impl<T> AddAssign for Degrees<T>
where
    T: Float + Copy,
{
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}

impl<T> Sub for Degrees<T>
where
    T: Float,
{
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::Output::new(self.0 - rhs.0)
    }
}

impl<T> SubAssign for Degrees<T>
where
    T: Float + Copy,
{
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs
    }
}

impl<T> Mul for Degrees<T>
where
    T: Float,
{
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self::Output::new(self.0 * rhs.0)
    }
}

impl<T> MulAssign for Degrees<T>
where
    T: Float + Copy,
{
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs
    }
}

impl<T> Div for Degrees<T>
where
    T: Float,
{
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self::Output::new(self.0 / rhs.0)
    }
}

impl<T> DivAssign for Degrees<T>
where
    T: Float + Copy,
{
    fn div_assign(&mut self, rhs: Self) {
        *self = *self / rhs
    }
}

impl<T> Rem for Degrees<T>
where
    T: Float,
{
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        Self::Output::new(self.0 % rhs.0)
    }
}

impl<T> RemAssign for Degrees<T>
where
    T: Float + Copy,
{
    fn rem_assign(&mut self, rhs: Self) {
        *self = *self % rhs
    }
}

impl<T> Neg for Degrees<T>
where
    T: Float,
{
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::Output::new(-self.0)
    }
}
