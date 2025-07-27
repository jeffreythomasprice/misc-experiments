mod basic_types;
mod compile_time_expression;

use crate::{assembler::compile_time_expression::compile_time_expression, simulation::language};
use basic_types::*;
use chumsky::prelude::*;
use std::collections::HashMap;

// TODO custom error types for everything here, no Result<_, String>

#[derive(Debug, Clone)]
enum Argument {
    Identifier(String),
    Number(NumberLiteral),
}

impl TryInto<language::SourceU64> for Argument {
    type Error = String;

    fn try_into(self) -> Result<language::SourceU64, Self::Error> {
        match self {
            Argument::Identifier(s) => Ok(language::SourceU64::Register(s.as_str().try_into()?)),
            Argument::Number(NumberLiteral::U64(result)) => {
                Ok(language::SourceU64::Literal(result))
            }
            Argument::Number(NumberLiteral::I64(result)) => {
                Ok(language::SourceU64::Literal(result as u64))
            }
            Argument::Number(number) => Err(format!("expected a u64 literal, found: {number:?}")),
        }
    }
}

impl TryInto<language::DestinationU64> for Argument {
    type Error = String;

    fn try_into(self) -> Result<language::DestinationU64, Self::Error> {
        match self {
            Argument::Identifier(s) => {
                Ok(language::DestinationU64::Register(s.as_str().try_into()?))
            }
            Argument::Number(number) => Err(format!(
                "expected a writable u64 register, found: {number:?}"
            )),
        }
    }
}

impl TryInto<language::SourceF64> for Argument {
    type Error = String;

    fn try_into(self) -> Result<language::SourceF64, Self::Error> {
        match self {
            Argument::Identifier(s) => Ok(language::SourceF64::Register(s.as_str().try_into()?)),
            Argument::Number(NumberLiteral::F64(result)) => {
                Ok(language::SourceF64::Literal(result))
            }
            Argument::Number(NumberLiteral::I64(result)) => {
                Ok(language::SourceF64::Literal(result as f64))
            }
            Argument::Number(NumberLiteral::U64(result)) => {
                Ok(language::SourceF64::Literal(result as f64))
            }
        }
    }
}

impl TryInto<language::DestinationF64> for Argument {
    type Error = String;

    fn try_into(self) -> Result<language::DestinationF64, Self::Error> {
        match self {
            Argument::Identifier(s) => {
                Ok(language::DestinationF64::Register(s.as_str().try_into()?))
            }
            Argument::Number(number) => Err(format!(
                "expected a writable f64 register, found: {number:?}"
            )),
        }
    }
}

#[derive(Debug, Clone)]
pub enum SourceU64 {
    Register(language::ReadableRegisterU64),
    Literal(u64),
    Label(String),
}

impl SourceU64 {
    fn into_runnable(
        &self,
        values: &HashMap<String, NumberLiteral>,
    ) -> Result<language::SourceU64, String> {
        match self {
            SourceU64::Register(register) => Ok(language::SourceU64::Register(register.clone())),
            SourceU64::Literal(literal) => Ok(language::SourceU64::Literal(*literal)),
            SourceU64::Label(label) => match values.get(label) {
                Some(NumberLiteral::U64(value)) => Ok(language::SourceU64::Literal(*value)),
                Some(NumberLiteral::I64(value)) => Ok(language::SourceU64::Literal(*value as u64)),
                Some(NumberLiteral::F64(_)) => Err(format!("{label} is an f64, expected u64")),
                None => Err(format!("label '{label}' not found in program")),
            },
        }
    }
}

impl TryInto<SourceU64> for Argument {
    type Error = String;

    fn try_into(self) -> Result<SourceU64, Self::Error> {
        match self {
            Argument::Identifier(s) => match s.as_str().try_into() {
                Ok(register) => Ok(SourceU64::Register(register)),
                Result::Err(_) => Ok(SourceU64::Label(s)),
            },
            Argument::Number(NumberLiteral::U64(result)) => Ok(SourceU64::Literal(result)),
            Argument::Number(NumberLiteral::I64(result)) => Ok(SourceU64::Literal(result as u64)),
            Argument::Number(number) => Err(format!("expected a u64 literal, found: {number:?}")),
        }
    }
}

