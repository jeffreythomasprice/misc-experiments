use std::{marker::PhantomData, ops::RangeBounds};

use crate::strings::{Match, PosStr};

use super::{Matcher, MatcherError};

pub struct RepeatMatcher<M, T, R> {
    matcher: M,
    range: R,
    phantom: PhantomData<T>,
}

pub fn repeat<'a, M, T, R>(matcher: M, range: R) -> RepeatMatcher<M, T, R>
where
    M: Matcher<'a, T>,
    R: RangeBounds<usize>,
{
    RepeatMatcher {
        matcher,
        range,
        phantom: PhantomData,
    }
}

impl<'a, M, T, R> Matcher<'a, Vec<T>> for RepeatMatcher<M, T, R>
where
    M: Matcher<'a, T>,
    R: RangeBounds<usize>,
{
    fn apply(&self, input: PosStr<'a>) -> Result<Match<'a, Vec<T>>, MatcherError> {
        let original_input = input;
        let mut remaining_input = original_input;
        let mut results = Vec::new();
        let mut last_error = None;
        let mut total_len = 0;

        loop {
            let good_now = self.range.contains(&results.len());
            let will_be_good_in_one_more = self.range.contains(&(results.len() + 1));
            if good_now && !will_be_good_in_one_more {
                break;
            }

            match self.matcher.apply(remaining_input) {
                Ok(Match {
                    source: _,
                    matched,
                    remainder,
                    value,
                }) => {
                    results.push(value);
                    remaining_input = remainder;
                    total_len += matched.s.len();
                }
                Err(e) => {
                    last_error = Some(e);
                    break;
                }
            };
        }

        if self.range.contains(&results.len()) {
            Ok(Match {
                source: original_input,
                matched: PosStr {
                    pos: original_input.pos,
                    s: &original_input.s[0..total_len],
                },
                remainder: remaining_input,
                value: results,
            })
        } else {
            Err(last_error.unwrap_or(MatcherError::NotEnoughRemainingInput(remaining_input.pos)))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        matchers::{repeat, str, Matcher, MatcherError},
        strings::{Match, PosStr, Position},
    };

    #[test]
    fn unlimited() {
        assert_eq!(
            repeat(str("foo"), ..).apply("foofoofoobar".into()),
            Ok(Match {
                source: PosStr {
                    pos: Position { line: 0, column: 0 },
                    s: "foofoofoobar"
                },
                matched: PosStr {
                    pos: Position { line: 0, column: 0 },
                    s: "foofoofoo"
                },
                remainder: PosStr {
                    pos: Position { line: 0, column: 9 },
                    s: "bar"
                },
                value: vec!["foo", "foo", "foo"]
            })
        );
    }

    #[test]
    fn at_least_success() {
        assert_eq!(
            repeat(str("foo"), 2..).apply("foofoofoobar".into()),
            Ok(Match {
                source: PosStr {
                    pos: Position { line: 0, column: 0 },
                    s: "foofoofoobar"
                },
                matched: PosStr {
                    pos: Position { line: 0, column: 0 },
                    s: "foofoofoo"
                },
                remainder: PosStr {
                    pos: Position { line: 0, column: 9 },
                    s: "bar"
                },
                value: vec!["foo", "foo", "foo"]
            })
        );
    }

    #[test]
    fn at_least_failure() {
        assert_eq!(
            repeat(str("foo"), 2..).apply("foobar".into()),
            Err(MatcherError::Expected(
                Position { line: 0, column: 3 },
                "foo".to_owned()
            ))
        );
    }

    #[test]
    fn no_more_than_success() {
        assert_eq!(
            repeat(str("foo"), ..2).apply("foofoofoobar".into()),
            Ok(Match {
                source: PosStr {
                    pos: Position { line: 0, column: 0 },
                    s: "foofoofoobar"
                },
                matched: PosStr {
                    pos: Position { line: 0, column: 0 },
                    s: "foo"
                },
                remainder: PosStr {
                    pos: Position { line: 0, column: 3 },
                    s: "foofoobar"
                },
                value: vec!["foo"]
            })
        );
    }

    #[test]
    fn no_more_than_success_empty_result() {
        assert_eq!(
            repeat(str("foo"), ..2).apply("bar".into()),
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
                value: vec![]
            })
        );
    }

    #[test]
    fn bounded_success() {
        assert_eq!(
            repeat(str("foo"), 2..=3).apply("foofoofoofoobar".into()),
            Ok(Match {
                source: PosStr {
                    pos: Position { line: 0, column: 0 },
                    s: "foofoofoofoobar"
                },
                matched: PosStr {
                    pos: Position { line: 0, column: 0 },
                    s: "foofoofoo"
                },
                remainder: PosStr {
                    pos: Position { line: 0, column: 9 },
                    s: "foobar"
                },
                value: vec!["foo", "foo", "foo"]
            })
        );
    }

    #[test]
    fn bounded_failure_too_few() {
        assert_eq!(
            repeat(str("foo"), 2..=3).apply("foobar".into()),
            Err(MatcherError::Expected(
                Position { line: 0, column: 3 },
                "foo".to_owned()
            ))
        );
    }
}
