use std::{marker::PhantomData, ops::RangeBounds};

use crate::strings::{Match, PosStr};

use super::{Matcher, MatcherError};

pub struct RepeatMatcher<M, T, R> {
    matcher: M,
    phantom1: PhantomData<T>,
    range: R,
}

pub fn repeat<'a, M, T, R>(matcher: M, range: R) -> RepeatMatcher<M, T, R>
where
    M: Matcher<'a, T>,
    R: RangeBounds<usize>,
{
    RepeatMatcher::new(matcher, range)
}

impl<'a, M, T, R> RepeatMatcher<M, T, R>
where
    M: Matcher<'a, T>,
    R: RangeBounds<usize>,
{
    pub fn new(matcher: M, range: R) -> Self {
        Self {
            matcher,
            range,
            phantom1: PhantomData,
        }
    }
}

impl<'a, M, T, R> Matcher<'a, Vec<T>> for RepeatMatcher<M, T, R>
where
    M: Matcher<'a, T>,
    R: RangeBounds<usize>,
{
    fn apply(&self, input: PosStr<'a>) -> Result<Match<'a, Vec<T>>, MatcherError> {
        let original_input = input;
        let mut remaining_input = original_input.clone();
        let mut results = Vec::new();
        let mut last_error = None;

        loop {
            let good_now = self.range.contains(&results.len());
            let will_be_good_in_one_more = self.range.contains(&(results.len() + 1));
            if good_now && !will_be_good_in_one_more {
                break;
            }

            match self.matcher.apply(remaining_input.clone()) {
                Ok(Match { remainder, value }) => {
                    results.push(value);
                    remaining_input = remainder;
                }
                Err(e) => {
                    last_error = Some(e);
                    break;
                }
            };
        }

        if self.range.contains(&results.len()) {
            Ok(Match {
                remainder: remaining_input,
                value: results,
            })
        } else {
            Err(last_error.unwrap_or(MatcherError::NotEnoughRemainingInput))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        matchers::{repeat, str::StrMatcher, Matcher, MatcherError},
        strings::{Match, PosStr, Position},
    };

    #[test]
    fn unlimited() {
        assert_eq!(
            repeat(StrMatcher::new("foo"), ..).apply("foofoofoobar".into()),
            Ok(Match {
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
            repeat(StrMatcher::new("foo"), 2..).apply("foofoofoobar".into()),
            Ok(Match {
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
            repeat(StrMatcher::new("foo"), 2..).apply("foobar".into()),
            Err(MatcherError::Expected(
                Position { line: 0, column: 3 },
                "foo".to_owned()
            ))
        );
    }

    #[test]
    fn no_more_than_success() {
        assert_eq!(
            repeat(StrMatcher::new("foo"), ..2).apply("foofoofoobar".into()),
            Ok(Match {
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
            repeat(StrMatcher::new("foo"), ..2).apply("bar".into()),
            Ok(Match {
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
            repeat(StrMatcher::new("foo"), 2..=3).apply("foofoofoofoobar".into()),
            Ok(Match {
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
            repeat(StrMatcher::new("foo"), 2..=3).apply("foobar".into()),
            Err(MatcherError::Expected(
                Position { line: 0, column: 3 },
                "foo".to_owned()
            ))
        );
    }
}
