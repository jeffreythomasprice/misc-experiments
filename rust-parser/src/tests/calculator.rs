use std::{error::Error, fmt::Display};

use crate::strings::{Match, PosStr};

#[derive(Debug, Clone, PartialEq)]
enum ASTNode {
    Number(f64),
    Add(Box<ASTNode>, Box<ASTNode>),
    Subtract(Box<ASTNode>, Box<ASTNode>),
    Multiply(Box<ASTNode>, Box<ASTNode>),
    Divide(Box<ASTNode>, Box<ASTNode>),
}

enum MultiplyOrDivide {
    Multiply,
    Divide,
}

enum AddOrSubtract {
    Add,
    Subtract,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParseError(String);

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ParseError({})", self.0)
    }
}

impl Error for ParseError {}

fn skip_whitespace<'a>(input: PosStr<'a>) -> PosStr<'a> {
    input
        .take_while_and_remainder(|_, c| c.is_whitespace())
        .remainder
}

fn parse_single_char(input: PosStr, c: char) -> Result<Match<char>, ParseError> {
    match input.take_single_char() {
        Some(m) => Ok(m.map(|_| c)),
        None => Err(ParseError(format!("expected {}", c))),
    }
}

fn parse_number(input: PosStr) -> Result<Match<ASTNode>, ParseError> {
    let input = skip_whitespace(input);
    let result = input.take_while_and_remainder(|_, c| {
        c.is_digit(10) || *c == '.' || *c == 'e' || *c == 'E' || *c == '+' || *c == '-'
    });
    if result.matched.is_empty() {
        Err(ParseError("expected number".to_owned()))
    } else {
        match result.matched.s.parse() {
            Ok(value) => Ok(result.map(|_| ASTNode::Number(value))),
            Err(e) => Err(ParseError(format!(
                "failed to parse input as number, input={}, error={:?}",
                result.matched.s, e
            ))),
        }
    }
}

fn parse_negated_expression(input: PosStr) -> Result<Match<ASTNode>, ParseError> {
    let Match {
        source: _,
        matched: _,
        remainder,
        value: _,
    } = parse_single_char(skip_whitespace(input), '-')?;
    parse_expression(remainder)
}

fn parse_parenthesis(input: PosStr) -> Result<Match<ASTNode>, ParseError> {
    let Match {
        source: _,
        matched: _,
        remainder,
        value: _,
    } = parse_single_char(skip_whitespace(input), '(')?;
    let result = parse_expression(remainder)?;
    let remainder = parse_single_char(skip_whitespace(result.remainder), ')')?.remainder;
    Ok(Match {
        source: input,
        matched: result.matched,
        remainder: remainder,
        value: result.value,
    })
}

fn parse_term(input: PosStr) -> Result<Match<ASTNode>, ParseError> {
    println!("TODO parse_term: {input:?}");

    let number_result = match parse_number(input) {
        Ok(result) => {
            println!("TODO parse_term found number: {result:?}");
            return Ok(result);
        }
        Err(e) => e,
    };

    let negated_result = match parse_negated_expression(input) {
        Ok(result) => {
            println!("TODO parse_term found negation: {result:?}");
            return Ok(result);
        }
        Err(e) => e,
    };

    let parenthesis_result = match parse_parenthesis(input) {
        Ok(result) => {
            println!("TODO parse_term found parens: {result:?}");
            return Ok(result);
        }
        Err(e) => e,
    };

    Err(ParseError(format!(
        "no branch matched, {}, {}, {}",
        negated_result, parenthesis_result, number_result
    )))
}

fn parse_mulops(input: PosStr) -> Result<Match<ASTNode>, ParseError> {
    println!("TODO parse_mulops: {input:?}");

    let Match {
        source: _,
        matched: _,
        remainder,
        value: first,
    } = parse_term(input)?;
    let mut result = first;
    let mut remainder = remainder;

    loop {
        let (op, partial_remainder) = match skip_whitespace(remainder).take_single_char() {
            Some(Match {
                source: _,
                matched: _,
                remainder,
                value: '*',
            }) => (MultiplyOrDivide::Multiply, remainder),
            Some(Match {
                source: _,
                matched: _,
                remainder,
                value: '/',
            }) => (MultiplyOrDivide::Divide, remainder),
            _ => break,
        };

        let Match {
            source: _,
            matched: _,
            remainder: partial_remainder,
            value: next,
        } = parse_term(partial_remainder)?;
        result = match op {
            MultiplyOrDivide::Multiply => ASTNode::Multiply(Box::new(result), Box::new(next)),
            MultiplyOrDivide::Divide => ASTNode::Divide(Box::new(result), Box::new(next)),
        };
        remainder = partial_remainder;
    }

    let matched = input
        .take_until_position_and_remainder(&remainder.pos)
        .map_err(|e| ParseError(format!("{e:?}")))?
        .value;

    Ok(Match {
        source: input,
        matched,
        remainder,
        value: result,
    })
}

