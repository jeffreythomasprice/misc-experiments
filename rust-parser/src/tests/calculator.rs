use std::{
    error::Error,
    fmt::{Debug, Display},
    ops::RangeBounds,
};

use crate::strings::{Match, PosStr};

#[derive(Debug, Clone, PartialEq)]
enum ASTNode {
    Number(f64),
    Add(Box<ASTNode>, Box<ASTNode>),
    Subtract(Box<ASTNode>, Box<ASTNode>),
    Multiply(Box<ASTNode>, Box<ASTNode>),
    Divide(Box<ASTNode>, Box<ASTNode>),
    Negate(Box<ASTNode>),
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
        Some(m) if m.value == c => Ok(m),
        _ => Err(ParseError(format!("expected {}", c))),
    }
}

fn prase_char_range<R>(input: PosStr, range: R) -> Result<Match<char>, ParseError>
where
    R: RangeBounds<char> + Debug,
{
    match input.take_single_char() {
        Some(m) if range.contains(&m.value) => Ok(m),
        Some(m) => Err(ParseError(format!("expected {:?}, got {}", range, m.value))),
        _ => Err(ParseError(format!("expected {:?}", range))),
    }
}

fn parse_number(input: PosStr) -> Result<Match<ASTNode>, ParseError> {
    /*
    https://www.json.org/json-en.html
    https://stackoverflow.com/a/13340826
    -?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?

    ignoring the leading negation because we have an AST node for that
    */

    let input = skip_whitespace(input);

    // 0|[1-9]\d*
    let first_integer_digit = prase_char_range(input, '0'..='9')?;
    let (remaining_integer_digits, remainder) = if ('1'..='9').contains(&first_integer_digit.value)
    {
        let r = first_integer_digit
            .remainder
            .take_while_and_remainder(|_, c| ('0'..='9').contains(c));
        (Some(r.value), r.remainder)
    } else {
        (None, first_integer_digit.remainder)
    };

    // (\.\d+)?
    let remainder = if let Ok(dot_match) = parse_single_char(remainder, '.') {
        let digits_match = dot_match
            .remainder
            .take_while_and_remainder(|_, c| ('0'..='9').contains(c));
        if !digits_match.value.is_empty() {
            digits_match.remainder
        } else {
            remainder
        }
    } else {
        remainder
    };

    // ([eE][+-]?\d+)?
    let remainder = match remainder.take_single_char() {
        Some(e) if e.value == 'e' || e.value == 'E' => match e.remainder.take_single_char() {
            Some(sign) if sign.value == '+' || sign.value == '-' => {
                let digits = sign
                    .remainder
                    .take_while_and_remainder(|_, c| ('0'..='9').contains(c));
                if digits.value.is_empty() {
                    remainder
                } else {
                    digits.remainder
                }
            }
            _ => e.remainder,
        },
        _ => remainder,
    };

    let full_match = input
        .take_until_position_and_remainder(&remainder.pos)
        .map_err(|e| ParseError(format!("{e:?}")))?;
    match full_match.matched.s.parse() {
        Ok(value) => Ok(full_match.map(|_| ASTNode::Number(value))),
        Err(e) => Err(ParseError(format!(
            "failed to parse input as number, input={}, error={:?}",
            full_match.matched.s, e
        ))),
    }
}

fn parse_negated_number(input: PosStr) -> Result<Match<ASTNode>, ParseError> {
    let Match {
        source: _,
        matched: _,
        remainder,
        value: _,
    } = parse_single_char(skip_whitespace(input), '-')?;
    Ok(parse_number(remainder)?.map(|x| ASTNode::Negate(Box::new(x))))
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
    let number_result = match parse_number(input) {
        Ok(result) => {
            return Ok(result);
        }
        Err(e) => e,
    };

    let negated_result = match parse_negated_number(input) {
        Ok(result) => {
            return Ok(result);
        }
        Err(e) => e,
    };

    let parenthesis_result = match parse_parenthesis(input) {
        Ok(result) => {
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
    fn negated_number() {
        assert_eq!(
            parse_expression("-7.1".into()),
            Ok(Match {
                source: PosStr {
                    pos: Position { line: 0, column: 0 },
                    s: "-7.1",
                },
                matched: PosStr {
                    pos: Position { line: 0, column: 0 },
                    s: "-7.1",
                },
                remainder: PosStr {
                    pos: Position { line: 0, column: 4 },
                    s: ""
                },
                value: ASTNode::Negate(Box::new(ASTNode::Number(7.1f64))),
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
                    pos: Position { line: 0, column: 9 },
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

    #[test]
    fn subtraction() {
        assert_eq!(
            parse_expression("1.5-2.7".into()),
            Ok(Match {
                source: PosStr {
                    pos: Position { line: 0, column: 0 },
                    s: "1.5-2.7",
                },
                matched: PosStr {
                    pos: Position { line: 0, column: 0 },
                    s: "1.5-2.7",
                },
                remainder: PosStr {
                    pos: Position { line: 0, column: 7 },
                    s: ""
                },
                value: ASTNode::Subtract(
                    Box::new(ASTNode::Number(1.5f64)),
                    Box::new(ASTNode::Number(2.7f64))
                ),
            })
        );
    }

    #[test]
    fn order_of_operations_1() {
        assert_eq!(
            parse_expression("-1*5/2+4".into()),
            Ok(Match {
                source: PosStr {
                    pos: Position { line: 0, column: 0 },
                    s: "-1*5/2+4",
                },
                matched: PosStr {
                    pos: Position { line: 0, column: 0 },
                    s: "-1*5/2+4",
                },
                remainder: PosStr {
                    pos: Position { line: 0, column: 8 },
                    s: ""
                },
                value: ASTNode::Add(
                    Box::new(ASTNode::Divide(
                        Box::new(ASTNode::Multiply(
                            Box::new(ASTNode::Negate(Box::new(ASTNode::Number(1f64)))),
                            Box::new(ASTNode::Number(5f64))
                        )),
                        Box::new(ASTNode::Number(2f64))
                    )),
                    Box::new(ASTNode::Number(4f64))
                )
            })
        );
    }

    #[test]
    fn order_of_operations_2() {
        assert_eq!(
            parse_expression("-1*5-2*4".into()),
            Ok(Match {
                source: PosStr {
                    pos: Position { line: 0, column: 0 },
                    s: "-1*5-2*4",
                },
                matched: PosStr {
                    pos: Position { line: 0, column: 0 },
                    s: "-1*5-2*4",
                },
                remainder: PosStr {
                    pos: Position { line: 0, column: 8 },
                    s: ""
                },
                value: ASTNode::Subtract(
                    Box::new(ASTNode::Multiply(
                        Box::new(ASTNode::Negate(Box::new(ASTNode::Number(1f64)))),
                        Box::new(ASTNode::Number(5f64))
                    )),
                    Box::new(ASTNode::Multiply(
                        Box::new(ASTNode::Number(2f64)),
                        Box::new(ASTNode::Number(4f64)),
                    )),
                ),
            })
        );
    }
}
