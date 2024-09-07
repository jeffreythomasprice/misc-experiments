use crate::strings::{Match, PosStr};

use super::{Matcher, MatcherError};

pub struct OptionalMatcher<M> {
    m: M,
}

pub fn optional<'a, M>(m: M) -> OptionalMatcher<M> {
    OptionalMatcher { m }
}

impl<'a, T, M> Matcher<'a, Option<T>> for OptionalMatcher<M>
where
    M: Matcher<'a, T>,
{
    fn apply(&self, input: PosStr<'a>) -> Result<Match<'a, Option<T>>, MatcherError> {
        match self.m.apply(input) {
            Ok(Match {
                source,
                matched,
                remainder,
                value,
            }) => Ok(Match {
                source,
                matched,
                remainder,
                value: Some(value),
            }),
            Err(_) => Ok(Match {
                source: input,
                matched: PosStr {
                    pos: input.pos,
                    s: "",
                },
                remainder: input,
                value: None,
            }),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use crate::{
        matchers::{str, Matcher},
        strings::{Match, PosStr, Position},
    };

    use super::optional;

    #[test]
    pub fn some() {
        assert_eq!(
            optional(str("foo")).apply("foobar".into()),
            Ok(Match {
                source: PosStr {
                    pos: Position { line: 0, column: 0 },
                    s: "foobar"
                },
                matched: PosStr {
                    pos: Position { line: 0, column: 0 },
                    s: "foo"
                },
                remainder: PosStr {
                    pos: Position { line: 0, column: 3 },
                    s: "bar"
                },
                value: Some("foo")
            })
        );
    }

    #[test]
    pub fn none() {
        assert_eq!(
            optional(str("foo")).apply("bar".into()),
            Ok(Match {
                source: PosStr {
                    pos: Position { line: 0, column: 0 },
                    s: "bar"
                },
                matched: PosStr {
                    pos: Position { line: 0, column: 0 },
                    s: ""
                },
                remainder: PosStr {
                    pos: Position { line: 0, column: 0 },
                    s: "bar"
                },
                value: None
            })
        );
    }
}
