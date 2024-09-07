use std::marker::PhantomData;

use crate::strings::{Match, PosStr};

use super::Matcher;

pub struct MapError;

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
    fn apply(&self, input: PosStr<'a>) -> Option<Match<'a, R>> {
        self.m
            .apply(input)
            .map(|Match { remainder, value }| match (self.f)(value) {
                Ok(value) => Some(Match { remainder, value }),
                Err(_) => None,
            })
            .flatten()
    }
}
