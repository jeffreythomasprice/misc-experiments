use std::{error::Error, fmt::Display};

#[derive(Debug, Clone)]
pub struct U4ValueTooLarge(pub u8);

impl Display for U4ValueTooLarge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({})", std::any::type_name::<Self>(), self.0)
    }
}

impl Error for U4ValueTooLarge {}

#[derive(Debug)]
pub struct U4(pub u8);

impl U4 {
    pub fn new(value: u8) -> Result<Self, U4ValueTooLarge> {
        if value > 0xf {
            Err(U4ValueTooLarge(value))?;
        }
        Ok(Self(value))
    }
}

impl Display for U4 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:X}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct U12ValueTooLarge(pub u16);

impl Error for U12ValueTooLarge {}

impl Display for U12ValueTooLarge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({})", std::any::type_name::<Self>(), self.0)
    }
}

#[derive(Debug)]
pub struct U12(pub u16);

impl U12 {
    pub fn new(value: u16) -> Result<Self, U12ValueTooLarge> {
        if value > 0xfff {
            Err(U12ValueTooLarge(value))?;
        }
        Ok(Self(value))
    }

    pub fn from_nibbles(high: U4, mid: U4, low: U4) -> Self {
        Self(((high.0 as u16) << 8) | ((mid.0 as u16) << 4) | (low.0 as u16))
    }
}

impl Display for U12 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:03X}", self.0)
    }
}

#[derive(Debug)]
pub struct Address(pub U12);

impl Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug)]
pub struct Register(pub U4);

impl Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "V{}", self.0)
    }
}
