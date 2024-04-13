use std::{error::Error, fmt::Display};

use super::{
    types::{Address, Register, U12, U4},
    U12ValueTooLarge, U4ValueTooLarge,
};

// 0nnn - SYS addr
#[derive(Debug)]
pub struct Sys {
    pub address: Address,
}

impl Display for Sys {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SYS {}", self.address)
    }
}

// 00E0 - CLS
#[derive(Debug)]
pub struct Cls;

impl Display for Cls {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CLS")
    }
}

// 00EE - RET
#[derive(Debug)]
pub struct Ret;

impl Display for Ret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RET")
    }
}

// 1nnn - JP addr
#[derive(Debug)]
pub struct JpLiteral {
    pub address: Address,
}

impl Display for JpLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "JP {}", self.address)
    }
}

// 2nnn - CALL addr
#[derive(Debug)]
pub struct Call {
    pub address: Address,
}

impl Display for Call {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CALL {}", self.address)
    }
}

// 3xkk - SE Vx, byte
#[derive(Debug)]
pub struct SeLiteral {
    pub left: Register,
    pub right: u8,
}

impl Display for SeLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SE {}, {}", self.left, self.right)
    }
}

// 4xkk - SNE Vx, byte
#[derive(Debug)]
pub struct SneLiteral {
    pub left: Register,
    pub right: u8,
}

impl Display for SneLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SNE {}, {}", self.left, self.right)
    }
}

// 5xy0 - SE Vx, Vy
#[derive(Debug)]
pub struct SeRegister {
    pub left: Register,
    pub right: Register,
}

impl Display for SeRegister {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SE {}, {}", self.left, self.right)
    }
}

// 6xkk - LD Vx, byte
#[derive(Debug)]
pub struct LdLiteral {
    pub left: Register,
    pub right: u8,
}

impl Display for LdLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LD {}, {}", self.left, self.right)
    }
}

// 7xkk - ADD Vx, byte
#[derive(Debug)]
pub struct AddLiteral {
    pub left: Register,
    pub right: u8,
}

impl Display for AddLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ADD {}, {}", self.left, self.right)
    }
}

// 8xy0 - LD Vx, Vy
#[derive(Debug)]
pub struct LdRegister {
    pub left: Register,
    pub right: Register,
}

impl Display for LdRegister {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LD {}, {}", self.left, self.right)
    }
}

// 8xy1 - OR Vx, Vy
#[derive(Debug)]
pub struct Or {
    pub left: Register,
    pub right: Register,
}

impl Display for Or {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "OR {}, {}", self.left, self.right)
    }
}

// 8xy2 - AND Vx, Vy
#[derive(Debug)]
pub struct And {
    pub left: Register,
    pub right: Register,
}

impl Display for And {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AND {}, {}", self.left, self.right)
    }
}

// 8xy3 - XOR Vx, Vy
#[derive(Debug)]
pub struct Xor {
    pub left: Register,
    pub right: Register,
}

impl Display for Xor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "XOR {}, {}", self.left, self.right)
    }
}

// 8xy4 - ADD Vx, Vy
#[derive(Debug)]
pub struct AddRegister {
    pub left: Register,
    pub right: Register,
}

impl Display for AddRegister {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ADD {}, {}", self.left, self.right)
    }
}

// 8xy5 - SUB Vx, Vy
#[derive(Debug)]
pub struct Sub {
    pub left: Register,
    pub right: Register,
}

impl Display for Sub {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SUB {}, {}", self.left, self.right)
    }
}

// 8xy6 - SHR Vx {, Vy}
#[derive(Debug)]
pub struct Shr {
    pub register: Register,
}

impl Display for Shr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SHR {}", self.register)
    }
}

// 8xy7 - SUBN Vx, Vy
#[derive(Debug)]
pub struct Subn {
    pub left: Register,
    pub right: Register,
}

impl Display for Subn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SUBN {}, {}", self.left, self.right)
    }
}

// 8xyE - SHL Vx {, Vy}
#[derive(Debug)]
pub struct Shl {
    pub register: Register,
}

impl Display for Shl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SHL {}", self.register)
    }
}

// 9xy0 - SNE Vx, Vy
#[derive(Debug)]
pub struct SneRegister {
    pub left: Register,
    pub right: Register,
}

impl Display for SneRegister {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SNE {}, {}", self.left, self.right)
    }
}

// Annn - LD I, addr
#[derive(Debug)]
pub struct LdI {
    pub address: Address,
}

impl Display for LdI {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LD I, {}", self.address)
    }
}

// Bnnn - JP V0, addr
#[derive(Debug)]
pub struct JpV0PlusLiteral {
    pub address: Address,
}

impl Display for JpV0PlusLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "JP V0, {}", self.address)
    }
}

