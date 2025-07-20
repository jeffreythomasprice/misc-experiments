mod basic_types;
mod compile_time_expression;

use crate::simulation::language;
use basic_types::*;
use chumsky::{extra::Err, prelude::*, regex::Regex};
use compile_time_expression::*;
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
        labels: &HashMap<String, language::ProgramPointer>,
    ) -> Result<language::SourceU64, String> {
        match self {
            SourceU64::Register(register) => Ok(language::SourceU64::Register(register.clone())),
            SourceU64::Literal(literal) => Ok(language::SourceU64::Literal(*literal)),
            SourceU64::Label(label) => match labels.get(label) {
                Some(label) => Ok(language::SourceU64::Literal(label.0 as u64)),
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
pub enum Instruction {
    SetU64 {
        destination: language::DestinationU64,
        source: SourceU64,
    },
    SetF64 {
        destination: language::DestinationF64,
        source: language::SourceF64,
    },
    AddU64 {
        destination: language::DestinationU64,
        left: SourceU64,
        right: SourceU64,
    },
    AddF64 {
        destination: language::DestinationF64,
        left: language::SourceF64,
        right: language::SourceF64,
    },
    SubU64 {
        destination: language::DestinationU64,
        left: SourceU64,
        right: SourceU64,
    },
    SubF64 {
        destination: language::DestinationF64,
        left: language::SourceF64,
        right: language::SourceF64,
    },
    MulU64 {
        destination: language::DestinationU64,
        left: SourceU64,
        right: SourceU64,
    },
    MulF64 {
        destination: language::DestinationF64,
        left: language::SourceF64,
        right: language::SourceF64,
    },
    DivU64 {
        destination: language::DestinationU64,
        left: SourceU64,
        right: SourceU64,
    },
    DivF64 {
        destination: language::DestinationF64,
        left: language::SourceF64,
        right: language::SourceF64,
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
        left: language::SourceF64,
        right: language::SourceF64,
    },
    JumpNotEqualU64 {
        address: SourceU64,
        left: SourceU64,
        right: SourceU64,
    },
    JumpNotEqualF64 {
        address: SourceU64,
        left: language::SourceF64,
        right: language::SourceF64,
    },
    JumpLessThanU64 {
        address: SourceU64,
        left: SourceU64,
        right: SourceU64,
    },
    JumpLessThanF64 {
        address: SourceU64,
        left: language::SourceF64,
        right: language::SourceF64,
    },
    JumpLessThanOrEqualToU64 {
        address: SourceU64,
        left: SourceU64,
        right: SourceU64,
    },
    JumpLessThanOrEqualToF64 {
        address: SourceU64,
        left: language::SourceF64,
        right: language::SourceF64,
    },
    JumpGreaterThanU64 {
        address: SourceU64,
        left: SourceU64,
        right: SourceU64,
    },
    JumpGreaterThanF64 {
        address: SourceU64,
        left: language::SourceF64,
        right: language::SourceF64,
    },
    JumpGreaterThanOrEqualToU64 {
        address: SourceU64,
        left: SourceU64,
        right: SourceU64,
    },
    JumpGreaterThanOrEqualToF64 {
        address: SourceU64,
        left: language::SourceF64,
        right: language::SourceF64,
    },
    ShiftLeft {
        destination: language::DestinationU64,
        source: SourceU64,
    },
    ShiftRight {
        destination: language::DestinationU64,
        source: SourceU64,
    },
    // TODO push, pop
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

            // TODO push, pop
            _ => Err(format!("unrecognized instruction: {instruction}")),
        }
    }

    fn into_runnable(
        &self,
        labels: &HashMap<String, language::ProgramPointer>,
    ) -> Result<language::Instruction, String> {
        match self {
            Instruction::SetU64 {
                destination,
                source,
            } => Ok(language::Instruction::SetU64 {
                destination: destination.clone(),
                source: source.clone().into_runnable(labels)?,
            }),
            Instruction::SetF64 {
                destination,
                source,
            } => Ok(language::Instruction::SetF64 {
                destination: destination.clone(),
                source: source.clone(),
            }),
            Instruction::AddU64 {
                destination,
                left,
                right,
            } => Ok(language::Instruction::AddU64 {
                destination: destination.clone(),
                left: left.into_runnable(labels)?,
                right: right.into_runnable(labels)?,
            }),
            Instruction::AddF64 {
                destination,
                left,
                right,
            } => Ok(language::Instruction::AddF64 {
                destination: destination.clone(),
                left: left.clone(),
                right: right.clone(),
            }),
            Instruction::SubU64 {
                destination,
                left,
                right,
            } => Ok(language::Instruction::SubU64 {
                destination: destination.clone(),
                left: left.into_runnable(labels)?,
                right: right.into_runnable(labels)?,
            }),
            Instruction::SubF64 {
                destination,
                left,
                right,
            } => Ok(language::Instruction::SubF64 {
                destination: destination.clone(),
                left: left.clone(),
                right: right.clone(),
            }),
            Instruction::MulU64 {
                destination,
                left,
                right,
            } => Ok(language::Instruction::MulU64 {
                destination: destination.clone(),
                left: left.into_runnable(labels)?,
                right: right.into_runnable(labels)?,
            }),
            Instruction::MulF64 {
                destination,
                left,
                right,
            } => Ok(language::Instruction::MulF64 {
                destination: destination.clone(),
                left: left.clone(),
                right: right.clone(),
            }),
            Instruction::DivU64 {
                destination,
                left,
                right,
            } => Ok(language::Instruction::DivU64 {
                destination: destination.clone(),
                left: left.into_runnable(labels)?,
                right: right.into_runnable(labels)?,
            }),
            Instruction::DivF64 {
                destination,
                left,
                right,
            } => Ok(language::Instruction::DivF64 {
                destination: destination.clone(),
                left: left.clone(),
                right: right.clone(),
            }),
            Instruction::Jump { address } => Ok(language::Instruction::Jump {
                address: address.clone().into_runnable(labels)?,
            }),
            Instruction::JumpEqualU64 {
                address,
                left,
                right,
            } => Ok(language::Instruction::JumpEqualU64 {
                address: address.clone().into_runnable(labels)?,
                left: left.into_runnable(labels)?,
                right: right.into_runnable(labels)?,
            }),
            Instruction::JumpEqualF64 {
                address,
                left,
                right,
            } => Ok(language::Instruction::JumpEqualF64 {
                address: address.clone().into_runnable(labels)?,
                left: left.clone(),
                right: right.clone(),
            }),
            Instruction::JumpNotEqualU64 {
                address,
                left,
                right,
            } => Ok(language::Instruction::JumpNotEqualU64 {
                address: address.clone().into_runnable(labels)?,
                left: left.into_runnable(labels)?,
                right: right.into_runnable(labels)?,
            }),
            Instruction::JumpNotEqualF64 {
                address,
                left,
                right,
            } => Ok(language::Instruction::JumpNotEqualF64 {
                address: address.clone().into_runnable(labels)?,
                left: left.clone(),
                right: right.clone(),
            }),
            Instruction::JumpLessThanU64 {
                address,
                left,
                right,
            } => Ok(language::Instruction::JumpLessThanU64 {
                address: address.clone().into_runnable(labels)?,
                left: left.into_runnable(labels)?,
                right: right.into_runnable(labels)?,
            }),
            Instruction::JumpLessThanF64 {
                address,
                left,
                right,
            } => Ok(language::Instruction::JumpLessThanF64 {
                address: address.clone().into_runnable(labels)?,
                left: left.clone(),
                right: right.clone(),
            }),
            Instruction::JumpLessThanOrEqualToU64 {
                address,
                left,
                right,
            } => Ok(language::Instruction::JumpLessThanOrEqualToU64 {
                address: address.clone().into_runnable(labels)?,
                left: left.into_runnable(labels)?,
                right: right.into_runnable(labels)?,
            }),
            Instruction::JumpLessThanOrEqualToF64 {
                address,
                left,
                right,
            } => Ok(language::Instruction::JumpLessThanOrEqualToF64 {
                address: address.clone().into_runnable(labels)?,
                left: left.clone(),
                right: right.clone(),
            }),
            Instruction::JumpGreaterThanU64 {
                address,
                left,
                right,
            } => Ok(language::Instruction::JumpGreaterThanU64 {
                address: address.clone().into_runnable(labels)?,
                left: left.into_runnable(labels)?,
                right: right.into_runnable(labels)?,
            }),
            Instruction::JumpGreaterThanF64 {
                address,
                left,
                right,
            } => Ok(language::Instruction::JumpGreaterThanF64 {
                address: address.clone().into_runnable(labels)?,
                left: left.clone(),
                right: right.clone(),
            }),
            Instruction::JumpGreaterThanOrEqualToU64 {
                address,
                left,
                right,
            } => Ok(language::Instruction::JumpGreaterThanOrEqualToU64 {
                address: address.clone().into_runnable(labels)?,
                left: left.into_runnable(labels)?,
                right: right.into_runnable(labels)?,
            }),
            Instruction::JumpGreaterThanOrEqualToF64 {
                address,
                left,
                right,
            } => Ok(language::Instruction::JumpGreaterThanOrEqualToF64 {
                address: address.clone().into_runnable(labels)?,
                left: left.clone(),
                right: right.clone(),
            }),
            Instruction::ShiftLeft {
                destination,
                source,
            } => Ok(language::Instruction::ShiftLeft {
                destination: destination.clone(),
                source: source.into_runnable(labels)?,
            }),
            Instruction::ShiftRight {
                destination,
                source,
            } => Ok(language::Instruction::ShiftRight {
                destination: destination.clone(),
                source: source.into_runnable(labels)?,
            }),
        }
    }
}

#[derive(Debug, Clone)]
enum LabelOrInstruction {
    Instruction(Instruction),
    Label(String),
}

#[derive(Debug, Clone)]
pub struct Program {
    pub labels: HashMap<String, language::ProgramPointer>,
    pub instructions: Vec<Instruction>,
    pub runnable_program: language::Program,
}

impl Program {
    fn new(content: Vec<LabelOrInstruction>) -> Result<Self, String> {
        let mut labels = HashMap::new();
        let mut instructions = Vec::new();
        let mut next_address = language::ProgramPointer(0);

        for item in content {
            match item {
                LabelOrInstruction::Instruction(instruction) => {
                    instructions.push(instruction);
                    next_address.advance();
                }
                LabelOrInstruction::Label(label) => {
                    labels.insert(label, next_address);
                }
            }
        }

        let runnable_instructions = instructions
            .iter()
            .map(|instruction| instruction.into_runnable(&labels))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Program {
            labels,
            instructions,
            runnable_program: language::Program::new(runnable_instructions),
        })
    }
}

