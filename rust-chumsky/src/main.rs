use std::num::ParseIntError;

use chumsky::prelude::*;

// fn identifier<'a>() -> impl Parser<'a, &'a str, &'a str> {
//     regex("^[a-zA-Z_][a-zA-Z0-9_]*$")
// }

// fn number_u32<'a>() -> impl Parser<'a, &'a str, u32, extra::Full<Rich<'a, &'a str>>> {
//     // regex("^[0-9]+$").map(|x: &str| x.parse())

//     // regex("^[0-9]+$").validate(|fs: &str, _, emitter| fs.parse())

//     regex("^[0-9]+$").try_map(|x: &str, span| match x.parse() {
//         Ok(result) => Ok(result),
//         Err(e) => Err(Rich::custom(span, format!("{e:?}"))),
//     })
// }

#[derive(Debug)]
enum Argument<'a> {
    Identifier(&'a str),
    U32(u32),
}

fn argument<'a>() -> impl Parser<'a, &'a str, Argument<'a>> {
    // let identifier = regex("^[a-zA-Z_][a-zA-Z0-9_]*$").try_map(|x: &str, span| Ok(x));

    // let number_u32 = regex("^[0-9]+$").try_map(|x: &str, span| match x.parse() {
    //     Ok(result) => Ok(result),
    //     Err(e) => Err(<EmptyErr as std::default::Default>::default()),
    // });

    let identifier = regex("^[a-zA-Z_][a-zA-Z0-9_]*$").try_map(|x: &str, span| Ok(x));

    let number_u32 = regex("^[0-9]+$").try_map(|x: &str, span| match x.parse() {
        Ok(result) => Ok(result),
        Err(e) => Err(Rich::custom(span, e)),
    });

    choice((
        identifier.map(Argument::Identifier),
        number_u32.map(Argument::U32),
    ))
}

fn main() {
    let input = "99999999999999";
    let parser = argument();
    let result = parser.parse(input);
    println!("{:?}", result);
}

// # use chumsky::{prelude::*, error::Simple};
// #[derive(Debug, PartialEq)]
// enum Token { Word(String), Num(u64) }
//
// let word = any::<_, extra::Err<Simple<char>>>()
//     .filter(|c: &char| c.is_alphabetic())
//     .repeated().at_least(1)
//     .collect::<String>()
//     .map(Token::Word);
//
// let num = any::<_, extra::Err<Simple<char>>>()
//     .filter(|c: &char| c.is_ascii_digit())
//     .repeated().at_least(1)
//     .collect::<String>()
//     .map(|s| Token::Num(s.parse().unwrap()));
//
// let token = word.or(num);
