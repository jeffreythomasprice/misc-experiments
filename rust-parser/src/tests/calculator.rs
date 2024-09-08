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
                    .map(|_, s| match s.parse::<f64>() {
                        Ok(x) => Ok(Box::new(ASTNode::Number(x))),
                        Err(e) => Err(MapError(format!(
                            "error parsing as float, input=\"{s}\", error={e:?}"
                        ))),
                    }),
            )
        };

        /*
        TODO more matchers

        expression = add_or_subtract_list

        add_or_subtract_list = multiply_or_divide_list (("+" | "-") multiply_or_divide_list)*

        multiply_or_divide_list = term (("*" | "/") term)*

        term =
            | number
            | "-" expression
            | "(" expression ")"
        */

        Self { m: match_f64 }
    }

    pub fn apply(
        &self,
        input: PosStr<'static>,
    ) -> Result<Match<'static, Box<ASTNode>>, MatcherError> {
        self.m.apply(input)
    }
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
    M1: Matcher<'a, T1>,
    M2: Matcher<'a, T2>,
    F: Fn(T1, T2, T1) -> T1,
{
    match2(m1, repeat(match2(m2, m1), ..)).map(|pos, value| todo!())
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
