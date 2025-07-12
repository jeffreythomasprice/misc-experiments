use std::num::TryFromIntError;

const GENERAL_PURPOSE_REGISTER_U64_0: &str = "r0";
const GENERAL_PURPOSE_REGISTER_U64_1: &str = "r1";
const GENERAL_PURPOSE_REGISTER_U64_2: &str = "r2";
const GENERAL_PURPOSE_REGISTER_U64_3: &str = "r3";
const GENERAL_PURPOSE_REGISTER_U64_4: &str = "r4";
const GENERAL_PURPOSE_REGISTER_U64_5: &str = "r5";
const GENERAL_PURPOSE_REGISTER_U64_6: &str = "r6";
const GENERAL_PURPOSE_REGISTER_U64_7: &str = "r7";

const POSITION_X_REGISTER_F64: &str = "position_x";
const POSITION_Y_REGISTER_F64: &str = "position_y";
const VELOCITY_X_REGISTER_F64: &str = "velocity_x";
const VELOCITY_Y_REGISTER_F64: &str = "velocity_y";
const TURRET_ANGLE_REGISTER_F64: &str = "turret_angle";
const TURRET_ANGULAR_VELOCITY_REGISTER_F64: &str = "turret_angular_velocity";
const SCANNER_DISTANCE_REGISTER_F64: &str = "scanner_distance";
const HEALTH_REGISTER_F64: &str = "health";
const ENERGY_REGISTER_F64: &str = "energy";
const GENERAL_PURPOSE_REGISTER_F64_0: &str = "f0";
const GENERAL_PURPOSE_REGISTER_F64_1: &str = "f1";
const GENERAL_PURPOSE_REGISTER_F64_2: &str = "f2";
const GENERAL_PURPOSE_REGISTER_F64_3: &str = "f3";
const GENERAL_PURPOSE_REGISTER_F64_4: &str = "f4";
const GENERAL_PURPOSE_REGISTER_F64_5: &str = "f5";
const GENERAL_PURPOSE_REGISTER_F64_6: &str = "f6";
const GENERAL_PURPOSE_REGISTER_F64_7: &str = "f7";

#[derive(Debug, Clone)]
pub enum ReadableRegisterU64 {
    GeneralPurpose0,
    GeneralPurpose1,
    GeneralPurpose2,
    GeneralPurpose3,
    GeneralPurpose4,
    GeneralPurpose5,
    GeneralPurpose6,
    GeneralPurpose7,
}

impl TryFrom<&str> for ReadableRegisterU64 {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            GENERAL_PURPOSE_REGISTER_U64_0 => Ok(ReadableRegisterU64::GeneralPurpose0),
            GENERAL_PURPOSE_REGISTER_U64_1 => Ok(ReadableRegisterU64::GeneralPurpose1),
            GENERAL_PURPOSE_REGISTER_U64_2 => Ok(ReadableRegisterU64::GeneralPurpose2),
            GENERAL_PURPOSE_REGISTER_U64_3 => Ok(ReadableRegisterU64::GeneralPurpose3),
            GENERAL_PURPOSE_REGISTER_U64_4 => Ok(ReadableRegisterU64::GeneralPurpose4),
            GENERAL_PURPOSE_REGISTER_U64_5 => Ok(ReadableRegisterU64::GeneralPurpose5),
            GENERAL_PURPOSE_REGISTER_U64_6 => Ok(ReadableRegisterU64::GeneralPurpose6),
            GENERAL_PURPOSE_REGISTER_U64_7 => Ok(ReadableRegisterU64::GeneralPurpose7),
            _ => Err(format!("Invalid register: {}", value)),
        }
    }
}

#[derive(Debug, Clone)]
pub enum WritableRegisterU64 {
    GeneralPurpose0,
    GeneralPurpose1,
    GeneralPurpose2,
    GeneralPurpose3,
    GeneralPurpose4,
    GeneralPurpose5,
    GeneralPurpose6,
    GeneralPurpose7,
}

impl TryFrom<&str> for WritableRegisterU64 {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            GENERAL_PURPOSE_REGISTER_U64_0 => Ok(WritableRegisterU64::GeneralPurpose0),
            GENERAL_PURPOSE_REGISTER_U64_1 => Ok(WritableRegisterU64::GeneralPurpose1),
            GENERAL_PURPOSE_REGISTER_U64_2 => Ok(WritableRegisterU64::GeneralPurpose2),
            GENERAL_PURPOSE_REGISTER_U64_3 => Ok(WritableRegisterU64::GeneralPurpose3),
            GENERAL_PURPOSE_REGISTER_U64_4 => Ok(WritableRegisterU64::GeneralPurpose4),
            GENERAL_PURPOSE_REGISTER_U64_5 => Ok(WritableRegisterU64::GeneralPurpose5),
            GENERAL_PURPOSE_REGISTER_U64_6 => Ok(WritableRegisterU64::GeneralPurpose6),
            GENERAL_PURPOSE_REGISTER_U64_7 => Ok(WritableRegisterU64::GeneralPurpose7),
            _ => Err(format!("Invalid register: {}", value)),
        }
    }
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
    GeneralPurpose0,
    GeneralPurpose1,
    GeneralPurpose2,
    GeneralPurpose3,
    GeneralPurpose4,
    GeneralPurpose5,
    GeneralPurpose6,
    GeneralPurpose7,
}

