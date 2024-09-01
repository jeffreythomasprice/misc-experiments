use std::marker::PhantomData;

use crate::strings::{Match, PosStr};

use super::Matcher;

pub struct MultipleMatcher<M, S, T, U> {
    m: M,
    s: S,
    phantom1: PhantomData<T>,
    phantom2: PhantomData<U>,
}

pub fn multiple<'a, M, S, T, U>(m: M, s: S) -> MultipleMatcher<M, S, T, U>
where
    M: Matcher<'a, T>,
    S: Matcher<'a, U>,
{
    MultipleMatcher::new(m, s)
}

impl<'a, M, S, T, U> MultipleMatcher<M, S, T, U>
where
    M: Matcher<'a, T>,
    S: Matcher<'a, U>,
{
    pub fn new(m: M, s: S) -> Self {
        Self {
            m,
            s,
            phantom1: PhantomData,
            phantom2: PhantomData,
        }
    }
}

impl<'a, M, S, T, U> Matcher<'a, Vec<T>> for MultipleMatcher<M, S, T, U>
where
    M: Matcher<'a, T>,
    S: Matcher<'a, U>,
{
    fn apply(&self, input: PosStr<'a>) -> Option<Match<'a, Vec<T>>> {
        let mut input = input;
        let mut results = Vec::new();

        // first
        match self.m.apply(input.clone()) {
            // we have at least one
            Some(Match { remainder, value }) => {
                results.push(value);
                input = remainder;
            }
            // no matches at all
            None => {
                // early exit with the empty results vector
                return Some(Match {
                    remainder: input,
                    value: results,
                });
            }
        };

        // as long as we can keep matching the skip then the actual match function, add to th results
        loop {
            match self.m.apply(self.s.skip(input.clone())) {
                Some(Match { remainder, value }) => {
                    results.push(value);
                    input = remainder;
                }
                None => break,
            };
        }

        Some(Match {
            remainder: input,
            value: results,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        matchers::{multiple::multiple, str::StrMatcher, Matcher},
        strings::{Match, PosStr, Position},
    };

    #[test]
    fn test() {
        assert_eq!(
            multiple(StrMatcher::new("aa"), StrMatcher::new("b")).apply("aabaabaa".into()),
            Some(Match {
                remainder: PosStr {
                    pos: Position { line: 0, column: 8 },
                    s: ""
                },
                value: vec!["aa", "aa", "aa"]
            })
        );
    }
}
