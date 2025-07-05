use std::{collections::HashMap, str::FromStr};

#[derive(Debug)]
pub enum Register32 {
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    R8,
}

#[derive(Debug)]
pub enum Register64 {
    R12,
    R34,
    R56,
    R78,
}

#[derive(Debug)]
pub enum RegisterOrLiteral32 {
    Register(Register32),
    U32(u32),
    I32(i32),
}

#[derive(Debug)]
pub enum RegisterOrLiteral64 {
    Register(Register64),
    U64(u64),
    I64(i64),
}

#[derive(Debug)]
pub enum Instruction {
    Add32 {
        destination: Register32,
        left: RegisterOrLiteral32,
        right: RegisterOrLiteral32,
    },
    Add64 {
        destination: Register64,
        left: RegisterOrLiteral64,
        right: RegisterOrLiteral64,
    },
}

#[derive(Debug)]
pub struct Program<'a> {
    pub labels: HashMap<&'a str, usize>,
    pub instructions: Vec<Instruction>,
}

impl<'a> FromStr for Register32 {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "r1" => Ok(Register32::R1),
            "r2" => Ok(Register32::R2),
            "r3" => Ok(Register32::R3),
            "r4" => Ok(Register32::R4),
            "r5" => Ok(Register32::R5),
            "r6" => Ok(Register32::R6),
            "r7" => Ok(Register32::R7),
            "r8" => Ok(Register32::R8),
            _ => Err(format!("not a valid 32-bit register: {}", s)),
        }
    }
}

impl<'a> FromStr for Register64 {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "r12" => Ok(Register64::R12),
            "r34" => Ok(Register64::R34),
            "r56" => Ok(Register64::R56),
            "r78" => Ok(Register64::R78),
            _ => Err(format!("not a valid 64-bit register: {}", s)),
        }
    }
}
