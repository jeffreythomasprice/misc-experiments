use std::marker::PhantomData;

use crate::strings::{Match, PosStr};

use super::{Matcher, MatcherError};

pub struct MapError(pub String);

pub struct MapMatcher<'a, T, R, M, F>
where
    M: Matcher<'a, T>,
    F: Fn(T) -> Result<R, MapError>,
{
    m: M,
    f: F,
    phantom1: PhantomData<&'a usize>,
    phantom2: PhantomData<T>,
    phantom3: PhantomData<R>,
}

pub trait Mappable<'a, T, R, M, F>
where
    M: Matcher<'a, T>,
    F: Fn(T) -> Result<R, MapError>,
{
    fn map(self, f: F) -> MapMatcher<'a, T, R, M, F>;
}

impl<'a, T, R, M, F> Mappable<'a, T, R, M, F> for M
where
    M: Matcher<'a, T>,
    F: Fn(T) -> Result<R, MapError>,
{
    fn map(self, f: F) -> MapMatcher<'a, T, R, M, F> {
        MapMatcher {
            m: self,
            f,
            phantom1: PhantomData,
            phantom2: PhantomData,
            phantom3: PhantomData,
        }
    }
}

impl<'a, T, R, M, F> Matcher<'a, R> for MapMatcher<'a, T, R, M, F>
where
    M: Matcher<'a, T>,
    F: Fn(T) -> Result<R, MapError>,
{
    fn apply(&self, input: PosStr<'a>) -> Result<Match<'a, R>, MatcherError> {
        match self.m.apply(input.clone()).map(
            |Match {
                 pos,
                 remainder,
                 value,
             }| {
                (self.f)(value).map(|value| Match {
                    pos,
                    remainder,
                    value,
                })
            },
        ) {
            Ok(Ok(result)) => Ok(result),
            Ok(Err(e)) => Err(MatcherError::Expected(input.pos.clone(), e.0)),
            Err(e) => Err(e),
        }
    }
}
