use std::{fmt::Debug, ops::RangeBounds};

use crate::strings::{BadPositionError, Match, PosStr};

#[derive(Debug, Clone, PartialEq)]
pub enum MatchError {
    Expected { expected: String, got: String },
    EndOfInput { expected: String },
    BadPositionError,
}

impl From<BadPositionError> for MatchError {
    fn from(_value: BadPositionError) -> Self {
        Self::BadPositionError
    }
}

pub fn any_char(input: PosStr) -> Result<Match<char>, MatchError> {
    match input.take_single_char() {
        Some(result) => Ok(result),
        None => Err(MatchError::EndOfInput {
            expected: "any character".to_owned(),
        }),
    }
}

pub fn specific_char(input: PosStr, c: char) -> Result<Match<char>, MatchError> {
    match input.take_single_char() {
        Some(result) if result.value == c => Ok(result),
        Some(result) => Err(MatchError::Expected {
            expected: format!("{c}"),
            got: format!("{}", result.value),
        }),
        None => Err(MatchError::EndOfInput {
            expected: format!("{c}"),
        }),
    }
}

pub fn char_range<R>(input: PosStr, r: R) -> Result<Match<char>, MatchError>
where
    R: RangeBounds<char> + Debug,
{
    match input.take_single_char() {
        Some(result) if r.contains(&result.value) => Ok(result),
        Some(result) => Err(MatchError::Expected {
            expected: format!("{r:?}"),
            got: format!("{}", result.value),
        }),
        None => Err(MatchError::EndOfInput {
            expected: format!("{r:?}"),
        }),
    }
}

pub fn seq2<T1, M1, T2, M2>(input: PosStr, m1: M1, m2: M2) -> Result<Match<(T1, T2)>, MatchError>
where
    M1: Fn(PosStr) -> Result<Match<T1>, MatchError>,
    M2: Fn(PosStr) -> Result<Match<T2>, MatchError>,
{
    let r1 = m1(input)?;
    let r2 = m2(r1.remainder)?;
    Ok(Match {
        source: input,
        matched: input
            .take_until_position_and_remainder(&r2.remainder.pos)?
            .matched,
        remainder: r2.remainder,
        value: (r1.value, r2.value),
    })
}

// TODO seq3, ...

pub fn any2<T, M1, M2>(input: PosStr, m1: M1, m2: M2) -> Result<Match<T>, (MatchError, MatchError)>
where
    M1: Fn(PosStr) -> Result<Match<T>, MatchError>,
    M2: Fn(PosStr) -> Result<Match<T>, MatchError>,
{
    let r1 = m1(input);
    let r2 = m2(input);
    match (r1, r2) {
        (Ok(result), _) | (_, Ok(result)) => Ok(result),
        (Err(e1), Err(e2)) => Err((e1, e2)),
    }
}

pub fn any3<T, M1, M2, M3>(
    input: PosStr,
    m1: M1,
    m2: M2,
    m3: M3,
) -> Result<Match<T>, (MatchError, MatchError, MatchError)>
where
    M1: Fn(PosStr) -> Result<Match<T>, MatchError>,
    M2: Fn(PosStr) -> Result<Match<T>, MatchError>,
    M3: Fn(PosStr) -> Result<Match<T>, MatchError>,
{
    let r1 = m1(input);
    let r2 = m2(input);
    let r3 = m3(input);
    match (r1, r2, r3) {
        (Ok(result), _, _) | (_, Ok(result), _) | (_, _, Ok(result)) => Ok(result),
        (Err(e1), Err(e2), Err(e3)) => Err((e1, e2, e3)),
    }
}
