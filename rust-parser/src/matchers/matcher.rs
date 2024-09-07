use std::ops::Deref;

use crate::strings::{Match, PosStr};

pub trait Matcher<'a, T> {
    // TODO apply should return an error with a custom error type
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

// impl<'a, T, M, D> Matcher<'a, T> for D
// where
//     M: Matcher<'a, T>,
//     D: Deref<Target = M>,
// {
//     fn apply(&self, input: PosStr<'a>) -> Option<Match<'a, T>> {
//         self.apply(input)
//     }
// }

// impl<'a, T, M> Matcher<'a, T> for Box<M>
// where
//     M: Matcher<'a, T>,
// {
//     fn apply(&self, input: PosStr<'a>) -> Option<Match<'a, T>> {
//         self.apply(input)
//     }
// }

// TODO tests
