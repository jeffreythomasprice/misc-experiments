use std::{collections::HashMap, num::ParseIntError, str::FromStr};

use chumsky::{
    container::Seq,
    error::RichReason,
    extra::{Err, Full, ParserExtra},
    prelude::*,
};

use crate::instruction_set::*;

#[derive(Debug)]
pub struct NumberLiteral<T> {
    value: T,
    had_type_suffix: bool,
}

#[derive(Debug)]
pub enum Argument<'a> {
    Identifier(&'a str),
    I32(NumberLiteral<i32>),
    U32(NumberLiteral<u32>),
    I64(NumberLiteral<i64>),
    U64(NumberLiteral<u64>),
}

#[derive(Debug)]
pub enum InstructionOrUnknown<'a> {
    Valid(Instruction),
    Unknown {
        command: &'a str,
        arguments: Vec<Argument<'a>>,
    },
}

#[derive(Debug)]
pub enum Statement<'a> {
    Instruction(InstructionOrUnknown<'a>),
    Label(&'a str),
}

pub fn program<'a>() -> impl Parser<'a, &'a str, Program<'a>, Err<Rich<'a, char>>> {
    statement()
        .repeated()
        .collect::<Vec<_>>()
        .validate(|instructions, e, emitter| {
            let mut valid_instruction_count = 0;
            let mut labels = HashMap::new();
            let instructions = instructions
                .into_iter()
                .filter_map(|statement| match statement {
                    Statement::Instruction(InstructionOrUnknown::Valid(instruction)) => {
                        valid_instruction_count += 1;
                        Some(instruction)
                    }
                    Statement::Instruction(InstructionOrUnknown::Unknown {
                        command: _,
                        arguments: _,
                    }) => None,
                    Statement::Label(label) => {
                        if labels.insert(label, valid_instruction_count).is_some() {
                            emitter.emit(Rich::custom(
                                e.span(),
                                format!("duplicate label: {}", label),
                            ));
                        }
                        None
                    }
                })
                .collect::<Vec<_>>();
            Program {
                labels,
                instructions,
            }
        })
}

fn statement<'a>() -> impl Parser<'a, &'a str, Statement<'a>, Err<Rich<'a, char>>> {
    choice((
        label().map(Statement::Label),
        instruction().map(Statement::Instruction),
    ))
}

fn label<'a>() -> impl Parser<'a, &'a str, &'a str, Err<Rich<'a, char>>> {
    identifier()
        .then(just(":"))
        .map(|(result, _)| result)
        .padded()
}

fn instruction<'a>() -> impl Parser<'a, &'a str, InstructionOrUnknown<'a>, Err<Rich<'a, char>>> {
    identifier()
        .padded()
        .then(
            argument()
                .padded()
                .separated_by(just(","))
                .collect::<Vec<_>>()
                .or_not(),
        )
        .map(|(command, arguments)| (command, arguments.unwrap_or(vec![])))
        .validate(
            |(command, arguments), e, emitter| match (command, arguments.as_slice()) {
                ("add", [destination, left, right]) => {
                    if let Ok(destination) = destination.try_into()
                        && let Ok(left) = left.try_into()
                        && let Ok(right) = right.try_into()
                    {
                        InstructionOrUnknown::Valid(Instruction::Add32 {
                            destination,
                            left,
                            right,
                        })
                    } else if let Ok(destination) = destination.try_into()
                        && let Ok(left) = left.try_into()
                        && let Ok(right) = right.try_into()
                    {
                        InstructionOrUnknown::Valid(Instruction::Add64 {
                            destination,
                            left,
                            right,
                        })
                    } else {
                        emitter.emit(Rich::custom(
                            e.span(),
                            format!("invalid arguments to {}: {:?}", command, arguments),
                        ));
                        InstructionOrUnknown::Unknown { command, arguments }
                    }
                }
                // TODO more commands
                _ => {
                    emitter.emit(Rich::custom(
                        e.span(),
                        format!("invalid command: {}", command),
                    ));
                    InstructionOrUnknown::Unknown { command, arguments }
                }
            },
        )
}

impl<'a> TryFrom<&Argument<'a>> for RegisterOrLiteral32 {
    type Error = String;

    fn try_from(value: &Argument<'a>) -> Result<Self, Self::Error> {
        match value {
            Argument::Identifier(identifier) => {
                identifier.parse().map(RegisterOrLiteral32::Register)
            }
            Argument::I32(NumberLiteral {
                value,
                had_type_suffix: _,
            }) => Ok(RegisterOrLiteral32::I32(*value)),
            Argument::U32(NumberLiteral {
                value,
                had_type_suffix: _,
            }) => Ok(RegisterOrLiteral32::U32(*value)),
            _ => Err(format!("not a valid 32-bit argument: {:?}", value)),
        }
    }
}

impl<'a> TryFrom<&Argument<'a>> for RegisterOrLiteral64 {
    type Error = String;

