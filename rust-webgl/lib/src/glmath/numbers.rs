use std::ops::{
    Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Rem, RemAssign, Sub, SubAssign,
};

pub trait BasicMath:
    Sized
    + Add<Output = Self>
    + AddAssign
    + Sub<Output = Self>
    + SubAssign
    + Mul<Output = Self>
    + MulAssign
    + Div<Output = Self>
    + DivAssign
    + Rem<Output = Self>
    + RemAssign
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
    fn tan(self) -> Self;
}

pub trait Float: BasicMath + CouldBeAnAngle + ExtraMathFunctions {
    const ZERO: Self;
    const ONE: Self;
    const TWO: Self;
    const _180: Self;
    const PI: Self;
    const FRAC_1_PI: Self;
    const FRAC_2_PI: Self;
    const FRAC_PI_2: Self;
    const FRAC_PI_3: Self;
    const FRAC_PI_4: Self;
    const FRAC_PI_6: Self;
    const FRAC_PI_8: Self;
    const FRAC_2_SQRT_PI: Self;
}

impl BasicMath for f32 {}

impl BasicMath for f64 {}

impl Float for f32 {
    const ZERO: Self = 0f32;

    const ONE: Self = 1f32;

    const TWO: Self = 2f32;

    const _180: Self = 180f32;

    const PI: Self = std::f32::consts::PI;

    const FRAC_1_PI: Self = std::f32::consts::FRAC_1_PI;

    const FRAC_2_PI: Self = std::f32::consts::FRAC_2_PI;

    const FRAC_PI_2: Self = std::f32::consts::FRAC_PI_2;

    const FRAC_PI_3: Self = std::f32::consts::FRAC_PI_3;

    const FRAC_PI_4: Self = std::f32::consts::FRAC_PI_4;

    const FRAC_PI_6: Self = std::f32::consts::FRAC_PI_6;

    const FRAC_PI_8: Self = std::f32::consts::FRAC_PI_8;

    const FRAC_2_SQRT_PI: Self = std::f32::consts::FRAC_2_SQRT_PI;
}

impl Float for f64 {
    const ZERO: Self = 0f64;

    const ONE: Self = 1f64;

    const TWO: Self = 2f64;

    const _180: Self = 180f64;

    const PI: Self = std::f64::consts::PI;

    const FRAC_1_PI: Self = std::f64::consts::FRAC_1_PI;

    const FRAC_2_PI: Self = std::f64::consts::FRAC_2_PI;

    const FRAC_PI_2: Self = std::f64::consts::FRAC_PI_2;

    const FRAC_PI_3: Self = std::f64::consts::FRAC_PI_3;

    const FRAC_PI_4: Self = std::f64::consts::FRAC_PI_4;

    const FRAC_PI_6: Self = std::f64::consts::FRAC_PI_6;

    const FRAC_PI_8: Self = std::f64::consts::FRAC_PI_8;

    const FRAC_2_SQRT_PI: Self = std::f64::consts::FRAC_2_SQRT_PI;
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

    fn tan(self) -> Self {
        self.tan()
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

    fn tan(self) -> Self {
        self.tan()
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

        let mut f = 1f32;
        f += 2f32;
        assert_eq!(f, 3f32);

        let mut f = 1f64;
        f += 2f64;
        assert_eq!(f, 3f64);
    }

    #[test]
    fn test_f32_sub() {
        fn f<F: Float>(a: F, b: F) -> F {
            a - b
        }
        assert_eq!(f(1f32, 2f32), -1f32);
        assert_eq!(f(1f64, 2f64), -1f64);

        let mut f = 1f32;
        f -= 2f32;
        assert_eq!(f, -1f32);

        let mut f = 1f64;
        f -= 2f64;
        assert_eq!(f, -1f64);
    }

    #[test]
    fn test_f32_mul() {
        fn f<F: Float>(a: F, b: F) -> F {
            a * b
        }
        assert_eq!(f(2f32, 3f32), 6f32);
        assert_eq!(f(2f64, 3f64), 6f64);

        let mut f = 2f32;
        f *= 3f32;
        assert_eq!(f, 6f32);

        let mut f = 2f64;
        f *= 3f64;
        assert_eq!(f, 6f64);
    }

    #[test]
    fn test_f32_div() {
        fn f<F: Float>(a: F, b: F) -> F {
            a / b
        }
        assert_eq!(f(6f32, 3f32), 2f32);
        assert_eq!(f(6f64, 3f64), 2f64);

        let mut f = 6f32;
        f /= 3f32;
        assert_eq!(f, 2f32);

        let mut f = 6f64;
        f /= 3f64;
        assert_eq!(f, 2f64);
    }

    #[test]
    fn test_f32_rem() {
        fn f<F: Float>(a: F, b: F) -> F {
            a % b
        }
        assert_eq!(f(6f32, 5f32), 1f32);
        assert_eq!(f(6f64, 5f64), 1f64);

        let mut f = 6f32;
        f %= 5f32;
        assert_eq!(f, 1f32);

        let mut f = 6f64;
        f %= 5f64;
        assert_eq!(f, 1f64);
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
