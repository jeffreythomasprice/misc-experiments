use chumsky::{
    container::Seq,
    error::RichReason,
    extra::{Err, Full, ParserExtra},
    prelude::*,
    regex::Regex,
};

enum NumberLiteral {
    I64(i64),
    U64(u64),
    F64(f64),
}

enum Argument {
    Identifier(String),
    Number(NumberLiteral),
}

struct Instruction {
    instruction: String,
    arguments: Vec<Argument>,
}

pub fn parse(input: &str) -> Result<Vec<Instruction>, Vec<Rich<char>>> {
    let identifier: Regex<&_, Err<Rich<char>>> = regex("[a-zA-Z_][a-zA-Z0-9_]*");

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
            .map(|s: &str| Argument::Identifier(s.to_string())),
        number.map(Argument::Number),
    ))
    .padded();

    let argument_list = argument.separated_by(just(',')).collect();

    let instruction =
        identifier
            .padded()
            .then(argument_list)
            .map(|(instruction, arguments): (&str, _)| Instruction {
                instruction: instruction.to_string(),
                arguments,
            });

    let program = instruction.repeated().collect::<Vec<_>>();
    // TODO validate() to check that all instructions are valid, turn them into Instruction enum

    program.parse(input).into_result()
}

// TODO no
// fn identifier<'a>() -> impl Parser<'a, &'a str, &'a str, Err<Rich<'a, char>>> {
//     regex("[a-zA-Z_][a-zA-Z0-9_]*")
// }
