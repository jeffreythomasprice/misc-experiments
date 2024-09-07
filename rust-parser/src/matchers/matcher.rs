use crate::strings::{Match, PosStr, Position};

#[derive(Debug, Clone, PartialEq)]
pub enum MatcherError {
    Expected(Position, String),
    NotEnoughRemainingInput,
}

pub trait Matcher<'a, T> {
    fn apply(&self, input: PosStr<'a>) -> Result<Match<'a, T>, MatcherError>;

    fn skip(&self, input: PosStr<'a>) -> Result<PosStr<'a>, MatcherError> {
        self.apply(input.clone()).map(
            |Match {
                 remainder,
                 value: _,
             }| remainder,
        )
    }
}

// TODO tests