impl TryFrom<&str> for ReadableRegisterF64 {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            POSITION_X_REGISTER_F64 => Ok(ReadableRegisterF64::PositionX),
            POSITION_Y_REGISTER_F64 => Ok(ReadableRegisterF64::PositionY),
            VELOCITY_X_REGISTER_F64 => Ok(ReadableRegisterF64::VelocityX),
            VELOCITY_Y_REGISTER_F64 => Ok(ReadableRegisterF64::VelocityY),
            TURRET_ANGLE_REGISTER_F64 => Ok(ReadableRegisterF64::TurretAngle),
            TURRET_ANGULAR_VELOCITY_REGISTER_F64 => Ok(ReadableRegisterF64::TurretAngularVelocity),
            SCANNER_DISTANCE_REGISTER_F64 => Ok(ReadableRegisterF64::ScannerDistance),
            HEALTH_REGISTER_F64 => Ok(ReadableRegisterF64::Health),
            ENERGY_REGISTER_F64 => Ok(ReadableRegisterF64::Energy),
            GENERAL_PURPOSE_REGISTER_F64_0 => Ok(ReadableRegisterF64::GeneralPurpose0),
            GENERAL_PURPOSE_REGISTER_F64_1 => Ok(ReadableRegisterF64::GeneralPurpose1),
            GENERAL_PURPOSE_REGISTER_F64_2 => Ok(ReadableRegisterF64::GeneralPurpose2),
            GENERAL_PURPOSE_REGISTER_F64_3 => Ok(ReadableRegisterF64::GeneralPurpose3),
            GENERAL_PURPOSE_REGISTER_F64_4 => Ok(ReadableRegisterF64::GeneralPurpose4),
            GENERAL_PURPOSE_REGISTER_F64_5 => Ok(ReadableRegisterF64::GeneralPurpose5),
            GENERAL_PURPOSE_REGISTER_F64_6 => Ok(ReadableRegisterF64::GeneralPurpose6),
            GENERAL_PURPOSE_REGISTER_F64_7 => Ok(ReadableRegisterF64::GeneralPurpose7),
            _ => Err(format!("Invalid register: {}", value)),
        }
    }
}

#[derive(Debug, Clone)]
pub enum WritableRegisterF64 {
    VelocityX,
    VelocityY,
    TurretAngularVelocity,
    GeneralPurpose0,
    GeneralPurpose1,
    GeneralPurpose2,
    GeneralPurpose3,
    GeneralPurpose4,
    GeneralPurpose5,
    GeneralPurpose6,
    GeneralPurpose7,
}

impl TryFrom<&str> for WritableRegisterF64 {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            VELOCITY_X_REGISTER_F64 => Ok(WritableRegisterF64::VelocityX),
            VELOCITY_Y_REGISTER_F64 => Ok(WritableRegisterF64::VelocityY),
            TURRET_ANGULAR_VELOCITY_REGISTER_F64 => Ok(WritableRegisterF64::TurretAngularVelocity),
            GENERAL_PURPOSE_REGISTER_F64_0 => Ok(WritableRegisterF64::GeneralPurpose0),
            GENERAL_PURPOSE_REGISTER_F64_1 => Ok(WritableRegisterF64::GeneralPurpose1),
            GENERAL_PURPOSE_REGISTER_F64_2 => Ok(WritableRegisterF64::GeneralPurpose2),
            GENERAL_PURPOSE_REGISTER_F64_3 => Ok(WritableRegisterF64::GeneralPurpose3),
            GENERAL_PURPOSE_REGISTER_F64_4 => Ok(WritableRegisterF64::GeneralPurpose4),
            GENERAL_PURPOSE_REGISTER_F64_5 => Ok(WritableRegisterF64::GeneralPurpose5),
            GENERAL_PURPOSE_REGISTER_F64_6 => Ok(WritableRegisterF64::GeneralPurpose6),
            GENERAL_PURPOSE_REGISTER_F64_7 => Ok(WritableRegisterF64::GeneralPurpose7),
            _ => Err(format!("Invalid register: {}", value)),
        }
    }
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
pub struct ProgramPointer(pub usize);

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

#[derive(Debug, Clone)]
pub struct Program {
    instructions: Vec<Instruction>,
}

impl Program {
    pub fn new(instructions: Vec<Instruction>) -> Self {
        Self { instructions }
    }

    pub fn get(&self, p: ProgramPointer) -> Option<&Instruction> {
        self.instructions.get(p.0)
    }
}
