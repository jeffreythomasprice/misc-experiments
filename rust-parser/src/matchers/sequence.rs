use crate::strings::{Match, PosStr};

use super::Matcher;

pub struct Match2<M1, M2> {
    m1: M1,
    m2: M2,
}

pub fn match2<'a, T1, M1, T2, M2>(m1: M1, m2: M2) -> Match2<M1, M2>
where
    M1: Matcher<'a, T1>,
    M2: Matcher<'a, T2>,
{
    Match2::new(m1, m2)
}

impl<M1, M2> Match2<M1, M2> {
    pub fn new(m1: M1, m2: M2) -> Self {
        Self { m1, m2 }
    }
}

impl<'a, T1, M1, T2, M2> Matcher<'a, (T1, T2)> for Match2<M1, M2>
where
    M1: Matcher<'a, T1>,
    M2: Matcher<'a, T2>,
{
    fn apply(&self, input: PosStr<'a>) -> Option<Match<'a, (T1, T2)>> {
        let (input, result1) = match self.m1.apply(input) {
            Some(Match { remainder, value }) => (remainder, value),
            None => return None,
        };
        let (input, result2) = match self.m2.apply(input) {
            Some(Match { remainder, value }) => (remainder, value),
            None => return None,
        };
        Some(Match {
            remainder: input,
            value: (result1, result2),
        })
    }
}

pub struct Match3<M1, M2, M3> {
    m1: M1,
    m2: M2,
    m3: M3,
}

pub fn match3<'a, T1, M1, T2, M2, T3, M3>(m1: M1, m2: M2, m3: M3) -> Match3<M1, M2, M3>
where
    M1: Matcher<'a, T1>,
    M2: Matcher<'a, T2>,
    M3: Matcher<'a, T3>,
{
    Match3::new(m1, m2, m3)
}

impl<M1, M2, M3> Match3<M1, M2, M3> {
    pub fn new(m1: M1, m2: M2, m3: M3) -> Self {
        Self { m1, m2, m3 }
    }
}

impl<'a, T1, M1, T2, M2, T3, M3> Matcher<'a, (T1, T2, T3)> for Match3<M1, M2, M3>
where
    M1: Matcher<'a, T1>,
    M2: Matcher<'a, T2>,
    M3: Matcher<'a, T3>,
{
    fn apply(&self, input: PosStr<'a>) -> Option<Match<'a, (T1, T2, T3)>> {
        let (input, result1) = match self.m1.apply(input) {
            Some(Match { remainder, value }) => (remainder, value),
            None => return None,
        };
        let (input, result2) = match self.m2.apply(input) {
            Some(Match { remainder, value }) => (remainder, value),
            None => return None,
        };
        let (input, result3) = match self.m3.apply(input) {
            Some(Match { remainder, value }) => (remainder, value),
            None => return None,
        };
        Some(Match {
            remainder: input,
            value: (result1, result2, result3),
        })
    }
}

// TODO tests
