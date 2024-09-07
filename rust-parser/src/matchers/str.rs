use crate::strings::{Match, PosStr};

use super::{Matcher, MatcherError};

pub struct StrMatcher<'a> {
    s: &'a str,
}

pub fn str(s: &str) -> StrMatcher {
    StrMatcher { s }
}

impl<'a> Matcher<'a, &'a str> for StrMatcher<'a> {
    fn apply(&self, input: PosStr<'a>) -> Result<Match<'a, &'a str>, MatcherError> {
        if input.s.len() < self.s.len() {
            return Err(MatcherError::NotEnoughRemainingInput(input.pos));
        }
        let original_pos = input.pos;
        let mut pos = input.pos;
        for (input, check) in input.s.chars().zip(self.s.chars()) {
            if input != check {
                return Err(MatcherError::Expected(original_pos, self.s.to_owned()));
            }
            pos = pos.advance(&check);
        }
        Ok(Match {
            source: input,
            matched: PosStr {
                pos: input.pos,
                s: &input.s[0..self.s.len()],
            },
            remainder: PosStr {
                pos,
                s: &input.s[self.s.len()..],
            },
            value: &input.s[0..self.s.len()],
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        matchers::{str, Matcher, MatcherError},
        strings::{Match, PosStr, Position},
    };

    #[test]
    fn some() {
        assert_eq!(
            str("foo").apply("foobar".into()),
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
                value: "foo"
            })
        );
    }

    #[test]
    fn none() {
        assert_eq!(
            str("foo").apply("barfoo".into()),
            Err(MatcherError::Expected(
                Position { line: 0, column: 0 },
                "foo".to_owned()
            ))
        );
    }

    #[test]
    fn none_input_is_too_small() {
        assert_eq!(
            str("foo").apply("f".into()),
            Err(MatcherError::NotEnoughRemainingInput(Position {
                line: 0,
                column: 0
            }))
        );
    }
}
