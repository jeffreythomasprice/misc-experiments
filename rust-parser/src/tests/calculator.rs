use std::fmt::Debug;

use crate::{
    matchers::{any2, any3, binary_list, char_range, specific_char, MatchError},
    strings::{Match, PosStr},
};

#[derive(Debug, Clone, PartialEq)]
enum ASTNode {
    Number(f64),
    Add(Box<ASTNode>, Box<ASTNode>),
    Subtract(Box<ASTNode>, Box<ASTNode>),
    Multiply(Box<ASTNode>, Box<ASTNode>),
    Divide(Box<ASTNode>, Box<ASTNode>),
    Negate(Box<ASTNode>),
}

impl ASTNode {
    pub fn eval(&self) -> f64 {
        match self {
            ASTNode::Number(value) => *value,
            ASTNode::Add(left, right) => left.eval() + right.eval(),
            ASTNode::Subtract(left, right) => left.eval() - right.eval(),
            ASTNode::Multiply(left, right) => left.eval() * right.eval(),
            ASTNode::Divide(left, right) => left.eval() / right.eval(),
            ASTNode::Negate(value) => -value.eval(),
        }
    }
}

fn skip_whitespace<'a>(input: PosStr<'a>) -> PosStr<'a> {
    input
        .take_while_and_remainder(|_, c| c.is_whitespace())
        .remainder
}

