const NUM_REGISTERS_I32: usize = 8;
const NUM_REGISTERS_F32: usize = 8;

#[derive(Debug, Clone, Copy)]
struct RegisterIndexI32(usize);

impl TryFrom<usize> for RegisterIndexI32 {
    type Error = String;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        if value < NUM_REGISTERS_I32 {
            Ok(Self(value))
        } else {
            Err(format!("register index out of range: {}", value))
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct RegisterIndexF32(usize);

impl TryFrom<usize> for RegisterIndexF32 {
    type Error = String;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        if value < NUM_REGISTERS_F32 {
            Ok(Self(value))
        } else {
            Err(format!("register index out of range: {}", value))
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum ReadRegisterI32 {
    GeneralPurpose(RegisterIndexI32),
}

#[derive(Debug, Clone, Copy)]
enum WriteRegisterI32 {
    GeneralPurpose(RegisterIndexI32),
}

#[derive(Debug, Clone, Copy)]
enum ReadRegisterF32 {
    GeneralPurpose(RegisterIndexF32),
    LocationX,
    LocationY,
    TurretAngle,
    Health,
}

#[derive(Debug, Clone, Copy)]
enum WriteRegisterF32 {
    GeneralPurpose(RegisterIndexF32),
    VelocityX,
    VelocityY,
    TurretAngle,
    DistanceToObstacle,
}

#[derive(Debug, Clone, Copy)]
pub enum Instruction {
    Nop,
    AddRegisterI32 {
        left: ReadRegisterI32,
        right: ReadRegisterI32,
        destination: WriteRegisterI32,
    },
    AddImmediateI32 {
        left: ReadRegisterI32,
        right: i32,
        destination: WriteRegisterI32,
    },
    SubtractRegisterI32 {
        left: ReadRegisterI32,
        right: ReadRegisterI32,
        destination: WriteRegisterI32,
    },
    SubtractImmediateI32 {
        left: ReadRegisterI32,
        right: i32,
        destination: WriteRegisterI32,
    },
    MultiplyRegisterI32 {
        left: ReadRegisterI32,
        right: ReadRegisterI32,
        destination: WriteRegisterI32,
    },
    MultiplyImmediateI32 {
        left: ReadRegisterI32,
        right: i32,
        destination: WriteRegisterI32,
    },
    DivideRegisterI32 {
        left: ReadRegisterI32,
        right: ReadRegisterI32,
        destination: WriteRegisterI32,
    },
    DivideImmediateI32 {
        left: ReadRegisterI32,
        right: i32,
        destination: WriteRegisterI32,
    },
    AddRegisterF32 {
        left: ReadRegisterF32,
        right: ReadRegisterF32,
        destination: WriteRegisterF32,
    },
    AddImmediateF32 {
        left: ReadRegisterF32,
        right: f32,
        destination: WriteRegisterF32,
    },
    SubtractRegisterF32 {
        left: ReadRegisterF32,
        right: ReadRegisterF32,
        destination: WriteRegisterF32,
    },
    SubtractImmediateF32 {
        left: ReadRegisterF32,
        right: f32,
        destination: WriteRegisterF32,
    },
    MultiplyRegisterF32 {
        left: ReadRegisterF32,
        right: ReadRegisterF32,
        destination: WriteRegisterF32,
    },
    MultiplyImmediateF32 {
        left: ReadRegisterF32,
        right: f32,
        destination: WriteRegisterF32,
    },
    DivideRegisterF32 {
        left: ReadRegisterF32,
        right: ReadRegisterF32,
        destination: WriteRegisterF32,
    },
    DivideImmediateF32 {
        left: ReadRegisterF32,
        right: f32,
        destination: WriteRegisterF32,
    },
}

enum NextInstructionError {
    Halt,
}

pub struct Emulator {
    program: Vec<Instruction>,
    program_pointer: usize,

    registers_i32: [i32; NUM_REGISTERS_I32],
    registers_f32: [f32; NUM_REGISTERS_F32],

    position_x: f32,
    position_y: f32,
    velocity_x: f32,
    velocity_y: f32,
    turret_angle: f32,
    health: f32,

    clock: u64,
    halted: bool,
}

impl Emulator {
    pub fn new(program: Vec<Instruction>) -> Self {
        Self {
            program,
            program_pointer: 0,
            registers_i32: [0; NUM_REGISTERS_I32],
            registers_f32: [0.; NUM_REGISTERS_F32],
            position_x: 0.,
            position_y: 0.,
            velocity_x: 0.,
            velocity_y: 0.,
            turret_angle: 0.,
            health: 0.,
            clock: 0,
            halted: false,
        }
    }

    pub fn step(&mut self) {
        match self.get_next_instruction() {
            Ok(instruction) => match instruction {
                Instruction::Nop => {
                    self.clock += 1;
                }
                Instruction::AddRegisterI32 {
                    left,
                    right,
                    destination,
                } => todo!(),
                Instruction::AddImmediateI32 {
                    left,
                    right,
                    destination,
                } => todo!(),
                Instruction::SubtractRegisterI32 {
                    left,
                    right,
                    destination,
                } => todo!(),
                Instruction::SubtractImmediateI32 {
                    left,
                    right,
                    destination,
                } => todo!(),
                Instruction::MultiplyRegisterI32 {
                    left,
                    right,
                    destination,
                } => todo!(),
                Instruction::MultiplyImmediateI32 {
                    left,
                    right,
                    destination,
                } => todo!(),
                Instruction::DivideRegisterI32 {
                    left,
                    right,
                    destination,
                } => todo!(),
                Instruction::DivideImmediateI32 {
                    left,
                    right,
                    destination,
                } => todo!(),
                Instruction::AddRegisterF32 {
                    left,
                    right,
                    destination,
                } => todo!(),
                Instruction::AddImmediateF32 {
                    left,
                    right,
                    destination,
                } => todo!(),
                Instruction::SubtractRegisterF32 {
                    left,
                    right,
                    destination,
                } => todo!(),
                Instruction::SubtractImmediateF32 {
                    left,
                    right,
                    destination,
                } => todo!(),
                Instruction::MultiplyRegisterF32 {
                    left,
                    right,
                    destination,
                } => todo!(),
                Instruction::MultiplyImmediateF32 {
                    left,
                    right,
                    destination,
                } => todo!(),
                Instruction::DivideRegisterF32 {
                    left,
                    right,
                    destination,
                } => todo!(),
                Instruction::DivideImmediateF32 {
                    left,
                    right,
                    destination,
                } => todo!(),
            },
            Err(NextInstructionError::Halt) => {
                // TODO log
                self.halted = true;
            }
        }
    }

    fn get_next_instruction(&mut self) -> Result<&Instruction, NextInstructionError> {
        match self.program.get(self.program_pointer) {
            Some(result) => Ok(result),
            None => Err(NextInstructionError::Halt),
        }
    }
}
