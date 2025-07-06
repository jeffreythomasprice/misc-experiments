use crate::math::Vec2;

pub struct Circle<T> {
    center: Vec2<T>,
    radius: T,
}

impl<T> Circle<T> {
    pub fn new(center: Vec2<T>, radius: T) -> Self {
        Self { center, radius }
    }

    pub fn center(&self) -> &Vec2<T> {
        &self.center
    }

    pub fn radius(&self) -> &T {
        &self.radius
    }
}