    fn try_from(value: &Argument<'a>) -> Result<Self, Self::Error> {
        match value {
            Argument::Identifier(identifier) => {
                identifier.parse().map(RegisterOrLiteral64::Register)
            }
            Argument::I32(NumberLiteral {
                value,
                had_type_suffix: true,
            }) => Err(format!(
                "invalid 32-bit literal where 64-bit expected: {:?}",
                value
            )),
            Argument::I32(NumberLiteral {
                value,
                had_type_suffix: false,
            }) => Ok(RegisterOrLiteral64::I64(*value as i64)),
            Argument::U32(NumberLiteral {
                value,
                had_type_suffix: true,
            }) => Err(format!(
                "invalid 32-bit literal where 64-bit expected: {:?}",
                value
            )),
            Argument::U32(NumberLiteral {
                value,
                had_type_suffix: false,
            }) => Ok(RegisterOrLiteral64::U64(*value as u64)),
            Argument::I64(NumberLiteral {
                value,
                had_type_suffix: _,
            }) => Ok(RegisterOrLiteral64::I64(*value)),
            Argument::U64(NumberLiteral {
                value,
                had_type_suffix: _,
            }) => Ok(RegisterOrLiteral64::U64(*value)),
        }
    }
}

impl<'a> TryFrom<&Argument<'a>> for Register32 {
    type Error = String;

    fn try_from(value: &Argument<'a>) -> Result<Self, Self::Error> {
        match value {
            Argument::Identifier(s) => s.parse(),
            _ => Err(format!("not a valid 32-bit register: {:?}", value)),
        }
    }
}

impl<'a> TryFrom<&Argument<'a>> for Register64 {
    type Error = String;

    fn try_from(value: &Argument<'a>) -> Result<Self, Self::Error> {
        match value {
            Argument::Identifier(s) => s.parse(),
            _ => Err(format!("not a valid 64-bit register: {:?}", value)),
        }
    }
}

fn argument<'a>() -> impl Parser<'a, &'a str, Argument<'a>, Err<Rich<'a, char>>> {
    // TODO can end up with stuff like I64(-1) followed by a command with no args ("u32"), so the suffix isn't correctly being respected in order
    choice((
        identifier().map(Argument::Identifier),
        unsigned_integer_with_suffix("u32".to_owned()).map(Argument::U32),
        signed_integer_with_suffix("i32".to_owned()).map(Argument::I32),
        unsigned_integer_with_suffix("u64".to_owned()).map(Argument::U64),
        signed_integer_with_suffix("i64".to_owned()).map(Argument::I64),
        unsigned_integer().map(Argument::U32),
        signed_integer().map(Argument::I32),
        unsigned_integer().map(Argument::U64),
        signed_integer().map(Argument::I64),
    ))
}

fn identifier<'a>() -> impl Parser<'a, &'a str, &'a str, Err<Rich<'a, char>>> {
    regex("[a-zA-Z_][a-zA-Z0-9_]*")
}

fn signed_integer<'a, T>() -> impl Parser<'a, &'a str, NumberLiteral<T>, Err<Rich<'a, char>>>
where
    T: FromStr,
    <T as FromStr>::Err: std::fmt::Display,
{
    regex(r"\-?[0-9]+").try_map(move |input: &str, span| match input.parse() {
        Ok(result) => Ok(NumberLiteral {
            value: result,
            had_type_suffix: false,
        }),
        Err(e) => Err(Rich::custom(span, e)),
    })
}

fn signed_integer_with_suffix<'a, T>(
    suffix: String,
) -> impl Parser<'a, &'a str, NumberLiteral<T>, Err<Rich<'a, char>>>
where
    T: FromStr,
    <T as FromStr>::Err: std::fmt::Display,
{
    signed_integer().then(just(suffix)).map(
        |(
            NumberLiteral {
                value,
                had_type_suffix: _,
            },
            _,
        ): (NumberLiteral<T>, _)| NumberLiteral {
            value,
            had_type_suffix: true,
        },
    )
}

fn unsigned_integer<'a, T>() -> impl Parser<'a, &'a str, NumberLiteral<T>, Err<Rich<'a, char>>>
where
    T: FromStr,
    <T as FromStr>::Err: std::fmt::Display,
{
    regex(r"[0-9]+").try_map(move |input: &str, span| match input.parse() {
        Ok(result) => Ok(NumberLiteral {
            value: result,
            had_type_suffix: false,
        }),
        Err(e) => Err(Rich::custom(span, e)),
    })
}

fn unsigned_integer_with_suffix<'a, T>(
    suffix: String,
) -> impl Parser<'a, &'a str, NumberLiteral<T>, Err<Rich<'a, char>>>
where
    T: FromStr,
    <T as FromStr>::Err: std::fmt::Display,
{
    unsigned_integer().then(just(suffix)).map(
        |(
            NumberLiteral {
                value,
                had_type_suffix: _,
            },
            _,
        ): (NumberLiteral<T>, _)| NumberLiteral {
            value,
            had_type_suffix: true,
        },
    )
}