fn parse_number(input: PosStr) -> Result<Match<ASTNode>, MatchError> {
    /*
    https://www.json.org/json-en.html
    https://stackoverflow.com/a/13340826
    -?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?

    ignoring the leading negation because we have an AST node for that
    */

    let input = skip_whitespace(input);

    // 0|[1-9]\d*
    let first_integer_digit = char_range(input, '0'..='9')?;
    let remainder = if ('1'..='9').contains(&first_integer_digit.value) {
        let r = first_integer_digit
            .remainder
            .take_while_and_remainder(|_, c| ('0'..='9').contains(c));
        r.remainder
    } else {
        first_integer_digit.remainder
    };

    // (\.\d+)?
    let remainder = if let Ok(dot_match) = specific_char(remainder, '.') {
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

    let full_match = input.take_until_position_and_remainder(&remainder.pos)?;
    match full_match.value.s.parse() {
        Ok(value) => Ok(full_match.map(|_| ASTNode::Number(value))),
        Err(e) => Err(MatchError::Parse {
            expected: "number".to_owned(),
            got: full_match.value.s.to_owned(),
            error: format!("{e:?}"),
        }),
    }
}

fn parse_negated_number(input: PosStr) -> Result<Match<ASTNode>, MatchError> {
    let remainder = specific_char(skip_whitespace(input), '-')?.remainder;
    Ok(parse_number(remainder)?.map(|x| ASTNode::Negate(Box::new(x))))
}

fn parse_parenthesis(input: PosStr) -> Result<Match<ASTNode>, MatchError> {
    let remainder = specific_char(skip_whitespace(input), '(')?.remainder;
    let result = parse_expression(remainder)?;
    let remainder = specific_char(skip_whitespace(result.remainder), ')')?.remainder;
    Ok(Match {
        value: result.value,
        remainder: remainder,
    })
}

fn parse_term(input: PosStr) -> Result<Match<ASTNode>, MatchError> {
    any3(input, parse_number, parse_negated_number, parse_parenthesis).map_err(|e| {
        MatchError::Expected {
            expected: "expression".to_owned(),
            got: format!("{e:?}"),
        }
    })
}

fn parse_mulops(input: PosStr) -> Result<Match<ASTNode>, MatchError> {
    enum Op {
        Multiply,
        Divide,
    }
    Ok(binary_list(
        input,
        parse_term,
        |input| {
            any2(
                skip_whitespace(input),
                |input| specific_char(input, '*').map(|x| x.map(|_| Op::Multiply)),
                |input| specific_char(input, '/').map(|x| x.map(|_| Op::Divide)),
            )
            .map_err(|e| MatchError::Expected {
                expected: "* or /".to_owned(),
                got: format!("{e:?}"),
            })
        },
        1..,
    )?
    .map(|results| {
        // unwrap is safe because range constraint
        let (first, remainder) = results.unwrap();
        remainder
            .into_iter()
            .fold(first, |left, (op, right)| match op {
                Op::Multiply => ASTNode::Multiply(Box::new(left), Box::new(right)),
                Op::Divide => ASTNode::Divide(Box::new(left), Box::new(right)),
            })
    }))
}

fn parse_addops(input: PosStr) -> Result<Match<ASTNode>, MatchError> {
    enum Op {
        Add,
        Subtract,
    }
    Ok(binary_list(
        input,
        parse_mulops,
        |input| {
            any2(
                skip_whitespace(input),
                |input| specific_char(input, '+').map(|x| x.map(|_| Op::Add)),
                |input| specific_char(input, '-').map(|x| x.map(|_| Op::Subtract)),
            )
            .map_err(|e| MatchError::Expected {
                expected: "+ or -".to_owned(),
                got: format!("{e:?}"),
            })
        },
        1..,
    )?
    .map(|results| {
        // unwrap is safe because range constraint
        let (first, remainder) = results.unwrap();
        remainder
            .into_iter()
            .fold(first, |left, (op, right)| match op {
                Op::Add => ASTNode::Add(Box::new(left), Box::new(right)),
                Op::Subtract => ASTNode::Subtract(Box::new(left), Box::new(right)),
            })
    }))
}

fn parse_expression(input: PosStr) -> Result<Match<ASTNode>, MatchError> {
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
        let result = parse_expression("  1.5".into());
        assert_eq!(
            result,
            Ok(Match {
                value: ASTNode::Number(1.5f64),
                remainder: PosStr {
                    pos: Position { line: 0, column: 5 },
                    s: ""
                },
            })
        );
        assert_eq!(result.unwrap().value.eval(), 1.5f64);
    }

    #[test]
    fn negated_number() {
        let result = parse_expression("-7.1".into());
        assert_eq!(
            result,
            Ok(Match {
                value: ASTNode::Negate(Box::new(ASTNode::Number(7.1f64))),
                remainder: PosStr {
                    pos: Position { line: 0, column: 4 },
                    s: ""
                },
            })
        );
        assert_eq!(result.unwrap().value.eval(), -7.1f64);
    }

    #[test]
    fn number_and_remainder() {
        let result = parse_expression("1 2".into());
        assert_eq!(
            result,
            Ok(Match {
                value: ASTNode::Number(1f64),
                remainder: PosStr {
                    pos: Position { line: 0, column: 1 },
                    s: " 2"
                },
            })
        );
        assert_eq!(result.unwrap().value.eval(), 1f64);
    }

    #[test]
    fn addition() {
        let result = parse_expression("1 + 2".into());
        assert_eq!(
            result,
            Ok(Match {
                value: ASTNode::Add(
                    Box::new(ASTNode::Number(1f64)),
                    Box::new(ASTNode::Number(2f64))
                ),
                remainder: PosStr {
                    pos: Position { line: 0, column: 5 },
                    s: ""
                },
            })
        );
        assert_eq!(result.unwrap().value.eval(), 3f64);
    }

    #[test]
    fn parenthesis() {
        let result = parse_expression("(1 + 2)*3".into());
        assert_eq!(
            result,
            Ok(Match {
                value: ASTNode::Multiply(
                    Box::new(ASTNode::Add(
                        Box::new(ASTNode::Number(1f64)),
                        Box::new(ASTNode::Number(2f64))
                    )),
                    Box::new(ASTNode::Number(3f64))
                ),
                remainder: PosStr {
                    pos: Position { line: 0, column: 9 },
                    s: ""
                },
            })
        );
        assert_eq!(result.unwrap().value.eval(), 9f64);
    }

    #[test]
    fn subtraction() {
        let result = parse_expression("1.5-2.5".into());
        assert_eq!(
            result,
            Ok(Match {
                value: ASTNode::Subtract(
                    Box::new(ASTNode::Number(1.5f64)),
                    Box::new(ASTNode::Number(2.5f64))
                ),
                remainder: PosStr {
                    pos: Position { line: 0, column: 7 },
                    s: ""
                },
            })
        );
        assert_eq!(result.unwrap().value.eval(), -1f64);
    }

    #[test]
    fn order_of_operations_1() {
        let result = parse_expression("-1*5/2+4".into());
        assert_eq!(
            result,
            Ok(Match {
                value: ASTNode::Add(
                    Box::new(ASTNode::Divide(
                        Box::new(ASTNode::Multiply(
                            Box::new(ASTNode::Negate(Box::new(ASTNode::Number(1f64)))),
                            Box::new(ASTNode::Number(5f64))
                        )),
                        Box::new(ASTNode::Number(2f64))
                    )),
                    Box::new(ASTNode::Number(4f64))
                ),
                remainder: PosStr {
                    pos: Position { line: 0, column: 8 },
                    s: ""
                },
            })
        );
        assert_eq!(result.unwrap().value.eval(), 1.5f64);
    }

    #[test]
    fn order_of_operations_2() {
        let result = parse_expression("-1*5-2*4".into());
        assert_eq!(
            result,
            Ok(Match {
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
                remainder: PosStr {
                    pos: Position { line: 0, column: 8 },
                    s: ""
                },
            })
        );
        assert_eq!(result.unwrap().value.eval(), -13f64);
    }

    #[test]
    fn many_additions() {
        let result = parse_expression("1+2+3+4+5".into());
        assert_eq!(
            result,
            Ok(Match {
                value: ASTNode::Add(
                    Box::new(ASTNode::Add(
                        Box::new(ASTNode::Add(
                            Box::new(ASTNode::Add(
                                Box::new(ASTNode::Number(1f64)),
                                Box::new(ASTNode::Number(2f64)),
                            )),
                            Box::new(ASTNode::Number(3f64))
                        )),
                        Box::new(ASTNode::Number(4f64))
                    )),
                    Box::new(ASTNode::Number(5f64))
                ),
                remainder: PosStr {
                    pos: Position { line: 0, column: 9 },
                    s: ""
                },
            })
        );
        assert_eq!(result.unwrap().value.eval(), 15f64);
    }

    #[test]
    fn many_multiplications() {
        let result = parse_expression("1*2*3*4*5".into());
        assert_eq!(
            result,
            Ok(Match {
                value: ASTNode::Multiply(
                    Box::new(ASTNode::Multiply(
                        Box::new(ASTNode::Multiply(
                            Box::new(ASTNode::Multiply(
                                Box::new(ASTNode::Number(1f64)),
                                Box::new(ASTNode::Number(2f64)),
                            )),
                            Box::new(ASTNode::Number(3f64))
                        )),
                        Box::new(ASTNode::Number(4f64))
                    )),
                    Box::new(ASTNode::Number(5f64))
                ),
                remainder: PosStr {
                    pos: Position { line: 0, column: 9 },
                    s: ""
                },
            })
        );
        assert_eq!(result.unwrap().value.eval(), 120f64);
    }
}
