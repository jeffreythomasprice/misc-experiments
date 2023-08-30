use std::f32::consts::PI;

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
