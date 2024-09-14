use std::{fmt::Debug, ops::RangeBounds};

use crate::strings::{BadPositionError, Match, PosStr};

#[derive(Debug, Clone, PartialEq)]
pub enum MatchError {
    Expected {
        expected: String,
        got: String,
    },
    Parse {
        expected: String,
        got: String,
        error: String,
    },
    EndOfInput {
        expected: String,
    },
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
        value: (r1.value, r2.value),
        remainder: r2.remainder,
    })
}

pub fn seq3<T1, M1, T2, M2, T3, M3>(
    input: PosStr,
    m1: M1,
    m2: M2,
    m3: M3,
) -> Result<Match<(T1, T2, T3)>, MatchError>
where
    M1: Fn(PosStr) -> Result<Match<T1>, MatchError>,
    M2: Fn(PosStr) -> Result<Match<T2>, MatchError>,
    M3: Fn(PosStr) -> Result<Match<T3>, MatchError>,
{
    let r1 = m1(input)?;
    let r2 = m2(r1.remainder)?;
    let r3 = m3(r2.remainder)?;
    Ok(Match {
        value: (r1.value, r2.value, r3.value),
        remainder: r3.remainder,
    })
}

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

/// Matches a binary list of the form
/// ```text
/// T1 T2 T1 T2 T1 ...
/// ```
/// Must start and end with T1, and every pair of T1 is separated by a T2.
///
/// Fails if a T2 is found but not followed by a T1.
///
/// Fails if the number of matched T1 doesn't satisfy the range constraint.
pub fn binary_list<T1, M1, T2, M2, R>(
    input: PosStr,
    m1: M1,
    m2: M2,
    r: R,
) -> Result<Match<Option<(T1, Vec<(T2, T1)>)>>, MatchError>
where
    M1: Fn(PosStr) -> Result<Match<T1>, MatchError>,
    M2: Fn(PosStr) -> Result<Match<T2>, MatchError>,
    R: RangeBounds<usize> + Debug,
{
    // match the first element
    let Match {
        value: first,
        remainder,
    } = match m1(input) {
        Ok(x) => x,
        Err(e) => {
            // we failed to match even one, but that could still be a succcess if the range constraint allows empty lists
            if r.contains(&0) {
                return Ok(Match {
                    value: None,
                    remainder: input,
                });
            }
            // nope, fail
            Err(e)?
        }
    };

    let mut remainder = remainder;
    let mut results = Vec::new();

    loop {
        // if we have a good set of data now, but adding one more would put us outside the range constraint, then we're done
        // len+1 becacuse the first element won't be in the results vec
        let current_count = results.len() + 1;
        if r.contains(&current_count) && !r.contains(&(current_count + 1)) {
            break;
        }

        // match the separator
        let Match {
            value: value2,
            remainder: partial_remainder,
        } = match m2(remainder) {
            Ok(x) => x,
            // no error, just means we didn't see the next separator, so we're done
            Err(_) => break,
        };
        // match the next element after the separator
        let Match {
            value: value1,
            remainder: partial_remainder,
        } = m1(partial_remainder)?;
        results.push((value2, value1));
        remainder = partial_remainder;
    }

    // we have out list, no parse errors, so we've either succeeded or failed based on whether the range constraint is satisfied
    let current_count = results.len() + 1;
    if r.contains(&current_count) {
        Ok(Match {
            value: Some((first, results)),
            remainder,
        })
    } else {
        Err(MatchError::Expected {
            expected: format!("{r:?} results"),
            got: format!("{current_count}"),
        })
    }
}

#[cfg(test)]
mod tests {
    // TODO tests
}
