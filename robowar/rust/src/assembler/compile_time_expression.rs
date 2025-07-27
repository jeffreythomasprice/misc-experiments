use std::collections::HashMap;
use std::rc::Rc;

use chumsky::{Parser, error::Rich};
use chumsky::{extra::Err, prelude::*};

use crate::assembler::basic_types::*;

#[derive(Debug, Clone, PartialEq)]
pub enum EvaluateError {
    NameNotFound(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum AST {
    NumberLiteral(NumberLiteral),
    Identifier(String),
    Add(Box<AST>, Box<AST>),
    Subtract(Box<AST>, Box<AST>),
    Multiply(Box<AST>, Box<AST>),
    Divide(Box<AST>, Box<AST>),
    Modulo(Box<AST>, Box<AST>),
    Negate(Box<AST>),
}

impl AST {
    pub fn evaluate(
        &self,
        variables: &HashMap<String, NumberLiteral>,
    ) -> Result<NumberLiteral, EvaluateError> {
        match self {
            AST::NumberLiteral(result) => Ok(result.clone()),
            AST::Identifier(identifier) => match variables.get(identifier.as_str()) {
                Some(result) => Ok(result.clone()),
                None => Err(EvaluateError::NameNotFound(identifier.clone())),
            },
            AST::Add(left, right) => Self::binop_coerce_types(
                left.evaluate(variables)?,
                right.evaluate(variables)?,
                |left, right| left + right,
                |left, right| left + right,
                |left, right| left + right,
            ),
            AST::Subtract(left, right) => Self::binop_coerce_types(
                left.evaluate(variables)?,
                right.evaluate(variables)?,
                |left, right| left - right,
                |left, right| left - right,
                |left, right| left - right,
            ),
            AST::Multiply(left, right) => Self::binop_coerce_types(
                left.evaluate(variables)?,
                right.evaluate(variables)?,
                |left, right| left * right,
                |left, right| left * right,
                |left, right| left * right,
            ),
            AST::Divide(left, right) => Self::binop_coerce_types(
                left.evaluate(variables)?,
                right.evaluate(variables)?,
                |left, right| left / right,
                |left, right| left / right,
                |left, right| left / right,
            ),
            AST::Modulo(left, right) => Self::binop_coerce_types(
                left.evaluate(variables)?,
                right.evaluate(variables)?,
                |left, right| left % right,
                |left, right| left % right,
                |left, right| left % right,
            ),
            AST::Negate(result) => Ok(match result.evaluate(variables)? {
                NumberLiteral::I64(result) => NumberLiteral::I64(-result),
                NumberLiteral::U64(result) => NumberLiteral::I64(-(result as i64)),
                NumberLiteral::F64(result) => NumberLiteral::F64(-result),
            }),
        }
    }

    fn binop_coerce_types(
        left: NumberLiteral,
        right: NumberLiteral,
        f_u64: impl Fn(u64, u64) -> u64,
        f_i64: impl Fn(i64, i64) -> i64,
        f_f64: impl Fn(f64, f64) -> f64,
    ) -> Result<NumberLiteral, EvaluateError> {
        match (left, right) {
            // at least one is f64, we're doing float math
            (NumberLiteral::F64(left), NumberLiteral::F64(right)) => {
                Ok(NumberLiteral::F64(f_f64(left, right)))
            }
            (NumberLiteral::F64(left), NumberLiteral::I64(right)) => {
                Ok(NumberLiteral::F64(f_f64(left, right as f64)))
            }
            (NumberLiteral::F64(left), NumberLiteral::U64(right)) => {
                Ok(NumberLiteral::F64(f_f64(left, right as f64)))
            }
            (NumberLiteral::I64(left), NumberLiteral::F64(right)) => {
                Ok(NumberLiteral::F64(f_f64(left as f64, right)))
            }
            (NumberLiteral::U64(left), NumberLiteral::F64(right)) => {
                Ok(NumberLiteral::F64(f_f64(left as f64, right)))
            }

            // at least one is i64, we're doing signed integer math
            (NumberLiteral::I64(left), NumberLiteral::I64(right)) => {
                Ok(NumberLiteral::I64(f_i64(left, right)))
            }
            (NumberLiteral::I64(left), NumberLiteral::U64(right)) => {
                Ok(NumberLiteral::I64(f_i64(left, right as i64)))
            }
            (NumberLiteral::U64(left), NumberLiteral::I64(right)) => {
                Ok(NumberLiteral::I64(f_i64(left as i64, right)))
            }

            // all are u64, we're doing unsigned integer math
            (NumberLiteral::U64(left), NumberLiteral::U64(right)) => {
                Ok(NumberLiteral::U64(f_u64(left, right)))
            }
        }
    }
}

pub fn compile_time_expression<'a>() -> impl Parser<'a, &'a str, Box<AST>, Err<Rich<'a, char>>> {
    let number = Rc::new(number_literal().map(|x| Box::new(AST::NumberLiteral(x))));

    let identifier = Rc::new(identifier().map(|x| Box::new(AST::Identifier(x))));

    let atom = choice((number, identifier)).padded();

    recursive(|expression| {
        let terminal = choice((
            atom,
            just("-")
                .padded()
                .ignore_then(expression.clone())
                .map(|x| Box::new(AST::Negate(x))),
            just("(")
                .padded()
                .ignore_then(expression)
                .then_ignore(just(")").padded()),
        ));

        enum MulOp {
            Multiply,
            Divide,
            Modulo,
        }
        let mulop = terminal.clone().foldl(
            choice((
                just("*").map(|_| MulOp::Multiply),
                just("/").map(|_| MulOp::Divide),
                just("%").map(|_| MulOp::Modulo),
            ))
            .padded()
            .then(terminal)
            .repeated(),
            |a, (op, b)| match op {
                MulOp::Multiply => Box::new(AST::Multiply(a, b)),
                MulOp::Divide => Box::new(AST::Divide(a, b)),
                MulOp::Modulo => Box::new(AST::Modulo(a, b)),
            },
        );

        enum AddOp {
            Add,
            Subtract,
        }
        

        mulop.clone().foldl(
            choice((
                just("+").map(|_| AddOp::Add),
                just("-").map(|_| AddOp::Subtract),
            ))
            .padded()
            .then(mulop)
            .repeated(),
            |a, (op, b)| match op {
                AddOp::Add => Box::new(AST::Add(a, b)),
                AddOp::Subtract => Box::new(AST::Subtract(a, b)),
            },
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expression_1() {
        successful_expression_test(
            r" 1 + 2 * 3",
            AST::Add(
                Box::new(AST::NumberLiteral(NumberLiteral::I64(1))),
                Box::new(AST::Multiply(
                    Box::new(AST::NumberLiteral(NumberLiteral::I64(2))),
                    Box::new(AST::NumberLiteral(NumberLiteral::I64(3))),
                )),
            ),
            NumberLiteral::I64(7),
        );
    }

    #[test]
    fn expression_2() {
        successful_expression_test(
            r"(1  + 2      	)*12  /   
			4",
            AST::Divide(
                Box::new(AST::Multiply(
                    Box::new(AST::Add(
                        Box::new(AST::NumberLiteral(NumberLiteral::I64(1))),
                        Box::new(AST::NumberLiteral(NumberLiteral::I64(2))),
                    )),
                    Box::new(AST::NumberLiteral(NumberLiteral::I64(12))),
                )),
                Box::new(AST::NumberLiteral(NumberLiteral::I64(4))),
            ),
            NumberLiteral::I64(9),
        );
    }

    #[test]
    fn expression_3() {
        successful_expression_test(
            r"1.5*3",
            AST::Multiply(
                Box::new(AST::NumberLiteral(NumberLiteral::F64(1.5))),
                Box::new(AST::NumberLiteral(NumberLiteral::I64(3))),
            ),
            NumberLiteral::F64(4.5),
        );
    }

    #[test]
    fn expression_4() {
        successful_expression_test(
            r"5 % 2",
            AST::Modulo(
                Box::new(AST::NumberLiteral(NumberLiteral::I64(5))),
                Box::new(AST::NumberLiteral(NumberLiteral::I64(2))),
            ),
            NumberLiteral::I64(1),
        );
    }

    #[test]
    fn variables() {
        let parser = compile_time_expression();
        let mut variables = HashMap::new();
        variables.insert(
            "x".to_string(),
            parser
                .parse("1+2*3")
                .into_result()
                .unwrap()
                .evaluate(&variables)
                .unwrap(),
        );
        assert_eq!(
            variables,
            HashMap::from([("x".to_string(), NumberLiteral::I64(7)),])
        );
        variables.insert(
            "y".to_string(),
            parser
                .parse("x*2")
                .into_result()
                .unwrap()
                .evaluate(&variables)
                .unwrap(),
        );
        assert_eq!(
            variables,
            HashMap::from([
                ("x".to_string(), NumberLiteral::I64(7)),
                ("y".to_string(), NumberLiteral::I64(14)),
            ])
        );
        variables.insert(
            "x".to_string(),
            parser
                .parse("-x")
                .into_result()
                .unwrap()
                .evaluate(&variables)
                .unwrap(),
        );
        assert_eq!(
            variables,
            HashMap::from([
                ("x".to_string(), NumberLiteral::I64(-7)),
                ("y".to_string(), NumberLiteral::I64(14)),
            ])
        );
    }

    fn successful_expression_test(
        input: &str,
        expected_ast: AST,
        expected_eval_value: NumberLiteral,
    ) {
        let result = compile_time_expression().parse(input).into_result();
        assert_eq!(result, Ok(Box::new(expected_ast)));
        assert_eq!(
            result.unwrap().evaluate(&HashMap::new()).unwrap(),
            expected_eval_value
        );
    }
}