pub fn parse(input: &str) -> Result<Program, String> {
    let argument = choice((
        identifier().map(|s| Argument::Identifier(s.to_string())),
        number_literal().map(Argument::Number),
    ));

    let argument_list = argument.padded().separated_by(just(',')).collect();

    // either a Some(valid instruction) or None
    let instruction = identifier().padded().then(argument_list).validate(
        |(instruction, arguments), e, emitter| match Instruction::new(instruction, arguments) {
            Ok(instruction) => Some(LabelOrInstruction::Instruction(instruction)),
            Err(error) => {
                emitter.emit(Rich::custom(e.span(), error));
                None
            }
        },
    );

    // because instructions have to be options because we skip invalid instructions, this will be an option too
    let label = identifier()
        .then(just(':'))
        .padded()
        .map(|(name, _)| Some(LabelOrInstruction::Label(name)));

    // TODO compile time expressions, of the form "identifier = expression"

    // a list of either valid instructions or labels, or None that represents places where invalid instructions were validated and rejected
    let program = choice((label, instruction)).repeated().collect::<Vec<_>>();

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

        foo:
        bar:
            add r0, r1, r2
            add velocity_x, velocity_x, 1.5
            sub velocity_y, velocity_y, -17
            jmp foo
        ";
        let result = parse(input);
        match &result {
            Ok(result) => {
                println!("TODO labels: {:?}", result.labels);
                for instruction in result.instructions.iter() {
                    println!("TODO instruction: {:?}", instruction);
                }
                println!("TODO runnable program: {:?}", result.runnable_program);
            }
            Err(e) => println!("TODO error parsing: {:?}", e),
        }
        assert!(result.is_ok());
    }
}