#[derive(Debug, Clone)]
pub enum SourceF64 {
    Register(language::ReadableRegisterF64),
    Literal(f64),
    Label(String),
}

impl SourceF64 {
    fn into_runnable(
        &self,
        values: &HashMap<String, NumberLiteral>,
    ) -> Result<language::SourceF64, String> {
        match self {
            SourceF64::Register(register) => Ok(language::SourceF64::Register(register.clone())),
            SourceF64::Literal(literal) => Ok(language::SourceF64::Literal(*literal)),
            SourceF64::Label(label) => match values.get(label) {
                Some(NumberLiteral::U64(value)) => Ok(language::SourceF64::Literal(*value as f64)),
                Some(NumberLiteral::I64(value)) => Ok(language::SourceF64::Literal(*value as f64)),
                Some(NumberLiteral::F64(value)) => Ok(language::SourceF64::Literal(*value)),
                None => Err(format!("label '{label}' not found in program")),
            },
        }
    }
}

impl TryInto<SourceF64> for Argument {
    type Error = String;

    fn try_into(self) -> Result<SourceF64, Self::Error> {
        match self {
            Argument::Identifier(s) => match s.as_str().try_into() {
                Ok(register) => Ok(SourceF64::Register(register)),
                Result::Err(_) => Ok(SourceF64::Label(s)),
            },
            Argument::Number(NumberLiteral::U64(result)) => Ok(SourceF64::Literal(result as f64)),
            Argument::Number(NumberLiteral::I64(result)) => Ok(SourceF64::Literal(result as f64)),
            Argument::Number(NumberLiteral::F64(result)) => Ok(SourceF64::Literal(result)),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Instruction {
    SetU64 {
        destination: language::DestinationU64,
        source: SourceU64,
    },
    SetF64 {
        destination: language::DestinationF64,
        source: SourceF64,
    },
    AddU64 {
        destination: language::DestinationU64,
        left: SourceU64,
        right: SourceU64,
    },
    AddF64 {
        destination: language::DestinationF64,
        left: SourceF64,
        right: SourceF64,
    },
    SubU64 {
        destination: language::DestinationU64,
        left: SourceU64,
        right: SourceU64,
    },
    SubF64 {
        destination: language::DestinationF64,
        left: SourceF64,
        right: SourceF64,
    },
    MulU64 {
        destination: language::DestinationU64,
        left: SourceU64,
        right: SourceU64,
    },
    MulF64 {
        destination: language::DestinationF64,
        left: SourceF64,
        right: SourceF64,
    },
    DivU64 {
        destination: language::DestinationU64,
        left: SourceU64,
        right: SourceU64,
    },
    DivF64 {
        destination: language::DestinationF64,
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
        destination: language::DestinationU64,
        source: SourceU64,
    },
    ShiftRight {
        destination: language::DestinationU64,
        source: SourceU64,
    },
    PushU64 {
        source: SourceU64,
    },
    PushF64 {
        source: SourceF64,
    },
    PopU64 {
        destination: language::DestinationU64,
    },
    PopF64 {
        destination: language::DestinationF64,
    },
    LoadU64 {
        destination: language::DestinationU64,
        source_address: SourceU64,
    },
    LoadF64 {
        destination: language::DestinationF64,
        source_address: SourceU64,
    },
    StoreU64 {
        destination_address: SourceU64,
        source: SourceU64,
    },
    StoreF64 {
        destination_address: SourceU64,
        source: SourceF64,
    },
}

impl Instruction {
    fn new(instruction: String, arguments: Vec<Argument>) -> Result<Self, String> {
        match instruction.to_lowercase().as_str() {
            "set" => match arguments.as_slice() {
                [destination, source] => {
                    if let Ok(destination) = destination.clone().try_into()
                        && let Ok(source) = source.clone().try_into()
                    {
                        Ok(Instruction::SetU64 {
                            destination,
                            source,
                        })
                    } else if let Ok(destination) = destination.clone().try_into()
                        && let Ok(source) = source.clone().try_into()
                    {
                        Ok(Instruction::SetF64 {
                            destination,
                            source,
                        })
                    } else {
                        Err(format!(
                            "invalid arguments for '{}': destination: {:?}, source: {:?}",
                            instruction.to_uppercase(),
                            destination,
                            source
                        ))
                    }
                }
                _ => Err(format!(
                    "expected 2 arguments for '{}', found {}",
                    instruction.to_uppercase(),
                    arguments.len()
                )),
            },
            "add" => match arguments.as_slice() {
                [destination, left, right] => {
                    if let Ok(destination) = destination.clone().try_into()
                        && let Ok(left) = left.clone().try_into()
                        && let Ok(right) = right.clone().try_into()
                    {
                        Ok(Instruction::AddU64 {
                            destination,
                            left,
                            right,
                        })
                    } else if let Ok(destination) = destination.clone().try_into()
                        && let Ok(left) = left.clone().try_into()
                        && let Ok(right) = right.clone().try_into()
                    {
                        Ok(Instruction::AddF64 {
                            destination,
                            left,
                            right,
                        })
                    } else {
                        Err(format!(
                            "invalid arguments for '{}': destination: {:?}, left: {:?}, right: {:?}",
                            instruction.to_uppercase(),
                            destination,
                            left,
                            right
                        ))
                    }
                }
                _ => Err(format!(
                    "expected 3 arguments for '{}', found {}",
                    instruction.to_uppercase(),
                    arguments.len()
                )),
            },

            "sub" => match arguments.as_slice() {
                [destination, left, right] => {
                    if let Ok(destination) = destination.clone().try_into()
                        && let Ok(left) = left.clone().try_into()
                        && let Ok(right) = right.clone().try_into()
                    {
                        Ok(Instruction::SubU64 {
                            destination,
                            left,
                            right,
                        })
                    } else if let Ok(destination) = destination.clone().try_into()
                        && let Ok(left) = left.clone().try_into()
                        && let Ok(right) = right.clone().try_into()
                    {
                        Ok(Instruction::SubF64 {
                            destination,
                            left,
                            right,
                        })
                    } else {
                        Err(format!(
                            "invalid arguments for '{}': destination: {:?}, left: {:?}, right: {:?}",
                            instruction.to_uppercase(),
                            destination,
                            left,
                            right
                        ))
                    }
                }
                _ => Err(format!(
                    "expected 3 arguments for '{}', found {}",
                    instruction.to_uppercase(),
                    arguments.len()
                )),
            },

            "mul" => match arguments.as_slice() {
                [destination, left, right] => {
                    if let Ok(destination) = destination.clone().try_into()
                        && let Ok(left) = left.clone().try_into()
                        && let Ok(right) = right.clone().try_into()
                    {
                        Ok(Instruction::MulU64 {
                            destination,
                            left,
                            right,
                        })
                    } else if let Ok(destination) = destination.clone().try_into()
                        && let Ok(left) = left.clone().try_into()
                        && let Ok(right) = right.clone().try_into()
                    {
                        Ok(Instruction::MulF64 {
                            destination,
                            left,
                            right,
                        })
                    } else {
                        Err(format!(
                            "invalid arguments for '{}': destination: {:?}, left: {:?}, right: {:?}",
                            instruction.to_uppercase(),
                            destination,
                            left,
                            right
                        ))
                    }
                }
                _ => Err(format!(
                    "expected 3 arguments for '{}', found {}",
                    instruction.to_uppercase(),
                    arguments.len()
                )),
            },

            "div" => match arguments.as_slice() {
                [destination, left, right] => {
                    if let Ok(destination) = destination.clone().try_into()
                        && let Ok(left) = left.clone().try_into()
                        && let Ok(right) = right.clone().try_into()
                    {
                        Ok(Instruction::DivU64 {
                            destination,
                            left,
                            right,
                        })
                    } else if let Ok(destination) = destination.clone().try_into()
                        && let Ok(left) = left.clone().try_into()
                        && let Ok(right) = right.clone().try_into()
                    {
                        Ok(Instruction::DivF64 {
                            destination,
                            left,
                            right,
                        })
                    } else {
                        Err(format!(
                            "invalid arguments for '{}': destination: {:?}, left: {:?}, right: {:?}",
                            instruction.to_uppercase(),
                            destination,
                            left,
                            right
                        ))
                    }
                }
                _ => Err(format!(
                    "expected 3 arguments for '{}', found {}",
                    instruction.to_uppercase(),
                    arguments.len()
                )),
            },

            "jmp" => match arguments.as_slice() {
                [address] => {
                    if let Ok(address) = address.clone().try_into() {
                        Ok(Instruction::Jump { address })
                    } else {
                        Err(format!(
                            "invalid argument for '{}': address: {:?}",
                            instruction.to_uppercase(),
                            address
                        ))
                    }
                }
                _ => Err(format!(
                    "expected 1 argument for '{}', found {}",
                    instruction.to_uppercase(),
                    arguments.len()
                )),
            },

            "jeq" => match arguments.as_slice() {
                [address, left, right] => {
                    if let Ok(address) = address.clone().try_into()
                        && let Ok(left) = left.clone().try_into()
                        && let Ok(right) = right.clone().try_into()
                    {
                        Ok(Instruction::JumpEqualU64 {
                            address,
                            left,
                            right,
                        })
                    } else if let Ok(address) = address.clone().try_into()
                        && let Ok(left) = left.clone().try_into()
                        && let Ok(right) = right.clone().try_into()
                    {
                        Ok(Instruction::JumpEqualF64 {
                            address,
                            left,
                            right,
                        })
                    } else {
                        Err(format!(
                            "invalid arguments for '{}': address: {:?}, left: {:?}, right: {:?}",
                            instruction.to_uppercase(),
                            address,
                            left,
                            right
                        ))
                    }
                }
                _ => Err(format!(
                    "expected 3 arguments for '{}', found {}",
                    instruction.to_uppercase(),
                    arguments.len()
                )),
            },

            "jne" => match arguments.as_slice() {
                [address, left, right] => {
                    if let Ok(address) = address.clone().try_into()
                        && let Ok(left) = left.clone().try_into()
                        && let Ok(right) = right.clone().try_into()
                    {
                        Ok(Instruction::JumpNotEqualU64 {
                            address,
                            left,
                            right,
                        })
                    } else if let Ok(address) = address.clone().try_into()
                        && let Ok(left) = left.clone().try_into()
                        && let Ok(right) = right.clone().try_into()
                    {
                        Ok(Instruction::JumpNotEqualF64 {
                            address,
                            left,
                            right,
                        })
                    } else {
                        Err(format!(
                            "invalid arguments for '{}': address: {:?}, left: {:?}, right: {:?}",
                            instruction.to_uppercase(),
                            address,
                            left,
                            right
                        ))
                    }
                }
                _ => Err(format!(
                    "expected 3 arguments for '{}', found {}",
                    instruction.to_uppercase(),
                    arguments.len()
                )),
            },

            "jlt" => match arguments.as_slice() {
                [address, left, right] => {
                    if let Ok(address) = address.clone().try_into()
                        && let Ok(left) = left.clone().try_into()
                        && let Ok(right) = right.clone().try_into()
                    {
                        Ok(Instruction::JumpLessThanU64 {
                            address,
                            left,
                            right,
                        })
                    } else if let Ok(address) = address.clone().try_into()
                        && let Ok(left) = left.clone().try_into()
                        && let Ok(right) = right.clone().try_into()
                    {
                        Ok(Instruction::JumpLessThanF64 {
                            address,
                            left,
                            right,
                        })
                    } else {
                        Err(format!(
                            "invalid arguments for '{}': address: {:?}, left: {:?}, right: {:?}",
                            instruction.to_uppercase(),
                            address,
                            left,
                            right
                        ))
                    }
                }
                _ => Err(format!(
                    "expected 3 arguments for '{}', found {}",
                    instruction.to_uppercase(),
                    arguments.len()
                )),
            },

            "jle" => match arguments.as_slice() {
                [address, left, right] => {
                    if let Ok(address) = address.clone().try_into()
                        && let Ok(left) = left.clone().try_into()
                        && let Ok(right) = right.clone().try_into()
                    {
                        Ok(Instruction::JumpLessThanOrEqualToU64 {
                            address,
                            left,
                            right,
                        })
                    } else if let Ok(address) = address.clone().try_into()
                        && let Ok(left) = left.clone().try_into()
                        && let Ok(right) = right.clone().try_into()
                    {
                        Ok(Instruction::JumpLessThanOrEqualToF64 {
                            address,
                            left,
                            right,
                        })
                    } else {
                        Err(format!(
                            "invalid arguments for '{}': address: {:?}, left: {:?}, right: {:?}",
                            instruction.to_uppercase(),
                            address,
                            left,
                            right
                        ))
                    }
                }
                _ => Err(format!(
                    "expected 3 arguments for '{}', found {}",
                    instruction.to_uppercase(),
                    arguments.len()
                )),
            },

            "jgt" => match arguments.as_slice() {
                [address, left, right] => {
                    if let Ok(address) = address.clone().try_into()
                        && let Ok(left) = left.clone().try_into()
                        && let Ok(right) = right.clone().try_into()
                    {
                        Ok(Instruction::JumpGreaterThanU64 {
                            address,
                            left,
                            right,
                        })
                    } else if let Ok(address) = address.clone().try_into()
                        && let Ok(left) = left.clone().try_into()
                        && let Ok(right) = right.clone().try_into()
                    {
                        Ok(Instruction::JumpGreaterThanF64 {
                            address,
                            left,
                            right,
                        })
                    } else {
                        Err(format!(
                            "invalid arguments for '{}': address: {:?}, left: {:?}, right: {:?}",
                            instruction.to_uppercase(),
                            address,
                            left,
                            right
                        ))
                    }
                }
                _ => Err(format!(
                    "expected 3 arguments for '{}', found {}",
                    instruction.to_uppercase(),
                    arguments.len()
                )),
            },

            "jge" => match arguments.as_slice() {
                [address, left, right] => {
                    if let Ok(address) = address.clone().try_into()
                        && let Ok(left) = left.clone().try_into()
                        && let Ok(right) = right.clone().try_into()
                    {
                        Ok(Instruction::JumpGreaterThanOrEqualToU64 {
                            address,
                            left,
                            right,
                        })
                    } else if let Ok(address) = address.clone().try_into()
                        && let Ok(left) = left.clone().try_into()
                        && let Ok(right) = right.clone().try_into()
                    {
                        Ok(Instruction::JumpGreaterThanOrEqualToF64 {
                            address,
                            left,
                            right,
                        })
                    } else {
                        Err(format!(
                            "invalid arguments for '{}': address: {:?}, left: {:?}, right: {:?}",
                            instruction.to_uppercase(),
                            address,
                            left,
                            right
                        ))
                    }
                }
                _ => Err(format!(
                    "expected 3 arguments for '{}', found {}",
                    instruction.to_uppercase(),
                    arguments.len()
                )),
            },

            "sl" => match arguments.as_slice() {
                [destination, source] => {
                    if let Ok(destination) = destination.clone().try_into()
                        && let Ok(source) = source.clone().try_into()
                    {
                        Ok(Instruction::ShiftLeft {
                            destination,
                            source,
                        })
                    } else {
                        Err(format!(
                            "invalid arguments for '{}': destination: {:?}, source: {:?}",
                            instruction.to_uppercase(),
                            destination,
                            source
                        ))
                    }
                }
                _ => Err(format!(
                    "expected 2 arguments for '{}', found {}",
                    instruction.to_uppercase(),
                    arguments.len()
                )),
            },

            "sr" => match arguments.as_slice() {
                [destination, source] => {
                    if let Ok(destination) = destination.clone().try_into()
                        && let Ok(source) = source.clone().try_into()
                    {
                        Ok(Instruction::ShiftRight {
                            destination,
                            source,
                        })
                    } else {
                        Err(format!(
                            "invalid arguments for '{}': destination: {:?}, source: {:?}",
                            instruction.to_uppercase(),
                            destination,
                            source
                        ))
                    }
                }
                _ => Err(format!(
                    "expected 2 arguments for '{}', found {}",
                    instruction.to_uppercase(),
                    arguments.len()
                )),
            },

            "push" => todo!(),

            "pop" => todo!(),

            "load" => todo!(),

            "store" => todo!(),

            _ => Err(format!("unrecognized instruction: {instruction}")),
        }
    }

    fn into_runnable(
        &self,
        values: &HashMap<String, NumberLiteral>,
    ) -> Result<language::Instruction, String> {
        match self {
            Instruction::SetU64 {
                destination,
                source,
            } => Ok(language::Instruction::SetU64 {
                destination: destination.clone(),
                source: source.clone().into_runnable(values)?,
            }),
            Instruction::SetF64 {
                destination,
                source,
            } => Ok(language::Instruction::SetF64 {
                destination: destination.clone(),
                source: source.clone().into_runnable(values)?,
            }),
            Instruction::AddU64 {
                destination,
                left,
                right,
            } => Ok(language::Instruction::AddU64 {
                destination: destination.clone(),
                left: left.into_runnable(values)?,
                right: right.into_runnable(values)?,
            }),
            Instruction::AddF64 {
                destination,
                left,
                right,
            } => Ok(language::Instruction::AddF64 {
                destination: destination.clone(),
                left: left.clone().into_runnable(values)?,
                right: right.clone().into_runnable(values)?,
            }),
            Instruction::SubU64 {
                destination,
                left,
                right,
            } => Ok(language::Instruction::SubU64 {
                destination: destination.clone(),
                left: left.into_runnable(values)?,
                right: right.into_runnable(values)?,
            }),
            Instruction::SubF64 {
                destination,
                left,
                right,
            } => Ok(language::Instruction::SubF64 {
                destination: destination.clone(),
                left: left.clone().into_runnable(values)?,
                right: right.clone().into_runnable(values)?,
            }),
            Instruction::MulU64 {
                destination,
                left,
                right,
            } => Ok(language::Instruction::MulU64 {
                destination: destination.clone(),
                left: left.into_runnable(values)?,
                right: right.into_runnable(values)?,
            }),
            Instruction::MulF64 {
                destination,
                left,
                right,
            } => Ok(language::Instruction::MulF64 {
                destination: destination.clone(),
                left: left.clone().into_runnable(values)?,
                right: right.clone().into_runnable(values)?,
            }),
            Instruction::DivU64 {
                destination,
                left,
                right,
            } => Ok(language::Instruction::DivU64 {
                destination: destination.clone(),
                left: left.into_runnable(values)?,
                right: right.into_runnable(values)?,
            }),
            Instruction::DivF64 {
                destination,
                left,
                right,
            } => Ok(language::Instruction::DivF64 {
                destination: destination.clone(),
                left: left.clone().into_runnable(values)?,
                right: right.clone().into_runnable(values)?,
            }),
            Instruction::Jump { address } => Ok(language::Instruction::Jump {
                address: address.clone().into_runnable(values)?,
            }),
            Instruction::JumpEqualU64 {
                address,
                left,
                right,
            } => Ok(language::Instruction::JumpEqualU64 {
                address: address.clone().into_runnable(values)?,
                left: left.into_runnable(values)?,
                right: right.into_runnable(values)?,
            }),
            Instruction::JumpEqualF64 {
                address,
                left,
                right,
            } => Ok(language::Instruction::JumpEqualF64 {
                address: address.clone().into_runnable(values)?,
                left: left.clone().into_runnable(values)?,
                right: right.clone().into_runnable(values)?,
            }),
            Instruction::JumpNotEqualU64 {
                address,
                left,
                right,
            } => Ok(language::Instruction::JumpNotEqualU64 {
                address: address.clone().into_runnable(values)?,
                left: left.into_runnable(values)?,
                right: right.into_runnable(values)?,
            }),
            Instruction::JumpNotEqualF64 {
                address,
                left,
                right,
            } => Ok(language::Instruction::JumpNotEqualF64 {
                address: address.clone().into_runnable(values)?,
                left: left.clone().into_runnable(values)?,
                right: right.clone().into_runnable(values)?,
            }),
            Instruction::JumpLessThanU64 {
                address,
                left,
                right,
            } => Ok(language::Instruction::JumpLessThanU64 {
                address: address.clone().into_runnable(values)?,
                left: left.into_runnable(values)?,
                right: right.into_runnable(values)?,
            }),
            Instruction::JumpLessThanF64 {
                address,
                left,
                right,
            } => Ok(language::Instruction::JumpLessThanF64 {
                address: address.clone().into_runnable(values)?,
                left: left.clone().into_runnable(values)?,
                right: right.clone().into_runnable(values)?,
            }),
            Instruction::JumpLessThanOrEqualToU64 {
                address,
                left,
                right,
            } => Ok(language::Instruction::JumpLessThanOrEqualToU64 {
                address: address.clone().into_runnable(values)?,
                left: left.into_runnable(values)?,
                right: right.into_runnable(values)?,
            }),
            Instruction::JumpLessThanOrEqualToF64 {
                address,
                left,
                right,
            } => Ok(language::Instruction::JumpLessThanOrEqualToF64 {
                address: address.clone().into_runnable(values)?,
                left: left.clone().into_runnable(values)?,
                right: right.clone().into_runnable(values)?,
            }),
            Instruction::JumpGreaterThanU64 {
                address,
                left,
                right,
            } => Ok(language::Instruction::JumpGreaterThanU64 {
                address: address.clone().into_runnable(values)?,
                left: left.into_runnable(values)?,
                right: right.into_runnable(values)?,
            }),
            Instruction::JumpGreaterThanF64 {
                address,
                left,
                right,
            } => Ok(language::Instruction::JumpGreaterThanF64 {
                address: address.clone().into_runnable(values)?,
                left: left.clone().into_runnable(values)?,
                right: right.clone().into_runnable(values)?,
            }),
            Instruction::JumpGreaterThanOrEqualToU64 {
                address,
                left,
                right,
            } => Ok(language::Instruction::JumpGreaterThanOrEqualToU64 {
                address: address.clone().into_runnable(values)?,
                left: left.into_runnable(values)?,
                right: right.into_runnable(values)?,
            }),
            Instruction::JumpGreaterThanOrEqualToF64 {
                address,
                left,
                right,
            } => Ok(language::Instruction::JumpGreaterThanOrEqualToF64 {
                address: address.clone().into_runnable(values)?,
                left: left.clone().into_runnable(values)?,
                right: right.clone().into_runnable(values)?,
            }),
            Instruction::ShiftLeft {
                destination,
                source,
            } => Ok(language::Instruction::ShiftLeft {
                destination: destination.clone(),
                source: source.into_runnable(values)?,
            }),
            Instruction::ShiftRight {
                destination,
                source,
            } => Ok(language::Instruction::ShiftRight {
                destination: destination.clone(),
                source: source.into_runnable(values)?,
            }),
            Instruction::PushU64 { source } => Ok(language::Instruction::PushU64 {
                source: source.clone().into_runnable(values)?,
            }),
            Instruction::PushF64 { source } => Ok(language::Instruction::PushF64 {
                source: source.clone().into_runnable(values)?,
            }),
            Instruction::PopU64 { destination } => Ok(language::Instruction::PopU64 {
                destination: destination.clone(),
            }),
            Instruction::PopF64 { destination } => Ok(language::Instruction::PopF64 {
                destination: destination.clone(),
            }),
            Instruction::LoadU64 {
                destination,
                source_address,
            } => Ok(language::Instruction::LoadU64 {
                destination: destination.clone(),
                source_address: source_address.into_runnable(values)?,
            }),
            Instruction::LoadF64 {
                destination,
                source_address,
            } => Ok(language::Instruction::LoadF64 {
                destination: destination.clone(),
                source_address: source_address.into_runnable(values)?,
            }),
            Instruction::StoreU64 {
                destination_address,
                source,
            } => Ok(language::Instruction::StoreU64 {
                destination_address: destination_address.into_runnable(values)?,
                source: source.into_runnable(values)?,
            }),
            Instruction::StoreF64 {
                destination_address,
                source,
            } => Ok(language::Instruction::StoreF64 {
                destination_address: destination_address.into_runnable(values)?,
                source: source.into_runnable(values)?,
            }),
        }
    }
}

#[derive(Debug, Clone)]
enum Statement {
    Instruction(Instruction),
    Label(String),
    Definition(String, Box<compile_time_expression::AST>),
}

#[derive(Debug, Clone)]
pub struct Program {
    pub runnable_program: language::Program,
}

impl Program {
    fn new(content: Vec<Statement>) -> Result<Self, String> {
        let mut values = HashMap::new();
        let mut instructions = Vec::new();
        let mut next_address = language::ProgramPointer(0);
        let mut expressions = Vec::new();

        // collect all the input into different lists, turning labels into address values as we go
        for item in content {
            match item {
                Statement::Instruction(instruction) => {
                    instructions.push(instruction);
                    next_address.advance();
                }
                Statement::Label(label) => {
                    values.insert(
                        label,
                        NumberLiteral::U64(next_address.try_into().map_err(|e| {
                            format!("failed to convert label address ({next_address:?}) into number literal: {e:?}")
                        })?),
                    );
                }
                Statement::Definition(name, expr) => {
                    expressions.push((name, expr));
                }
            }
        }

        // resolve definitions of compile-time variables
        for (name, expr) in expressions {
            let value = expr
                .evaluate(&values)
                .map_err(|e| format!("faile to evaluate AST: {e:?}"))?;
            values.insert(name, value);
        }

        // turn instructions into runnable instructions by substituting label and expression values
        let runnable_instructions = instructions
            .iter()
            .map(|instruction| instruction.into_runnable(&values))
            .collect::<Result<Vec<_>, _>>()?;

        // TODO configurable stack and heap sizes
        let stack_size = 1024;
        let heap_size = 65536;

        Ok(Program {
            runnable_program: language::Program::new(runnable_instructions, stack_size, heap_size),
        })
    }
}

pub fn parse(input: &str) -> Result<Program, String> {
    let argument = choice((
        identifier().map(|s| Argument::Identifier(s.to_string())),
        number_literal().map(Argument::Number),
    ));

    let argument_list = argument.padded().separated_by(just(',')).collect();

    /*
    the various kinds of statement will be parsed as options, because we want to do a validator to show errors and skip invalids but still
    continue parsing

    that means even stuff that doesn't need that will return options
    */

    let instruction = identifier().padded().then(argument_list).validate(
        |(instruction, arguments), e, emitter| match Instruction::new(instruction, arguments) {
            Ok(instruction) => Some(Statement::Instruction(instruction)),
            Err(error) => {
                emitter.emit(Rich::custom(e.span(), error));
                None
            }
        },
    );

    let label = identifier()
        .then(just(':'))
        .padded()
        .map(|(name, _)| Some(Statement::Label(name)));

    let expression = identifier()
        .then_ignore(just("=").padded())
        .then(compile_time_expression())
        .map(|(name, expr)| Some(Statement::Definition(name, expr)));

    // a list of either valid statements, or None that represents places where invalid instructions were validated and rejected
    let program = choice((expression, label, instruction))
        .repeated()
        .collect::<Vec<_>>();

    Program::new(
        program
            .parse(input)
            .into_result()
            .map_err(|e| format!("{e:?}"))?
            .into_iter()
            .flatten()
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO actual tests
    #[test]
    fn test_parse() {
        let input = r"
        main:
            add r0, r0, r0

        x = 42
        y = 43.5
        foo:
        bar:
            add r0, r1, x
            sub f1, velocity_x, y
            jmp foo
        ";
        let result = parse(input);
        match &result {
            Ok(result) => println!("TODO runnable program: {:?}", result.runnable_program),
            Err(e) => println!("TODO error parsing: {:?}", e),
        }
        assert!(result.is_ok());
    }
}
