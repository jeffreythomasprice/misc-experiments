use std::marker::PhantomData;

use crate::strings::{Match, PosStr};

use super::{Matcher, MatcherError};

pub struct MapError(pub String);

pub struct MapMatcher<'a, T, R, M, F>
where
    M: Matcher<'a, T>,
    F: Fn(&PosStr<'a>, T) -> Result<R, MapError>,
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
    F: Fn(&PosStr<'a>, T) -> Result<R, MapError>,
{
    fn map(self, f: F) -> MapMatcher<'a, T, R, M, F>;
}

impl<'a, T, R, M, F> Mappable<'a, T, R, M, F> for M
where
    M: Matcher<'a, T>,
    F: Fn(&PosStr<'a>, T) -> Result<R, MapError>,
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

pub trait StrMappable<'a, T, M>
where
    M: Matcher<'a, T>,
{
    fn map_to_str(self) -> impl Matcher<'a, &'a str>;
}

impl<'a, T, M> StrMappable<'a, T, M> for M
where
    M: Matcher<'a, T>,
{
    fn map_to_str(self) -> impl Matcher<'a, &'a str> {
        self.map(|s, _| Ok(s.s))
    }
}

impl<'a, T, R, M, F> Matcher<'a, R> for MapMatcher<'a, T, R, M, F>
where
    M: Matcher<'a, T>,
    F: Fn(&PosStr<'a>, T) -> Result<R, MapError>,
{
    fn apply(&self, input: PosStr<'a>) -> Result<Match<'a, R>, MatcherError> {
        match self.m.apply(input).map(
            |Match {
                 source,
                 matched,
                 remainder,
                 value,
             }| {
                (self.f)(&matched, value).map(|value| Match {
                    source,
                    matched,
                    remainder,
                    value,
                })
            },
        ) {
            Ok(Ok(result)) => Ok(result),
            Ok(Err(e)) => Err(MatcherError::Expected(input.pos, e.0)),
            Err(e) => Err(e),
        }
    }
}
