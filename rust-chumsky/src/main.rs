use std::{num::ParseIntError, str::FromStr};

use chumsky::{
    container::Seq,
    error::RichReason,
    extra::{Err, Full, ParserExtra},
    prelude::*,
};

fn identifier<'a>() -> impl Parser<'a, &'a str, &'a str, Err<Rich<'a, char>>> {
    regex("[a-zA-Z_][a-zA-Z0-9_]*")
}

#[derive(Debug)]
struct NumberLiteral<T> {
    value: T,
    had_type_suffix: bool,
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

#[derive(Debug)]
enum Argument<'a> {
    Identifier(&'a str),
    I32(NumberLiteral<i32>),
    U32(NumberLiteral<u32>),
    I64(NumberLiteral<i64>),
    U64(NumberLiteral<u64>),
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

#[derive(Debug)]
enum Register32 {
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    R8,
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

impl<'a> TryFrom<&Argument<'a>> for Register32 {
    type Error = String;

    fn try_from(value: &Argument<'a>) -> Result<Self, Self::Error> {
        match value {
            Argument::Identifier(s) => s.parse(),
            _ => Err(format!("not a valid 32-bit register: {:?}", value)),
        }
    }
}

#[derive(Debug)]
enum Register64 {
    R12,
    R34,
    R56,
    R78,
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

impl<'a> TryFrom<&Argument<'a>> for Register64 {
    type Error = String;

    fn try_from(value: &Argument<'a>) -> Result<Self, Self::Error> {
        match value {
            Argument::Identifier(s) => s.parse(),
            _ => Err(format!("not a valid 64-bit register: {:?}", value)),
        }
    }
}

#[derive(Debug)]
enum RegisterOrLiteral32 {
    Register(Register32),
    U32(u32),
    I32(i32),
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

#[derive(Debug)]
enum RegisterOrLiteral64 {
    Register(Register64),
    U64(u64),
    I64(i64),
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

#[derive(Debug)]
enum Instruction<'a> {
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
    Unknown {
        command: &'a str,
        arguments: Vec<Argument<'a>>,
    },
}

fn instruction<'a>() -> impl Parser<'a, &'a str, Instruction<'a>, Err<Rich<'a, char>>> {
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
        .validate(|(command, arguments), _, emitter| {
            match (command, arguments.as_slice()) {
                ("add", [destination, left, right]) => {
                    if let Ok(destination) = destination.try_into()
                        && let Ok(left) = left.try_into()
                        && let Ok(right) = right.try_into()
                    {
                        Instruction::Add32 {
                            destination,
                            left,
                            right,
                        }
                    } else if let Ok(destination) = destination.try_into()
                        && let Ok(left) = left.try_into()
                        && let Ok(right) = right.try_into()
                    {
                        Instruction::Add64 {
                            destination,
                            left,
                            right,
                        }
                    } else {
                        // TODO emit error to emitter
                        Instruction::Unknown { command, arguments }
                    }
                }
                _ => {
                    // TODO emit error to emitter
                    Instruction::Unknown { command, arguments }
                }
            }
        })
}

fn label<'a>() -> impl Parser<'a, &'a str, &'a str, Err<Rich<'a, char>>> {
    identifier()
        .then(just(":"))
        .map(|(result, _)| result)
        .padded()
}

#[derive(Debug)]
enum Statement<'a> {
    Instruction(Instruction<'a>),
    Label(&'a str),
}

fn statement<'a>() -> impl Parser<'a, &'a str, Statement<'a>, Err<Rich<'a, char>>> {
    choice((
        label().map(Statement::Label),
        instruction().map(Statement::Instruction),
    ))
}

fn program<'a>() -> impl Parser<'a, &'a str, Vec<Statement<'a>>, Err<Rich<'a, char>>> {
    statement().repeated().collect()
}

fn main() {
    let input = r"
        main:
            add r1, r2, 42
            add r34, r34, -1
    "
    .to_string();

    let parser = program();

    match parser.parse(&input).into_result() {
        Ok(result) => {
            for statement in result.iter() {
                println!("statement: {:?}", statement);
            }
        }
        Result::Err(e) => println!("failed: {:?}", e),
    };
}
