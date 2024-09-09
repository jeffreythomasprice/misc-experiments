use crate::strings::{Match, PosStr};

use super::{Matcher, MatcherError};

pub struct Match2Matcher<M1, M2> {
    m1: M1,
    m2: M2,
}

pub fn match2<'a, T1, M1, T2, M2>(m1: M1, m2: M2) -> Match2Matcher<M1, M2>
where
    M1: Matcher<'a, T1>,
    M2: Matcher<'a, T2>,
{
    Match2Matcher { m1, m2 }
}

impl<'a, T1, M1, T2, M2> Matcher<'a, (T1, T2)> for Match2Matcher<M1, M2>
where
    M1: Matcher<'a, T1>,
    M2: Matcher<'a, T2>,
{
    fn apply(&self, input: PosStr<'a>) -> Result<Match<'a, (T1, T2)>, MatcherError> {
        let original_input = input;
        let (input, s1, result1) = match self.m1.apply(input) {
            Ok(Match {
                source: _,
                matched,
                remainder,
                value,
            }) => (remainder, matched, value),
            Err(e) => return Err(e),
        };
        let (input, s2, result2) = match self.m2.apply(input) {
            Ok(Match {
                source: _,
                matched,
                remainder,
                value,
            }) => (remainder, matched, value),
            Err(e) => return Err(e),
        };
        Ok(Match {
            source: original_input,
            matched: PosStr {
                pos: original_input.pos,
                s: &original_input.s[0..(s1.s.len() + s2.s.len())],
            },
            remainder: input,
            value: (result1, result2),
        })
    }
}

pub struct Match3Matcher<M1, M2, M3> {
    m1: M1,
    m2: M2,
    m3: M3,
}

pub fn match3<'a, T1, M1, T2, M2, T3, M3>(m1: M1, m2: M2, m3: M3) -> Match3Matcher<M1, M2, M3>
where
    M1: Matcher<'a, T1>,
    M2: Matcher<'a, T2>,
    M3: Matcher<'a, T3>,
{
    Match3Matcher { m1, m2, m3 }
}

impl<'a, T1, M1, T2, M2, T3, M3> Matcher<'a, (T1, T2, T3)> for Match3Matcher<M1, M2, M3>
where
    M1: Matcher<'a, T1>,
    M2: Matcher<'a, T2>,
    M3: Matcher<'a, T3>,
{
    fn apply(&self, input: PosStr<'a>) -> Result<Match<'a, (T1, T2, T3)>, MatcherError> {
        let original_input = input;
        let (input, s1, result1) = match self.m1.apply(input) {
            Ok(Match {
                source: _,
                matched,
                remainder,
                value,
            }) => (remainder, matched, value),
            Err(e) => return Err(e),
        };
        let (input, s2, result2) = match self.m2.apply(input) {
            Ok(Match {
                source: _,
                matched,
                remainder,
                value,
            }) => (remainder, matched, value),
            Err(e) => return Err(e),
        };
        let (input, s3, result3) = match self.m3.apply(input) {
            Ok(Match {
                source: _,
                matched,
                remainder,
                value,
            }) => (remainder, matched, value),
            Err(e) => return Err(e),
        };
        Ok(Match {
            source: original_input,
            matched: PosStr {
                pos: original_input.pos,
                s: &original_input.s[0..(s1.s.len() + s2.s.len() + s3.s.len())],
            },
            remainder: input,
            value: (result1, result2, result3),
        })
    }
}

// TODO match4

#[cfg(test)]
mod tests {
    use crate::{
        matchers::{match3, str, Matcher, MatcherError},
        strings::{Match, PosStr, Position},
    };

    use super::match2;

    #[test]
    fn match2_success() {
        assert_eq!(
            match2(str("foo"), str("bar")).apply("foobar".into()),
            Ok(Match {
                source: PosStr {
                    pos: Position { line: 0, column: 0 },
                    s: "foobar",
                },
                matched: PosStr {
                    pos: Position { line: 0, column: 0 },
                    s: "foobar",
                },
                remainder: PosStr {
                    pos: Position { line: 0, column: 6 },
                    s: "",
                },
                value: ("foo", "bar")
            })
        );
    }

    #[test]
    fn match2_failure_1() {
        assert_eq!(
            match2(str("foo"), str("bar")).apply("fobar".into()),
            Err(MatcherError::Expected(
                Position { line: 0, column: 0 },
                "foo".to_owned()
            ))
        );
    }

    #[test]
    fn match2_failure_2() {
        assert_eq!(
            match2(str("foo"), str("bar")).apply("foobr".into()),
            Err(MatcherError::NotEnoughRemainingInput(Position {
                line: 0,
                column: 3
            },))
        );
    }

    #[test]
    fn match3_success() {
        assert_eq!(
            match3(str("foo"), str("bar"), str("baz")).apply("foobarbaz".into()),
            Ok(Match {
                source: PosStr {
                    pos: Position { line: 0, column: 0 },
                    s: "foobarbaz",
                },
                matched: PosStr {
                    pos: Position { line: 0, column: 0 },
                    s: "foobarbaz",
                },
                remainder: PosStr {
                    pos: Position { line: 0, column: 9 },
                    s: "",
                },
                value: ("foo", "bar", "baz")
            })
        );
    }

    #[test]
    fn match3_failure_1() {
        assert_eq!(
            match3(str("foo"), str("bar"), str("baz")).apply("fobarbaz".into()),
            Err(MatcherError::Expected(
                Position { line: 0, column: 0 },
                "foo".to_owned()
            ))
        );
    }

    #[test]
    fn match3_failure_2() {
        assert_eq!(
            match3(str("foo"), str("bar"), str("baz")).apply("foobrbaz".into()),
            Err(MatcherError::Expected(
                Position { line: 0, column: 3 },
                "bar".to_owned()
            ))
        );
    }

    #[test]
    fn match3_failure_3() {
        assert_eq!(
            match3(str("foo"), str("bar"), str("baz")).apply("foobarbz".into()),
            Err(MatcherError::NotEnoughRemainingInput(Position {
                line: 0,
                column: 6
            },))
        );
    }

    // TODO tests for match4
}
