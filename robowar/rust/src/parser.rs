use chumsky::{
    container::Seq,
    error::RichReason,
    extra::{Err, Full, ParserExtra},
    prelude::*,
    regex::Regex,
};
use tracing::*;

use crate::simulation::{self, language};

#[derive(Debug, Clone)]
pub enum NumberLiteral {
    I64(i64),
    U64(u64),
    F64(f64),
}

#[derive(Debug, Clone)]
pub enum Argument {
    Identifier(String),
    Number(NumberLiteral),
}

#[derive(Debug, Clone)]
pub struct Instruction {
    pub instruction: String,
    pub arguments: Vec<Argument>,
}

#[derive(Debug, Clone)]
pub enum ParsedInstruction {
    Valid(language::Instruction),
    Invalid {
        instruction: Instruction,
        error: String,
    },
}

impl TryInto<language::SourceU64> for Argument {
    type Error = String;

    fn try_into(self) -> Result<language::SourceU64, Self::Error> {
        match self {
            // TODO handle label addresses?
            Argument::Identifier(s) => Ok(language::SourceU64::Register(s.as_str().try_into()?)),
            Argument::Number(NumberLiteral::U64(result)) => {
                Ok(language::SourceU64::Literal(result))
            }
            Argument::Number(NumberLiteral::I64(result)) => {
                Ok(language::SourceU64::Literal(result as u64))
            }
            Argument::Number(number) => Err(format!("expected a u64 literal, found: {:?}", number)),
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
                "expected a writable u64 register, found: {:?}",
                number
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
            Argument::Number(number) => Err(format!("expected a f64 literal, found: {:?}", number)),
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
                "expected a writable f64 register, found: {:?}",
                number
            )),
        }
    }
}

impl ParsedInstruction {
    fn new(
        Instruction {
            instruction,
            arguments,
        }: Instruction,
    ) -> Self {
        match instruction.to_lowercase().as_str() {
            "add" => match arguments.as_slice() {
                [destination, left, right] => {
                    if let Ok(destination) = destination.clone().try_into()
                        && let Ok(left) = left.clone().try_into()
                        && let Ok(right) = right.clone().try_into()
                    {
                        ParsedInstruction::Valid(language::Instruction::AddU64 {
                            destination,
                            left,
                            right,
                        })
                    } else if let Ok(destination) = destination.clone().try_into()
                        && let Ok(left) = left.clone().try_into()
                        && let Ok(right) = right.clone().try_into()
                    {
                        ParsedInstruction::Valid(language::Instruction::AddF64 {
                            destination,
                            left,
                            right,
                        })
                    } else {
                        ParsedInstruction::Invalid {
                            instruction: Instruction {
                                instruction: instruction.clone(),
                                arguments: arguments.clone(),
                            },
                            error: format!(
                                "invalid arguments for '{}': destination: {:?}, left: {:?}, right: {:?}",
                                instruction.to_uppercase(),
                                destination,
                                left,
                                right
                            ),
                        }
                    }
                }
                _ => ParsedInstruction::Invalid {
                    instruction: Instruction {
                        instruction: instruction.clone(),
                        arguments: arguments.clone(),
                    },
                    error: format!(
                        "expected 3 arguments for '{}', found {}",
                        instruction.to_uppercase(),
                        arguments.len()
                    ),
                },
            },

            "sub" => match arguments.as_slice() {
                [destination, left, right] => {
                    if let Ok(destination) = destination.clone().try_into()
                        && let Ok(left) = left.clone().try_into()
                        && let Ok(right) = right.clone().try_into()
                    {
                        ParsedInstruction::Valid(language::Instruction::SubU64 {
                            destination,
                            left,
                            right,
                        })
                    } else if let Ok(destination) = destination.clone().try_into()
                        && let Ok(left) = left.clone().try_into()
                        && let Ok(right) = right.clone().try_into()
                    {
                        ParsedInstruction::Valid(language::Instruction::SubF64 {
                            destination,
                            left,
                            right,
                        })
                    } else {
                        ParsedInstruction::Invalid {
                            instruction: Instruction {
                                instruction: instruction.clone(),
                                arguments: arguments.clone(),
                            },
                            error: format!(
                                "invalid arguments for '{}': destination: {:?}, left: {:?}, right: {:?}",
                                instruction.to_uppercase(),
                                destination,
                                left,
                                right
                            ),
                        }
                    }
                }
                _ => ParsedInstruction::Invalid {
                    instruction: Instruction {
                        instruction: instruction.clone(),
                        arguments: arguments.clone(),
                    },
                    error: format!(
                        "expected 3 arguments for '{}', found {}",
                        instruction.to_uppercase(),
                        arguments.len()
                    ),
                },
            },

            "mul" => match arguments.as_slice() {
                [destination, left, right] => {
                    if let Ok(destination) = destination.clone().try_into()
                        && let Ok(left) = left.clone().try_into()
                        && let Ok(right) = right.clone().try_into()
                    {
                        ParsedInstruction::Valid(language::Instruction::MulU64 {
                            destination,
                            left,
                            right,
                        })
                    } else if let Ok(destination) = destination.clone().try_into()
                        && let Ok(left) = left.clone().try_into()
                        && let Ok(right) = right.clone().try_into()
                    {
                        ParsedInstruction::Valid(language::Instruction::MulF64 {
                            destination,
                            left,
                            right,
                        })
                    } else {
                        ParsedInstruction::Invalid {
                            instruction: Instruction {
                                instruction: instruction.clone(),
                                arguments: arguments.clone(),
                            },
                            error: format!(
                                "invalid arguments for '{}': destination: {:?}, left: {:?}, right: {:?}",
                                instruction.to_uppercase(),
                                destination,
                                left,
                                right
                            ),
                        }
                    }
                }
                _ => ParsedInstruction::Invalid {
                    instruction: Instruction {
                        instruction: instruction.clone(),
                        arguments: arguments.clone(),
                    },
                    error: format!(
                        "expected 3 arguments for '{}', found {}",
                        instruction.to_uppercase(),
                        arguments.len()
                    ),
                },
            },

            "div" => match arguments.as_slice() {
                [destination, left, right] => {
                    if let Ok(destination) = destination.clone().try_into()
                        && let Ok(left) = left.clone().try_into()
                        && let Ok(right) = right.clone().try_into()
                    {
                        ParsedInstruction::Valid(language::Instruction::DivU64 {
                            destination,
                            left,
                            right,
                        })
                    } else if let Ok(destination) = destination.clone().try_into()
                        && let Ok(left) = left.clone().try_into()
                        && let Ok(right) = right.clone().try_into()
                    {
                        ParsedInstruction::Valid(language::Instruction::DivF64 {
                            destination,
                            left,
                            right,
                        })
                    } else {
                        ParsedInstruction::Invalid {
                            instruction: Instruction {
                                instruction: instruction.clone(),
                                arguments: arguments.clone(),
                            },
                            error: format!(
                                "invalid arguments for '{}': destination: {:?}, left: {:?}, right: {:?}",
                                instruction.to_uppercase(),
                                destination,
                                left,
                                right
                            ),
                        }
                    }
                }
                _ => ParsedInstruction::Invalid {
                    instruction: Instruction {
                        instruction: instruction.clone(),
                        arguments: arguments.clone(),
                    },
                    error: format!(
                        "expected 3 arguments for '{}', found {}",
                        instruction.to_uppercase(),
                        arguments.len()
                    ),
                },
            },

            "jmp" => match arguments.as_slice() {
                [address] => {
                    if let Ok(address) = address.clone().try_into() {
                        ParsedInstruction::Valid(language::Instruction::Jump { address })
                    } else {
                        ParsedInstruction::Invalid {
                            instruction: Instruction {
                                instruction: instruction.clone(),
                                arguments: arguments.clone(),
                            },
                            error: format!(
                                "invalid argument for '{}': address: {:?}",
                                instruction.to_uppercase(),
                                address
                            ),
                        }
                    }
                }
                _ => ParsedInstruction::Invalid {
                    instruction: Instruction {
                        instruction: instruction.clone(),
                        arguments: arguments.clone(),
                    },
                    error: format!(
                        "expected 1 argument for '{}', found {}",
                        instruction.to_uppercase(),
                        arguments.len()
                    ),
                },
            },

            /*
            TODO handle all instruction types

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
            */
            _ => ParsedInstruction::Invalid {
                instruction: Instruction {
                    instruction: instruction.clone(),
                    arguments,
                },
                error: format!("unrecognized instruction: {}", instruction),
            },
        }
    }
}

