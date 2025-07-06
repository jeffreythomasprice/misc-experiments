use std::ops::Sub;

use crate::math::Vec2;

pub struct Ray2<T> {
    origin: Vec2<T>,
    delta: Vec2<T>,
}

impl<T> Ray2<T> {
    pub fn new(origin: Vec2<T>, delta: Vec2<T>) -> Self {
        Self { origin, delta }
    }

    pub fn origin(&self) -> &Vec2<T> {
        &self.origin
    }

    pub fn delta(&self) -> &Vec2<T> {
        &self.delta
    }
}

impl<T> Ray2<T>
where
    T: Sub<T, Output = T> + Clone,
{
    pub fn new_between_points(a: Vec2<T>, b: Vec2<T>) -> Self {
        Self::new(a.clone(), b - a)
    }
}
