use std::ops::{Add, Div, Mul, Neg, Rem, Sub};

pub trait BasicMath:
    Sized
    + Add<Self, Output = Self>
    + Sub<Self, Output = Self>
    + Mul<Self, Output = Self>
    + Div<Self, Output = Self>
    + Rem<Self, Output = Self>
    + Neg<Output = Self>
{
}

pub trait CouldBeAnAngle: Sized {
    type Output;

    fn cos(self) -> Self::Output;
    fn sin(self) -> Self::Output;
}

pub trait ExtraMathFunctions: Sized {
    fn sqrt(self) -> Self;
}

pub trait Float: BasicMath + CouldBeAnAngle + ExtraMathFunctions {
    const ZERO: Self;
    const ONE: Self;
}

impl BasicMath for f32 {}

impl BasicMath for f64 {}

impl Float for f32 {
    const ZERO: Self = 0f32;

    const ONE: Self = 1f32;
}

impl Float for f64 {
    const ZERO: Self = 0f64;

    const ONE: Self = 1f64;
}

impl CouldBeAnAngle for f32 {
    type Output = f32;

    fn cos(self) -> Self::Output {
        self.cos()
    }

    fn sin(self) -> Self::Output {
        self.sin()
    }
}

impl ExtraMathFunctions for f32 {
    fn sqrt(self) -> Self {
        self.sqrt()
    }
}

impl CouldBeAnAngle for f64 {
    type Output = f64;

    fn cos(self) -> Self::Output {
        self.cos()
    }

    fn sin(self) -> Self::Output {
        self.sin()
    }
}

impl ExtraMathFunctions for f64 {
    fn sqrt(self) -> Self {
        self.sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_f32_add() {
        fn f<F: Float>(a: F, b: F) -> F {
            a + b
        }
        assert_eq!(f(1f32, 2f32), 3f32);
        assert_eq!(f(1f64, 2f64), 3f64);
    }

    #[test]
    fn test_f32_sub() {
        fn f<F: Float>(a: F, b: F) -> F {
            a - b
        }
        assert_eq!(f(1f32, 2f32), -1f32);
        assert_eq!(f(1f64, 2f64), -1f64);
    }

    #[test]
    fn test_f32_mul() {
        fn f<F: Float>(a: F, b: F) -> F {
            a * b
        }
        assert_eq!(f(2f32, 3f32), 6f32);
        assert_eq!(f(2f64, 3f64), 6f64);
    }

    #[test]
    fn test_f32_div() {
        fn f<F: Float>(a: F, b: F) -> F {
            a / b
        }
        assert_eq!(f(6f32, 3f32), 2f32);
        assert_eq!(f(6f64, 3f64), 2f64);
    }

    #[test]
    fn test_f32_rem() {
        fn f<F: Float>(a: F, b: F) -> F {
            a % b
        }
        assert_eq!(f(6f32, 5f32), 1f32);
        assert_eq!(f(6f64, 5f64), 1f64);
    }

    #[test]
    fn test_f32_neg() {
        fn f<F: Float>(a: F) -> F {
            -a
        }
        assert_eq!(f(1f32), -1f32);
        assert_eq!(f(1f64), -1f64);
    }
}
