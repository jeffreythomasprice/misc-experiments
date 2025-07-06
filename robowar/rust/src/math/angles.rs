use std::fmt::Debug;

use crate::math::Vec2;

pub trait Angle {
    fn pi() -> Self;
    fn degrees_to_radians(&self) -> Self;
    fn radians_to_degrees(&self) -> Self;
    fn cos_of_radians(&self) -> Self;
    fn sin_of_radians(&self) -> Self;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Radians<T>(pub T);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Degrees<T>(pub T);

impl Angle for f32 {
    fn pi() -> Self {
        std::f32::consts::PI
    }

    fn degrees_to_radians(&self) -> Self {
        (*self) * Self::pi() / 180.
    }

    fn radians_to_degrees(&self) -> Self {
        (*self) * 180. / Self::pi()
    }

    fn cos_of_radians(&self) -> Self {
        f32::cos(*self)
    }

    fn sin_of_radians(&self) -> Self {
        f32::sin(*self)
    }
}

impl Angle for f64 {
    fn pi() -> Self {
        std::f64::consts::PI
    }

    fn degrees_to_radians(&self) -> Self {
        (*self) * Self::pi() / 180.
    }

    fn radians_to_degrees(&self) -> Self {
        (*self) * 180. / Self::pi()
    }

    fn cos_of_radians(&self) -> Self {
        f64::cos(*self)
    }

    fn sin_of_radians(&self) -> Self {
        f64::sin(*self)
    }
}

impl<T> Radians<T>
where
    T: Angle,
{
    pub fn from_radians(t: T) -> Self {
        Self(t)
    }

    pub fn from_degrees(t: T) -> Self {
        Self::from_radians(t.degrees_to_radians())
    }

    pub fn cos_sin_vec2(&self) -> Vec2<T> {
        Vec2::new(self.0.cos_of_radians(), self.0.sin_of_radians())
    }

    pub fn cos(&self) -> T {
        self.0.cos_of_radians()
    }

    pub fn sin(&self) -> T {
        self.0.sin_of_radians()
    }
}

impl<T> Degrees<T>
where
    T: Angle,
{
    pub fn from_radians(t: T) -> Self {
        Self::from_degrees(t.radians_to_degrees())
    }

    pub fn from_degrees(t: T) -> Self {
        Self(t)
    }

    pub fn cos_sin_vec2(&self) -> Vec2<T> {
        Vec2::new(self.0.cos_of_radians(), self.0.sin_of_radians())
    }

    pub fn cos(&self) -> T {
        self.0.cos_of_radians()
    }

    pub fn sin(&self) -> T {
        self.0.sin_of_radians()
    }
}
