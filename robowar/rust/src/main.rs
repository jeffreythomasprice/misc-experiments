use std::num::ParseIntError;

use chumsky::prelude::*;

fn identifier<'a>() -> impl Parser<'a, &'a str, &'a str> {
    regex("^[a-zA-Z_][a-zA-Z0-9_]*$")
}

fn number_u32<'a>() -> chumsky::combinator::TryMap<
    chumsky::regex::Regex<&'a str, ParseIntError>,
    &'a str,
    impl Fn(&str, _) -> Result<u32, ParseIntError>,
> {
    regex("^[0-9]+$").try_map(|x: &str, span| x.parse::<u32>())
}

fn main() {
    let input = "foobar";
    let parser = identifier();
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
