use crate::strings::{Match, PosStr, Position};

use super::Matcher;

pub struct TakeWhileMatcher<F> {
    f: F,
}

impl<F> TakeWhileMatcher<F> {
    pub fn new(f: F) -> Self {
        Self { f }
    }
}

impl<'a, F> Matcher<'a, Option<PosStr<'a>>> for TakeWhileMatcher<F>
where
    F: Fn(&Position, &char) -> bool,
{
    fn apply(&self, input: PosStr<'a>) -> Option<Match<'a, Option<PosStr<'a>>>> {
        Some(input.take_while_and_remainder(&self.f))
    }
}

// TODO tests
