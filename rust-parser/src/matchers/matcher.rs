use crate::strings::{Match, PosStr};

pub trait Matcher<'a, T> {
    fn apply(&self, input: PosStr<'a>) -> Option<Match<'a, T>>;

    fn skip(&self, input: PosStr<'a>) -> PosStr<'a> {
        match self.apply(input.clone()) {
            Some(Match {
                remainder,
                value: _,
            }) => remainder,
            None => input,
        }
    }
}

// TODO tests