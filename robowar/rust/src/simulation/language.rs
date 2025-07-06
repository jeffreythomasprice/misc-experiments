use std::num::TryFromIntError;

#[derive(Debug, Clone)]
pub enum ReadableRegisterU64 {
    GeneralPurpose1,
    GeneralPurpose2,
    GeneralPurpose3,
    GeneralPurpose4,
    GeneralPurpose5,
    GeneralPurpose6,
    GeneralPurpose7,
    GeneralPurpose8,
}

#[derive(Debug, Clone)]
pub enum WritableRegisterU64 {
    GeneralPurpose1,
    GeneralPurpose2,
    GeneralPurpose3,
    GeneralPurpose4,
    GeneralPurpose5,
    GeneralPurpose6,
    GeneralPurpose7,
    GeneralPurpose8,
}

#[derive(Debug, Clone)]
pub enum ReadableRegisterF64 {
    PositionX,
    PositionY,
    VelocityX,
    VelocityY,
    TurretAngle,
    TurretAngularVelocity,
    ScannerDistance,
    Health,
    Energy,
    GeneralPurpose1,
    GeneralPurpose2,
    GeneralPurpose3,
    GeneralPurpose4,
    GeneralPurpose5,
    GeneralPurpose6,
    GeneralPurpose7,
    GeneralPurpose8,
}

#[derive(Debug, Clone)]
pub enum WritableRegisterF64 {
    VelocityX,
    VelocityY,
    TurretAngularVelocity,
    GeneralPurpose1,
    GeneralPurpose2,
    GeneralPurpose3,
    GeneralPurpose4,
    GeneralPurpose5,
    GeneralPurpose6,
    GeneralPurpose7,
    GeneralPurpose8,
}

#[derive(Debug, Clone)]
pub enum SourceU64 {
    Register(ReadableRegisterU64),
    Literal(u64),
}

#[derive(Debug, Clone)]
pub enum DestinationU64 {
    Register(WritableRegisterU64),
}

#[derive(Debug, Clone)]
pub enum SourceF64 {
    Register(ReadableRegisterF64),
    Literal(f64),
}

#[derive(Debug, Clone)]
pub enum DestinationF64 {
    Register(WritableRegisterF64),
}

#[derive(Debug, Clone)]
pub enum Instruction {
    AddU64 {
        destination: DestinationU64,
        left: SourceU64,
        right: SourceU64,
    },
    AddF64 {
        destination: DestinationF64,
        left: SourceF64,
        right: SourceF64,
    },
    SubU64 {
        destination: DestinationU64,
        left: SourceU64,
        right: SourceU64,
    },
    SubF64 {
        destination: DestinationF64,
        left: SourceF64,
        right: SourceF64,
    },
    MulU64 {
        destination: DestinationU64,
        left: SourceU64,
        right: SourceU64,
    },
    MulF64 {
        destination: DestinationF64,
        left: SourceF64,
        right: SourceF64,
    },
    DivU64 {
        destination: DestinationU64,
        left: SourceU64,
        right: SourceU64,
    },
    DivF64 {
        destination: DestinationF64,
        left: SourceF64,
        right: SourceF64,
    },
    Jump {
        address: SourceU64,
    },
    JumpEqualU64 {
        address: SourceU64,
        left: SourceU64,
        right: SourceU64,
    },
    JumpEqualF64 {
        address: SourceU64,
        left: SourceF64,
        right: SourceF64,
    },
    JumpNotEqualU64 {
        address: SourceU64,
        left: SourceU64,
        right: SourceU64,
    },
    JumpNotEqualF64 {
        address: SourceU64,
        left: SourceF64,
        right: SourceF64,
    },
    JumpLessThanU64 {
        address: SourceU64,
        left: SourceU64,
        right: SourceU64,
    },
    JumpLessThanF64 {
        address: SourceU64,
        left: SourceF64,
        right: SourceF64,
    },
    JumpLessThanOrEqualToU64 {
        address: SourceU64,
        left: SourceU64,
        right: SourceU64,
    },
    JumpLessThanOrEqualToF64 {
        address: SourceU64,
        left: SourceF64,
        right: SourceF64,
    },
    JumpGreaterThanU64 {
        address: SourceU64,
        left: SourceU64,
        right: SourceU64,
    },
    JumpGreaterThanF64 {
        address: SourceU64,
        left: SourceF64,
        right: SourceF64,
    },
    JumpGreaterThanOrEqualToU64 {
        address: SourceU64,
        left: SourceU64,
        right: SourceU64,
    },
    JumpGreaterThanOrEqualToF64 {
        address: SourceU64,
        left: SourceF64,
        right: SourceF64,
    },
    ShiftLeft {
        destination: DestinationU64,
        source: SourceU64,
    },
    ShiftRight {
        destination: DestinationU64,
        source: SourceU64,
    },
}

#[derive(Debug, Clone, Copy)]
pub struct ProgramPointer(usize);

pub struct Program {
    instructions: Vec<Instruction>,
}

impl ProgramPointer {
    pub fn advance(&mut self) -> &mut Self {
        self.0 += 1;
        self
    }
}

impl From<usize> for ProgramPointer {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl TryFrom<u32> for ProgramPointer {
    type Error = TryFromIntError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        value.try_into().map(Self)
    }
}

impl TryFrom<u64> for ProgramPointer {
    type Error = TryFromIntError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        value.try_into().map(Self)
    }
}

impl Program {
    pub fn get(&self, p: ProgramPointer) -> Option<&Instruction> {
        self.instructions.get(p.0)
    }
}
