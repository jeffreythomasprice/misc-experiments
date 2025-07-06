use std::fmt::Debug;

pub trait Angle {
    fn pi() -> Self;
    fn degrees_to_radians(&self) -> Self;
    fn radians_to_degrees(&self) -> Self;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Radians<T>(T);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Degrees<T>(T);

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
}

impl<T> Radians<T>
where
    T: Angle,
{
    pub fn radians(t: T) -> Self {
        Self(t)
    }

    pub fn degrees(t: T) -> Self {
        Self::radians(t.degrees_to_radians())
    }
}

impl<T> Degrees<T>
where
    T: Angle,
{
    pub fn radians(t: T) -> Self {
        Self::degrees(t.radians_to_degrees())
    }

    pub fn degrees(t: T) -> Self {
        Self(t)
    }
}
