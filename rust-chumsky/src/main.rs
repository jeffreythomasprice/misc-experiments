use std::{num::ParseIntError, str::FromStr};

use chumsky::{
    error::RichReason,
    extra::{Err, Full, ParserExtra},
    prelude::*,
};

fn identifier<'a>() -> impl Parser<'a, &'a str, &'a str, Err<Rich<'a, char>>> {
    regex("^[a-zA-Z_][a-zA-Z0-9_]*$")
}

#[derive(Debug)]
struct NumberLiteral<T> {
    value: T,
    had_type_suffix: bool,
}

fn unsigned_integer<'a, T>(
    suffix: String,
) -> impl Parser<'a, &'a str, NumberLiteral<T>, Err<Rich<'a, char>>>
where
    T: FromStr,
    <T as FromStr>::Err: std::fmt::Display,
{
    regex(format!("^[0-9]+({})?$", suffix).as_str()).try_map(move |x: &str, span| {
        let (input, had_type_suffix) = if x.ends_with(&suffix) {
            (x.trim_end_matches(&suffix), true)
        } else {
            (x, false)
        };
        match input.parse() {
            Ok(result) => Ok(NumberLiteral {
                value: result,
                had_type_suffix,
            }),
            Err(e) => Err(Rich::custom(span, e)),
        }
    })
}

#[derive(Debug)]
enum Argument<'a> {
    Identifier(&'a str),
    U64(NumberLiteral<u64>),
    U32(NumberLiteral<u32>),
}

fn argument<'a>() -> impl Parser<'a, &'a str, Argument<'a>, Err<Rich<'a, char>>> {
    choice((
        identifier().map(Argument::Identifier),
        unsigned_integer("u32".to_owned()).map(Argument::U32),
        unsigned_integer("u64".to_owned()).map(Argument::U64),
    ))
}

fn main() {
    // let input = format!("{}", u64::MAX);
    // let input = "123u32".to_string();
    let input = "123u64".to_string();
    // let input = "123".to_string();

    let parser = argument();

    let result = parser.parse(&input);

    println!("{:?}", result);
}
