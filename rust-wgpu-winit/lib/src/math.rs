use std::ops::{Add, Rem, Sub};

trait Wrappable:
    Copy + Add<Output = Self> + Sub<Output = Self> + Rem<Output = Self> + PartialOrd + Sized
{
    fn zero() -> Self;
}

impl Wrappable for f32 {
    fn zero() -> Self {
        0.0
    }
}

impl Wrappable for f64 {
    fn zero() -> Self {
        0.0
    }
}

pub fn wrap<T: Wrappable>(value: T, min: T, max: T) -> T {
    let range = max - min;
    let result = value % range;
    min + if result >= T::zero() {
        result
    } else {
        range - result
    }
}
