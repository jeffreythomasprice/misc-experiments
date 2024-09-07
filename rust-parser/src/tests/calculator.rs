use crate::{
    matchers::{
        any2, char, char_range, match2, match3, optional, repeat, specific_char, str, take_while,
        MapError, Mappable, Matcher, MatcherError, StrMappable,
    },
    strings::{Match, PosStr},
};

#[derive(Debug, Clone, PartialEq)]
enum ASTNode {
    Number(f64),
    Add(Box<ASTNode>, Box<ASTNode>),
    Subtract(Box<ASTNode>, Box<ASTNode>),
    Multiply(Box<ASTNode>, Box<ASTNode>),
    Divide(Box<ASTNode>, Box<ASTNode>),
}

struct Parser {
    m: Box<dyn Matcher<'static, Box<ASTNode>>>,
}

impl Parser {
    pub fn new() -> Self {
        let match_f64 = {
            /*
            based on the number spec from here https://www.json.org/json-en.html

            integer part is one of:
                0
                [1-9][0-9]*
            then optionally a fractional part which is:
                .[0-9]+
            then optionally an expoenent which is:
                [eE][-+]?[0-9]+
            */

            let zero = str("0");
            let digit = || char_range('0'..='9');
            let digit_not_zero = char_range('1'..='9');

            let integer_part = any2(
                zero.map_to_str(),
                match2(digit_not_zero, repeat(digit(), ..)).map_to_str(),
            );

            let fractional_part = optional(match2(str("."), repeat(digit(), 1..))).map_to_str();

            let exponent_part = optional(match3(
                any2(specific_char('e'), specific_char('E')),
                optional(any2(specific_char('+'), specific_char('-'))),
                repeat(digit(), 1..),
            ))
            .map_to_str();

            Box::new(
                match3(integer_part, fractional_part, exponent_part)
                    .map_to_str()
                    .map(|_, s| {
                        println!("TODO JEFF s = {s}");
                        match s.parse::<f64>() {
                            Ok(x) => Ok(Box::new(ASTNode::Number(x))),
                            Err(e) => Err(MapError(format!(
                                "error parsing as float, input=\"{s}\", error={e:?}"
                            ))),
                        }
                    }),
            )
        };

        // TODO match_f64

        // TODO match various operations, parenthesis

        Self { m: match_f64 }
    }

    pub fn apply(
        &self,
        input: PosStr<'static>,
    ) -> Result<Match<'static, Box<ASTNode>>, MatcherError> {
        self.m.apply(input)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        strings::{Match, PosStr, Position},
        tests::calculator::ASTNode,
    };

    use super::Parser;

    #[test]
    fn single_number() {
        let parser = Parser::new();
        assert_eq!(
            parser.apply("1.5".into()),
            Ok(Match {
                source: PosStr {
                    pos: Position { line: 0, column: 0 },
                    s: "1.5",
                },
                matched: PosStr {
                    pos: Position { line: 0, column: 0 },
                    s: "1.5",
                },
                remainder: PosStr {
                    pos: Position { line: 0, column: 3 },
                    s: ""
                },
                value: Box::new(ASTNode::Number(1.5f64)),
            })
        );
    }

    /*
    TODO JEFF more test cases

    1 2 -> has remainder
    1 + 2
    (1 + 2) * 3
    1.5-2.7
    -1*5/2+4
    -1*5-2*4
    */
}