// Cxkk - RND Vx, byte
#[derive(Debug)]
pub struct Rnd {
    pub left: Register,
    pub right: u8,
}

impl Display for Rnd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RND {}, {}", self.left, self.right)
    }
}

// Dxyn - DRW Vx, Vy, nibble
#[derive(Debug)]
pub struct Drw {
    pub x: Register,
    pub y: Register,
    pub height: U4,
}

impl Display for Drw {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DRW {}, {}, {}", self.x, self.y, self.height)
    }
}

// Ex9E - SKP Vx
#[derive(Debug)]
pub struct Skp {
    pub register: Register,
}

impl Display for Skp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SKP {}", self.register)
    }
}

// ExA1 - SKNP Vx
#[derive(Debug)]
pub struct Sknp {
    pub register: Register,
}

impl Display for Sknp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SKNP {}", self.register)
    }
}

// Fx07 - LD Vx, DT
#[derive(Debug)]
pub struct LdRegWithDt {
    pub register: Register,
}

impl Display for LdRegWithDt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LD {}, DT", self.register)
    }
}

// Fx0A - LD Vx, K
#[derive(Debug)]
pub struct WaitKey {
    pub register: Register,
}

impl Display for WaitKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LD {}, K", self.register)
    }
}

// Fx15 - LD DT, Vx
#[derive(Debug)]
pub struct LdDtWithReg {
    pub register: Register,
}

impl Display for LdDtWithReg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LD DT, {}", self.register)
    }
}

// Fx18 - LD ST, Vx
#[derive(Debug)]
pub struct LdStWithReg {
    pub register: Register,
}

impl Display for LdStWithReg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LD ST, {}", self.register)
    }
}

// Fx1E - ADD I, Vx
#[derive(Debug)]
pub struct IncIReg {
    pub register: Register,
}

impl Display for IncIReg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ADD I, {}", self.register)
    }
}

// Fx29 - LD F, Vx
#[derive(Debug)]
pub struct LdSpriteLocation {
    pub register: Register,
}

impl Display for LdSpriteLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LD F, {}", self.register)
    }
}

// Fx33 - LD B, Vx
#[derive(Debug)]
pub struct Bcd {
    pub register: Register,
}

impl Display for Bcd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LD B, {}", self.register)
    }
}

// Fx55 - LD [I], Vx
#[derive(Debug)]
pub struct LdIWithRegs {
    pub register: Register,
}

impl Display for LdIWithRegs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LD [I], {}", self.register)
    }
}

// Fx65 - LD Vx, [I]
#[derive(Debug)]
pub struct LdRegsWithI {
    pub register: Register,
}

impl Display for LdRegsWithI {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LD {}, [I]", self.register)
    }
}

#[derive(Debug, Clone)]
pub enum InstructionParseError {
    InvalidSliceLength { received: usize, expected: usize },
    U4ValueTooLarge(U4ValueTooLarge),
    U12ValueTooLarge(U12ValueTooLarge),
}

impl From<U4ValueTooLarge> for InstructionParseError {
    fn from(value: U4ValueTooLarge) -> Self {
        Self::U4ValueTooLarge(value)
    }
}

impl From<U12ValueTooLarge> for InstructionParseError {
    fn from(value: U12ValueTooLarge) -> Self {
        Self::U12ValueTooLarge(value)
    }
}

impl Display for InstructionParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}(", std::any::type_name::<Self>())?;
        match self {
            InstructionParseError::InvalidSliceLength { received, expected } => write!(
                f,
                "invalid slice length, received {received}, expected {expected}"
            )?,
            InstructionParseError::U4ValueTooLarge(x) => write!(f, "{x}")?,
            InstructionParseError::U12ValueTooLarge(x) => write!(f, "{x}")?,
        }
        write!(f, ")")?;
        Ok(())
    }
}

impl Error for InstructionParseError {}

#[derive(Debug)]
pub enum Instruction {
    Sys(Sys),
    Cls(Cls),
    Ret(Ret),
    JpLiteral(JpLiteral),
    Call(Call),
    SeLiteral(SeLiteral),
    SneLiteral(SneLiteral),
    SeRegister(SeRegister),
    LdLiteral(LdLiteral),
    AddLiteral(AddLiteral),
    LdRegister(LdRegister),
    Or(Or),
    And(And),
    Xor(Xor),
    AddRegister(AddRegister),
    Sub(Sub),
    Shr(Shr),
    Subn(Subn),
    Shl(Shl),
    SneRegister(SneRegister),
    LdI(LdI),
    JpV0PlusLiteral(JpV0PlusLiteral),
    Rnd(Rnd),
    Drw(Drw),
    Skp(Skp),
    Sknp(Sknp),
    LdRegWithDt(LdRegWithDt),
    WaitKey(WaitKey),
    LdDtWithReg(LdDtWithReg),
    LdStWithReg(LdStWithReg),
    IncIReg(IncIReg),
    LdSpriteLocation(LdSpriteLocation),
    Bcd(Bcd),
    LdIWithRegs(LdIWithRegs),
    LdRegsWithI(LdRegsWithI),
}