fn parse_addops(input: PosStr) -> Result<Match<ASTNode>, ParseError> {
    // TODO de-duplicate with parse_mulops? generic binary_op?

    println!("TODO parse_addops: {input:?}");

    let Match {
        source: _,
        matched: _,
        remainder,
        value: first,
    } = parse_mulops(input)?;
    let mut result = first;
    let mut remainder = remainder;

    loop {
        let (op, partial_remainder) = match skip_whitespace(remainder).take_single_char() {
            Some(Match {
                source: _,
                matched: _,
                remainder,
                value: '+',
            }) => (AddOrSubtract::Add, remainder),
            Some(Match {
                source: _,
                matched: _,
                remainder,
                value: '-',
            }) => (AddOrSubtract::Subtract, remainder),
            _ => break,
        };

        let Match {
            source: _,
            matched: _,
            remainder: partial_remainder,
            value: next,
        } = parse_mulops(partial_remainder)?;
        result = match op {
            AddOrSubtract::Add => ASTNode::Add(Box::new(result), Box::new(next)),
            AddOrSubtract::Subtract => ASTNode::Subtract(Box::new(result), Box::new(next)),
        };
        remainder = partial_remainder;
    }

    let matched = input
        .take_until_position_and_remainder(&remainder.pos)
        .map_err(|e| ParseError(format!("{e:?}")))?
        .value;

    Ok(Match {
        source: input,
        matched,
        remainder,
        value: result,
    })
}

fn parse_expression(input: PosStr) -> Result<Match<ASTNode>, ParseError> {
    parse_addops(input)
}

#[cfg(test)]
mod tests {
    use crate::{
        strings::{Match, PosStr, Position},
        tests::calculator::ASTNode,
    };

    use super::*;

    #[test]
    fn single_number() {
        assert_eq!(
            parse_expression("  1.5".into()),
            Ok(Match {
                source: PosStr {
                    pos: Position { line: 0, column: 0 },
                    s: "  1.5",
                },
                matched: PosStr {
                    pos: Position { line: 0, column: 0 },
                    s: "  1.5",
                },
                remainder: PosStr {
                    pos: Position { line: 0, column: 5 },
                    s: ""
                },
                value: ASTNode::Number(1.5f64),
            })
        );
    }

    #[test]
    fn number_and_remainder() {
        assert_eq!(
            parse_expression("1 2".into()),
            Ok(Match {
                source: PosStr {
                    pos: Position { line: 0, column: 0 },
                    s: "1 2",
                },
                matched: PosStr {
                    pos: Position { line: 0, column: 0 },
                    s: "1",
                },
                remainder: PosStr {
                    pos: Position { line: 0, column: 1 },
                    s: " 2"
                },
                value: ASTNode::Number(1f64),
            })
        );
    }

    #[test]
    fn addition() {
        assert_eq!(
            parse_expression("1 + 2".into()),
            Ok(Match {
                source: PosStr {
                    pos: Position { line: 0, column: 0 },
                    s: "1 + 2",
                },
                matched: PosStr {
                    pos: Position { line: 0, column: 0 },
                    s: "1 + 2",
                },
                remainder: PosStr {
                    pos: Position { line: 0, column: 5 },
                    s: ""
                },
                value: ASTNode::Add(
                    Box::new(ASTNode::Number(1f64)),
                    Box::new(ASTNode::Number(2f64))
                ),
            })
        );
    }

    #[test]
    fn parenthesis() {
        assert_eq!(
            parse_expression("(1 + 2)*3".into()),
            Ok(Match {
                source: PosStr {
                    pos: Position { line: 0, column: 0 },
                    s: "(1 + 2)*3",
                },
                matched: PosStr {
                    pos: Position { line: 0, column: 0 },
                    s: "(1 + 2)*3",
                },
                remainder: PosStr {
                    pos: Position { line: 0, column: 5 },
                    s: ""
                },
                value: ASTNode::Multiply(
                    Box::new(ASTNode::Add(
                        Box::new(ASTNode::Number(1f64)),
                        Box::new(ASTNode::Number(2f64))
                    )),
                    Box::new(ASTNode::Number(3f64))
                ),
            })
        );
    }

    /*
    TODO JEFF more test cases

    1.5-2.7
    -1*5/2+4
    -1*5-2*4
    */
}
