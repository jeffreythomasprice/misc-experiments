#[derive(Debug, Clone, Copy)]
pub struct Radians<T>(T);

#[derive(Debug, Clone, Copy)]
pub struct Degrees<T>(T);

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

// TODO impl into each other degrees and radians
