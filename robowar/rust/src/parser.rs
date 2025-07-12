use std::collections::HashMap;

use chumsky::{
    container::Seq,
    error::RichReason,
    extra::{Err, Full, ParserExtra},
    prelude::*,
    regex::Regex,
    text::ascii::ident,
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

#[derive(Debug, Clone)]
pub struct UnvalidatedInstruction {
    pub instruction: String,
    pub arguments: Vec<Argument>,
}

#[derive(Debug, Clone)]
pub enum ValidInstruction {
    Valid(language::Instruction),
    Invalid {
        instruction: UnvalidatedInstruction,
        error: String,
    },
}

impl ValidInstruction {
    fn new(
        UnvalidatedInstruction {
            instruction,
            arguments,
        }: UnvalidatedInstruction,
    ) -> Self {
        match instruction.to_lowercase().as_str() {
            "add" => match arguments.as_slice() {
                [destination, left, right] => {
                    if let Ok(destination) = destination.clone().try_into()
                        && let Ok(left) = left.clone().try_into()
                        && let Ok(right) = right.clone().try_into()
                    {
                        ValidInstruction::Valid(language::Instruction::AddU64 {
                            destination,
                            left,
                            right,
                        })
                    } else if let Ok(destination) = destination.clone().try_into()
                        && let Ok(left) = left.clone().try_into()
                        && let Ok(right) = right.clone().try_into()
                    {
                        ValidInstruction::Valid(language::Instruction::AddF64 {
                            destination,
                            left,
                            right,
                        })
                    } else {
                        ValidInstruction::Invalid {
                            instruction: UnvalidatedInstruction {
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
                _ => ValidInstruction::Invalid {
                    instruction: UnvalidatedInstruction {
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
                        ValidInstruction::Valid(language::Instruction::SubU64 {
                            destination,
                            left,
                            right,
                        })
                    } else if let Ok(destination) = destination.clone().try_into()
                        && let Ok(left) = left.clone().try_into()
                        && let Ok(right) = right.clone().try_into()
                    {
                        ValidInstruction::Valid(language::Instruction::SubF64 {
                            destination,
                            left,
                            right,
                        })
                    } else {
                        ValidInstruction::Invalid {
                            instruction: UnvalidatedInstruction {
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
                _ => ValidInstruction::Invalid {
                    instruction: UnvalidatedInstruction {
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
                        ValidInstruction::Valid(language::Instruction::MulU64 {
                            destination,
                            left,
                            right,
                        })
                    } else if let Ok(destination) = destination.clone().try_into()
                        && let Ok(left) = left.clone().try_into()
                        && let Ok(right) = right.clone().try_into()
                    {
                        ValidInstruction::Valid(language::Instruction::MulF64 {
                            destination,
                            left,
                            right,
                        })
                    } else {
                        ValidInstruction::Invalid {
                            instruction: UnvalidatedInstruction {
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
                _ => ValidInstruction::Invalid {
                    instruction: UnvalidatedInstruction {
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
                        ValidInstruction::Valid(language::Instruction::DivU64 {
                            destination,
                            left,
                            right,
                        })
                    } else if let Ok(destination) = destination.clone().try_into()
                        && let Ok(left) = left.clone().try_into()
                        && let Ok(right) = right.clone().try_into()
                    {
                        ValidInstruction::Valid(language::Instruction::DivF64 {
                            destination,
                            left,
                            right,
                        })
                    } else {
                        ValidInstruction::Invalid {
                            instruction: UnvalidatedInstruction {
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
                _ => ValidInstruction::Invalid {
                    instruction: UnvalidatedInstruction {
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
                        ValidInstruction::Valid(language::Instruction::Jump { address })
                    } else {
                        ValidInstruction::Invalid {
                            instruction: UnvalidatedInstruction {
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
                _ => ValidInstruction::Invalid {
                    instruction: UnvalidatedInstruction {
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
            _ => ValidInstruction::Invalid {
                instruction: UnvalidatedInstruction {
                    instruction: instruction.clone(),
                    arguments,
                },
                error: format!("unrecognized instruction: {}", instruction),
            },
        }
    }
}

#[derive(Debug, Clone)]
enum LabelOrInstruction {
    Instruction(language::Instruction),
    Label(String),
}

#[derive(Debug, Clone)]
pub struct Program {
    pub labels: HashMap<String, language::ProgramPointer>,
    pub instructions: Vec<language::Instruction>,
}

impl Program {
    fn new(content: Vec<LabelOrInstruction>) -> Self {
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

        Program {
            labels,
            instructions,
        }
    }
}

// TODO actually return a full program
pub fn parse(input: &str) -> Result<Program, Vec<Rich<char>>> {
    let identifier: Regex<&_, Err<Rich<char>>> = regex("[a-zA-Z_][a-zA-Z0-9_]*");
    let identifier = identifier.map(|s: &str| s.to_string());

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
    let number = choice((number_i64, number_u64));

    let argument = choice((
        identifier
            .clone()
            .map(|s| Argument::Identifier(s.to_string())),
        number.map(Argument::Number),
    ));

    let argument_list = argument.padded().separated_by(just(',')).collect();

    // either a Some(valid instruction) or None
    let instruction = identifier.clone().padded().then(argument_list).validate(
        |(instruction, arguments), e, emitter| match ValidInstruction::new(UnvalidatedInstruction {
            instruction,
            arguments,
        }) {
            ValidInstruction::Valid(instruction) => Some(instruction),
            ValidInstruction::Invalid {
                instruction: _,
                error,
            } => {
                emitter.emit(Rich::custom(e.span(), error));
                None
            }
        },
    );

    // a list of either valid instructions or labels, or None that represents places where invalid instructions were validated and rejected
    let instruction = instruction.map(|x| x.map(LabelOrInstruction::Instruction));
    let label = identifier
        .then(just(':'))
        .padded()
        .map(|(name, _)| Some(LabelOrInstruction::Label(name)));

    let program = choice((label, instruction)).repeated().collect::<Vec<_>>();

    Ok(Program::new(
        program
            .parse(input)
            .into_result()?
            .into_iter()
            .flatten()
            .collect(),
    ))
}

#[cfg(test)]
mod tests {
    use crate::parser::parse;

    // TODO actual tests
    #[test]
    fn test_parse() {
        let input = r"
        main:
            add r0, r0, r0

        foo:
        bar:
            add r0, r1, r2
        ";
        let result = parse(input);
        match &result {
            Ok(result) => {
                println!("TODO labels: {:?}", result.labels);
                for instruction in result.instructions.iter() {
                    println!("TODO instruction: {:?}", instruction);
                }
            }
            Err(e) => println!("TODO error parsing: {:?}", e),
        }
        assert!(result.is_ok());
    }
}
