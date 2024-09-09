use std::rc::Rc;

use crate::{
    matchers::{
        any2, any3, any4, char, char_range, defer, match2, match3, optional, repeat, specific_char,
        str, take_while, MapError, Mappable, Matcher, MatcherError, StrMappable,
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

enum AddOrSubtract {
    Add,
    Subtract,
}

enum MultiplyOrDivide {
    Multiply,
    Divide,
}

impl Parser {
    pub fn new() -> Self {
        let expression = defer::<Box<ASTNode>, _>();

        let number = {
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

            tokenize(
                match3(integer_part, fractional_part, exponent_part)
                    .map_to_str()
                    .map(|_, s| match s.parse::<f64>() {
                        Ok(x) => Ok(Box::new(ASTNode::Number(x))),
                        Err(e) => Err(MapError(format!(
                            "error parsing as float, input=\"{s}\", error={e:?}"
                        ))),
                    }),
            )
        };

        let negate = match2(tokenize(str("-")), expression).map(|_, (_, x)| Ok(x));

        let parenthesis =
            match3(tokenize(str("(")), expression, tokenize(str(")"))).map(|_, (_, x, _)| Ok(x));

        let term = any3(number, negate, parenthesis);

        let multiply_or_divide_operator = any2(
            tokenize(str("*")).map(|_, _| Ok(MultiplyOrDivide::Multiply)),
            tokenize(str("/")).map(|_, _| Ok(MultiplyOrDivide::Divide)),
        );
        let multiply_or_divide_list = binary_operator(
            Rc::new(term),
            multiply_or_divide_operator,
            |left, op, right| {
                Box::new(match op {
                    MultiplyOrDivide::Multiply => ASTNode::Multiply(left, right),
                    MultiplyOrDivide::Divide => ASTNode::Divide(left, right),
                })
            },
        );

        let add_or_subtract_operator = any2(
            tokenize(str("+")).map(|_, _| Ok(AddOrSubtract::Add)),
            tokenize(str("-")).map(|_, _| Ok(AddOrSubtract::Subtract)),
        );
        let add_or_subtract_list = binary_operator(
            Rc::new(multiply_or_divide_list),
            add_or_subtract_operator,
            |left, op, right| {
                Box::new(match op {
                    AddOrSubtract::Add => ASTNode::Add(left, right),
                    AddOrSubtract::Subtract => ASTNode::Subtract(left, right),
                })
            },
        );

        expression.set(add_or_subtract_list);

        Self {
            m: Box::new(expression),
        }
    }

    pub fn apply(
        &self,
        input: PosStr<'static>,
    ) -> Result<Match<'static, Box<ASTNode>>, MatcherError> {
        self.m.apply(input)
    }
}

fn tokenize<'a, T, M>(m: M) -> impl Matcher<'a, T>
where
    M: Matcher<'a, T>,
{
    match2(
        any4(
            specific_char(' '),
            specific_char('\t'),
            specific_char('\n'),
            specific_char('\r'),
        ),
        m,
    )
    .map(|_, (_, b)| Ok(b))
}

/**
For matchers M1 and M2, matches an alternating list of the form:
```
M1 (M2 M1)*
```

That is, a sequence of the first matcher separated by instances of the second matcher.

As it progresses left to right each pair of elements and the separator between them will be passed to the given function. The resulting element will form the new first element.
e.g. for the given input sequence
```
a b c d e
a,c,e are instances of T1
b,d are instances of T2
```
The return value will be equivalent to:
```
f(f(a, b, c), d, e)
```
*/
fn binary_operator<'a, T1, M1, T2, M2, F>(m1: M1, m2: M2, f: F) -> impl Matcher<'a, T1>
where
    M1: Matcher<'a, T1> + Clone,
    M2: Matcher<'a, T2>,
    F: Fn(T1, T2, T1) -> T1 + 'a,
{
    match2(m1.clone(), repeat(match2(m2, m1), ..)).map(move |_pos, (first, remainder)| {
        let mut result = first;
        for (op, right) in remainder {
            result = f(result, op, right);
        }
        Ok(result)
    })
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