impl Instruction {
    pub fn from_bytes(slice: &[u8]) -> Result<Option<Self>, InstructionParseError> {
        if slice.len() != 2 {
            Err(InstructionParseError::InvalidSliceLength {
                received: slice.len(),
                expected: 2,
            })?;
        }
        /*
        naming conventions:
        high = the most significant byte of the whole u16
        low = the least significant byte
        nib0 = nibble 0, the most significant nibble of high
        nib1 = nibble 1, the least significant nibble of high
        nib2 = nibble 2, the most significant nibble of low
        nib3 = nibble 3, the least significant nibble of low
        */
        let high = &slice[0];
        let low = &slice[1];
        Ok(
            match (
                U4::new((high & 0xf0) >> 4)?,
                U4::new(high & 0x0f)?,
                U4::new((low & 0xf0) >> 4)?,
                U4::new(low & 0x0f)?,
            ) {
                (U4(0x0), U4(0x0), U4(0xe), U4(0x0)) => Some(Self::Cls(Cls)),
                (U4(0x0), U4(0x0), U4(0xe), U4(0xe)) => Some(Self::Ret(Ret)),
                (U4(0x0), nib1, nib2, nib3) => Some(Self::Sys(Sys {
                    address: Address(U12::from_nibbles(nib1, nib2, nib3)),
                })),
                (U4(0x1), nib1, nib2, nib3) => Some(Self::JpLiteral(JpLiteral {
                    address: Address(U12::from_nibbles(nib1, nib2, nib3)),
                })),
                (U4(0x2), nib1, nib2, nib3) => Some(Self::Call(Call {
                    address: Address(U12::from_nibbles(nib1, nib2, nib3)),
                })),
                (U4(0x3), nib1, _, _) => Some(Self::SeLiteral(SeLiteral {
                    left: Register(nib1),
                    right: *low,
                })),
                (U4(0x4), nib1, _, _) => Some(Self::SneLiteral(SneLiteral {
                    left: Register(nib1),
                    right: *low,
                })),
                (U4(0x5), nib1, nib2, U4(0x0)) => Some(Self::SeRegister(SeRegister {
                    left: Register(nib1),
                    right: Register(nib2),
                })),
                (U4(0x6), nib1, _, _) => Some(Self::LdLiteral(LdLiteral {
                    left: Register(nib1),
                    right: *low,
                })),
                (U4(0x7), nib1, _, _) => Some(Self::AddLiteral(AddLiteral {
                    left: Register(nib1),
                    right: *low,
                })),
                (U4(0x8), nib1, nib2, U4(0x0)) => Some(Self::LdRegister(LdRegister {
                    left: Register(nib1),
                    right: Register(nib2),
                })),
                (U4(0x8), nib1, nib2, U4(0x1)) => Some(Self::Or(Or {
                    left: Register(nib1),
                    right: Register(nib2),
                })),
                (U4(0x8), nib1, nib2, U4(0x2)) => Some(Self::And(And {
                    left: Register(nib1),
                    right: Register(nib2),
                })),
                (U4(0x8), nib1, nib2, U4(0x3)) => Some(Self::Xor(Xor {
                    left: Register(nib1),
                    right: Register(nib2),
                })),
                (U4(0x8), nib1, nib2, U4(0x4)) => Some(Self::AddRegister(AddRegister {
                    left: Register(nib1),
                    right: Register(nib2),
                })),
                (U4(0x8), nib1, nib2, U4(0x5)) => Some(Self::Sub(Sub {
                    left: Register(nib1),
                    right: Register(nib2),
                })),
                (U4(0x8), nib1, _, U4(0x6)) => Some(Self::Shr(Shr {
                    register: Register(nib1),
                })),
                (U4(0x8), nib1, nib2, U4(0x7)) => Some(Self::Subn(Subn {
                    left: Register(nib1),
                    right: Register(nib2),
                })),
                (U4(0x8), nib1, _, U4(0xe)) => Some(Self::Shl(Shl {
                    register: Register(nib1),
                })),
                (U4(0x9), nib1, nib2, U4(0x0)) => Some(Self::SneRegister(SneRegister {
                    left: Register(nib1),
                    right: Register(nib2),
                })),
                (U4(0xa), nib1, nib2, nib3) => Some(Self::LdI(LdI {
                    address: Address(U12::from_nibbles(nib1, nib2, nib3)),
                })),
                (U4(0xb), nib1, nib2, nib3) => Some(Self::JpV0PlusLiteral(JpV0PlusLiteral {
                    address: Address(U12::from_nibbles(nib1, nib2, nib3)),
                })),
                (U4(0xc), nib1, _, _) => Some(Self::Rnd(Rnd {
                    left: Register(nib1),
                    right: *low,
                })),
                (U4(0xd), nib1, nib2, nib3) => Some(Self::Drw(Drw {
                    x: Register(nib1),
                    y: Register(nib2),
                    height: nib3,
                })),
                (U4(0xe), nib1, U4(0x9), U4(0xe)) => Some(Self::Skp(Skp {
                    register: Register(nib1),
                })),
                (U4(0xe), nib1, U4(0xa), U4(0x1)) => Some(Self::Sknp(Sknp {
                    register: Register(nib1),
                })),
                (U4(0xf), nib1, U4(0x0), U4(0x7)) => Some(Self::LdRegWithDt(LdRegWithDt {
                    register: Register(nib1),
                })),
                (U4(0xf), nib1, U4(0x0), U4(0xa)) => Some(Self::WaitKey(WaitKey {
                    register: Register(nib1),
                })),
                (U4(0xf), nib1, U4(0x1), U4(0x5)) => Some(Self::LdDtWithReg(LdDtWithReg {
                    register: Register(nib1),
                })),
                (U4(0xf), nib1, U4(0x1), U4(0x8)) => Some(Self::LdStWithReg(LdStWithReg {
                    register: Register(nib1),
                })),
                (U4(0xf), nib1, U4(0x1), U4(0xe)) => Some(Self::IncIReg(IncIReg {
                    register: Register(nib1),
                })),
                (U4(0xf), nib1, U4(0x2), U4(0x9)) => {
                    Some(Self::LdSpriteLocation(LdSpriteLocation {
                        register: Register(nib1),
                    }))
                }
                (U4(0xf), nib1, U4(0x3), U4(0x3)) => Some(Self::Bcd(Bcd {
                    register: Register(nib1),
                })),
                (U4(0xf), nib1, U4(0x5), U4(0x5)) => Some(Self::LdIWithRegs(LdIWithRegs {
                    register: Register(nib1),
                })),
                (U4(0xf), nib1, U4(0x6), U4(0x5)) => Some(Self::LdRegsWithI(LdRegsWithI {
                    register: Register(nib1),
                })),
                /*
                TODO super chip-48
                00Cn - SCD nibble
                00FB - SCR
                00FC - SCL
                00FD - EXIT
                00FE - LOW
                00FF - HIGH
                Dxy0 - DRW Vx, Vy, 0
                Fx30 - LD HF, Vx
                Fx75 - LD R, Vx
                Fx85 - LD Vx, R
                */
                _ => None,
            },
        )
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Instruction::Sys(x) => x.fmt(f),
            Instruction::Cls(x) => x.fmt(f),
            Instruction::Ret(x) => x.fmt(f),
            Instruction::JpLiteral(x) => x.fmt(f),
            Instruction::Call(x) => x.fmt(f),
            Instruction::SeLiteral(x) => x.fmt(f),
            Instruction::SneLiteral(x) => x.fmt(f),
            Instruction::SeRegister(x) => x.fmt(f),
            Instruction::LdLiteral(x) => x.fmt(f),
            Instruction::AddLiteral(x) => x.fmt(f),
            Instruction::LdRegister(x) => x.fmt(f),
            Instruction::Or(x) => x.fmt(f),
            Instruction::And(x) => x.fmt(f),
            Instruction::Xor(x) => x.fmt(f),
            Instruction::AddRegister(x) => x.fmt(f),
            Instruction::Sub(x) => x.fmt(f),
            Instruction::Shr(x) => x.fmt(f),
            Instruction::Subn(x) => x.fmt(f),
            Instruction::Shl(x) => x.fmt(f),
            Instruction::SneRegister(x) => x.fmt(f),
            Instruction::LdI(x) => x.fmt(f),
            Instruction::JpV0PlusLiteral(x) => x.fmt(f),
            Instruction::Rnd(x) => x.fmt(f),
            Instruction::Drw(x) => x.fmt(f),
            Instruction::Skp(x) => x.fmt(f),
            Instruction::Sknp(x) => x.fmt(f),
            Instruction::LdRegWithDt(x) => x.fmt(f),
            Instruction::WaitKey(x) => x.fmt(f),
            Instruction::LdDtWithReg(x) => x.fmt(f),
            Instruction::LdStWithReg(x) => x.fmt(f),
            Instruction::IncIReg(x) => x.fmt(f),
            Instruction::LdSpriteLocation(x) => x.fmt(f),
            Instruction::Bcd(x) => x.fmt(f),
            Instruction::LdIWithRegs(x) => x.fmt(f),
            Instruction::LdRegsWithI(x) => x.fmt(f),
        }
    }
}
