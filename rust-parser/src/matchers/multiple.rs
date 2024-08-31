use crate::strings::{Match, PosStr};

use super::Matcher;

pub struct MultipleMatcher<MatchF, SkipF> {
    mf: MatchF,
    sf: SkipF,
}

impl<MatchF, SkipF> MultipleMatcher<MatchF, SkipF> {
    pub fn new(mf: MatchF, sf: SkipF) -> Self {
        Self { mf, sf }
    }
}

impl<'a, T, MatchF, SkipF> Matcher<'a, Vec<T>> for MultipleMatcher<MatchF, SkipF>
where
    MatchF: Fn(PosStr<'a>) -> Option<Match<'a, T>>,
    SkipF: Fn(PosStr<'a>) -> PosStr<'a>,
{
    fn apply(&self, input: PosStr<'a>) -> Option<Match<'a, Vec<T>>> {
        let mut input = input;
        let mut results = Vec::new();

        // first
        match (self.mf)(input.clone()) {
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
            match (self.mf)((self.sf)(input.clone())) {
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

// TODO tests
