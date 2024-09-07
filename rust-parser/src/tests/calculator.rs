use crate::{
    matchers::{
        any2, match2, match3, repeat, str, take_while, MapError, Mappable, Matcher, MatcherError,
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
        let match_u32 = {
            take_while(|_pos, c| ('0'..='9').contains(c)).map(|value| {
                value
                    .s
                    .parse::<u32>()
                    .map_err(|e| MapError(format!("failed to parse as u32: {e:?}")))
            })
        };

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

            todo!()
        };

        // TODO match_f64

        // TODO match various operations, parenthesis

        Self { m: todo!() }
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
            parser.apply(" 1.5 ".into()),
            Ok(Match {
                pos: Position { line: 0, column: 0 },
                remainder: PosStr {
                    pos: Position { line: 0, column: 0 },
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
