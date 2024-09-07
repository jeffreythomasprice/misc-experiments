use std::{fmt::Debug, ops::RangeBounds};

use crate::strings::{Match, PosStr};

use super::{MapError, Mappable, Matcher, MatcherError};

pub struct CharMatcher {}

pub fn char() -> CharMatcher {
    CharMatcher {}
}

impl<'a> Matcher<'a, char> for CharMatcher {
    fn apply(&self, input: PosStr<'a>) -> Result<Match<'a, char>, MatcherError> {
        let first = input.s.char_indices().next();
        match first {
            Some((i, c)) => Ok(Match {
                source: input,
                matched: PosStr {
                    pos: input.pos,
                    s: &input.s[0..=i],
                },
                remainder: PosStr {
                    pos: input.pos.advance(&c),
                    s: &input.s[(i + c.len_utf8())..],
                },
                value: c,
            }),
            None => Err(MatcherError::NotEnoughRemainingInput(input.pos)),
        }
    }
}

pub fn specific_char<'a>(desired: char) -> impl Matcher<'a, char> {
    char().map(move |_, c| {
        if c == desired {
            Ok(c)
        } else {
            Err(MapError(format!("expected {desired}, got {c}")))
        }
    })
}

pub fn char_range<'a, R>(range: R) -> impl Matcher<'a, char>
where
    R: RangeBounds<char> + Debug + 'a,
{
    char().map(move |_, c| {
        if range.contains(&c) {
            Ok(c)
        } else {
            Err(MapError(format!("expected {range:?}, got {c}")))
        }
    })
}

#[cfg(test)]
mod tests {
    use crate::{
        matchers::{char, Matcher, MatcherError},
        strings::{Match, PosStr, Position},
    };

    #[test]
    fn success() {
        assert_eq!(
            char().apply("abc".into()),
            Ok(Match {
                source: PosStr {
                    pos: Position { line: 0, column: 0 },
                    s: "abc",
                },
                matched: PosStr {
                    pos: Position { line: 0, column: 0 },
                    s: "a",
                },
                remainder: PosStr {
                    pos: Position { line: 0, column: 1 },
                    s: "bc",
                },
                value: 'a',
            })
        );
    }

    #[test]
    fn failure() {
        assert_eq!(
            char().apply("".into()),
            Err(MatcherError::NotEnoughRemainingInput(Position {
                line: 0,
                column: 0
            }))
        );
    }

    // TODO tests for specific char
    // TODO tests for char range
}
