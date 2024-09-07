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
        match self.m.apply(input.clone()) {
            Ok(Match {
                pos,
                remainder,
                value,
            }) => Ok(Match {
                pos,
                remainder,
                value: Some(value),
            }),
            Err(_) => Ok(Match {
                pos: input.pos.clone(),
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
                pos: Position { line: 0, column: 0 },
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
                pos: Position { line: 0, column: 0 },
                remainder: PosStr {
                    pos: Position { line: 0, column: 0 },
                    s: "bar"
                },
                value: None
            })
        );
    }
}
