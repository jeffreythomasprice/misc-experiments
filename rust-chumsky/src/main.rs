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
struct Instruction<'a> {
    command: &'a str,
    arguments: Vec<Argument<'a>>,
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
        .map(|(command, arguments)| Instruction {
            command,
            arguments: arguments.unwrap_or(vec![]),
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
            foo  1 ,  foo, -42i64

        loop:
            add x, x, 1
            cmp x, 42
            jne loop
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