// TODO actually return a full program
pub fn parse(input: &str) -> Result<(), Vec<Rich<char>>> {
    let identifier: Regex<&_, Err<Rich<char>>> = regex("[a-zA-Z_][a-zA-Z0-9_]*");
    let identifier = identifier.map(|s: &str| s.to_string()).padded();

    let number_i64 = regex(r"-?[0-9]+").try_map(|input: &str, span| match input.parse() {
        Ok(result) => Ok(NumberLiteral::I64(result)),
        Err(e) => Err(Rich::custom(span, e)),
    });
    let number_u64 = regex(r"[0-9]+").try_map(|input: &str, span| match input.parse() {
        Ok(result) => Ok(NumberLiteral::U64(result)),
        Err(e) => Err(Rich::custom(span, e)),
    });
    // TODO hex, binary, octal
    // TODO floats
    let number = choice((number_i64, number_u64)).padded();

    let argument = choice((
        identifier
            .clone()
            .map(|s| Argument::Identifier(s.to_string())),
        number.map(Argument::Number),
    ));

    let argument_list = argument.separated_by(just(',')).collect();

    let instruction = identifier
        .then(argument_list)
        .map(|(instruction, arguments)| Instruction {
            instruction: instruction.to_string(),
            arguments,
        })
        .validate(|instruction, e, emitter| {
            let result = ParsedInstruction::new(instruction);
            if let ParsedInstruction::Invalid {
                instruction: _,
                error,
            } = &result
            {
                emitter.emit(Rich::custom(e.span(), error));
            }
            result
        });

    let program = instruction.repeated().collect::<Vec<_>>();

    let result = program.parse(input).into_result()?;
    for instruction in result.iter() {
        info!("TODO parsed instruction: {:?}", instruction);
    }

    Ok(())
}
