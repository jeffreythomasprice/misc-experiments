use crate::strings::{Match, PosStr};

use super::{Matcher, MatcherError};

pub struct Any2Matcher<M1, M2> {
    m1: M1,
    m2: M2,
}

pub fn any2<'a, T, M1, M2>(m1: M1, m2: M2) -> Any2Matcher<M1, M2>
where
    M1: Matcher<'a, T>,
    M2: Matcher<'a, T>,
{
    Any2Matcher { m1, m2 }
}

impl<'a, T, M1, M2> Matcher<'a, T> for Any2Matcher<M1, M2>
where
    M1: Matcher<'a, T>,
    M2: Matcher<'a, T>,
{
    fn apply(&self, input: PosStr<'a>) -> Result<Match<'a, T>, MatcherError> {
        let e1 = match self.m1.apply(input) {
            Ok(result) => return Ok(result),
            Err(e) => e,
        };
        let e2 = match self.m2.apply(input) {
            Ok(result) => return Ok(result),
            Err(e) => e,
        };
        Err(MatcherError::Expected(
            input.pos,
            format!("one of [{e1:?}, {e2:?}]"),
        ))
    }
}

pub struct Any3Matcher<M1, M2, M3> {
    m1: M1,
    m2: M2,
    m3: M3,
}

pub fn any3<'a, T, M1, M2, M3>(m1: M1, m2: M2, m3: M3) -> Any3Matcher<M1, M2, M3>
where
    M1: Matcher<'a, T>,
    M2: Matcher<'a, T>,
    M3: Matcher<'a, T>,
{
    Any3Matcher { m1, m2, m3 }
}

impl<'a, T, M1, M2, M3> Matcher<'a, T> for Any3Matcher<M1, M2, M3>
where
    M1: Matcher<'a, T>,
    M2: Matcher<'a, T>,
    M3: Matcher<'a, T>,
{
    fn apply(&self, input: PosStr<'a>) -> Result<Match<'a, T>, MatcherError> {
        let e1 = match self.m1.apply(input) {
            Ok(result) => return Ok(result),
            Err(e) => e,
        };
        let e2 = match self.m2.apply(input) {
            Ok(result) => return Ok(result),
            Err(e) => e,
        };
        let e3 = match self.m3.apply(input) {
            Ok(result) => return Ok(result),
            Err(e) => e,
        };
        Err(MatcherError::Expected(
            input.pos,
            format!("one of [{e1:?}, {e2:?}, {e3:?}]"),
        ))
    }
}

pub struct Any4Matcher<M1, M2, M3, M4> {
    m1: M1,
    m2: M2,
    m3: M3,
    m4: M4,
}

pub fn any4<'a, T, M1, M2, M3, M4>(m1: M1, m2: M2, m3: M3, m4: M4) -> Any4Matcher<M1, M2, M3, M4>
where
    M1: Matcher<'a, T>,
    M2: Matcher<'a, T>,
    M3: Matcher<'a, T>,
    M4: Matcher<'a, T>,
{
    Any4Matcher { m1, m2, m3, m4 }
}

impl<'a, T, M1, M2, M3, M4> Matcher<'a, T> for Any4Matcher<M1, M2, M3, M4>
where
    M1: Matcher<'a, T>,
    M2: Matcher<'a, T>,
    M3: Matcher<'a, T>,
    M4: Matcher<'a, T>,
{
    fn apply(&self, input: PosStr<'a>) -> Result<Match<'a, T>, MatcherError> {
        let e1 = match self.m1.apply(input) {
            Ok(result) => return Ok(result),
            Err(e) => e,
        };
        let e2 = match self.m2.apply(input) {
            Ok(result) => return Ok(result),
            Err(e) => e,
        };
        let e3 = match self.m3.apply(input) {
            Ok(result) => return Ok(result),
            Err(e) => e,
        };
        let e4 = match self.m4.apply(input) {
            Ok(result) => return Ok(result),
            Err(e) => e,
        };
        Err(MatcherError::Expected(
            input.pos,
            format!("one of [{e1:?}, {e2:?}, {e3:?}, {e4:?}]"),
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        matchers::{any2, any3, str, Matcher, MatcherError},
        strings::{Match, PosStr, Position},
    };

    #[test]
    pub fn any2_success_1() {
        assert_eq!(
            any2(str("foo"), str("bar")).apply("foo".into()),
            Ok(Match {
                source: PosStr {
                    pos: Position { line: 0, column: 0 },
                    s: "foo",
                },
                matched: PosStr {
                    pos: Position { line: 0, column: 0 },
                    s: "foo",
                },
                remainder: PosStr {
                    pos: Position { line: 0, column: 3 },
                    s: "",
                },
                value: "foo"
            })
        )
    }

    #[test]
    pub fn any2_success_2() {
        assert_eq!(
            any2(str("foo"), str("bar")).apply("bar".into()),
            Ok(Match {
                source: PosStr {
                    pos: Position { line: 0, column: 0 },
                    s: "bar",
                },
                matched: PosStr {
                    pos: Position { line: 0, column: 0 },
                    s: "bar",
                },
                remainder: PosStr {
                    pos: Position { line: 0, column: 3 },
                    s: "",
                },
                value: "bar"
            })
        )
    }

    #[test]
    pub fn any2_failure() {
        assert_eq!(
            any2(str("foo"), str("bar")).apply("baz".into()),
            Err(MatcherError::Expected(
                Position { line: 0, column: 0 },
                "one of [Expected(Position { line: 0, column: 0 }, \"foo\"), Expected(Position { line: 0, column: 0 }, \"bar\")]".to_owned()
            ))
        )
    }

    #[test]
    pub fn any3_success_1() {
        assert_eq!(
            any3(str("a"), str("b"), str("c")).apply("a".into()),
            Ok(Match {
                source: PosStr {
                    pos: Position { line: 0, column: 0 },
                    s: "a",
                },
                matched: PosStr {
                    pos: Position { line: 0, column: 0 },
                    s: "a",
                },
                remainder: PosStr {
                    pos: Position { line: 0, column: 1 },
                    s: "",
                },
                value: "a"
            })
        )
    }

    #[test]
    pub fn any3_success_2() {
        assert_eq!(
            any3(str("a"), str("b"), str("c")).apply("b".into()),
            Ok(Match {
                source: PosStr {
                    pos: Position { line: 0, column: 0 },
                    s: "b",
                },
                matched: PosStr {
                    pos: Position { line: 0, column: 0 },
                    s: "b",
                },
                remainder: PosStr {
                    pos: Position { line: 0, column: 1 },
                    s: "",
                },
                value: "b"
            })
        )
    }

    #[test]
    pub fn any3_success_3() {
        assert_eq!(
            any3(str("a"), str("b"), str("c")).apply("c".into()),
            Ok(Match {
                source: PosStr {
                    pos: Position { line: 0, column: 0 },
                    s: "c",
                },
                matched: PosStr {
                    pos: Position { line: 0, column: 0 },
                    s: "c",
                },
                remainder: PosStr {
                    pos: Position { line: 0, column: 1 },
                    s: "",
                },
                value: "c"
            })
        )
    }

    #[test]
    pub fn any3_failure() {
        assert_eq!(
            any3(str("a"), str("b"), str("c")).apply("foobar".into()),
            Err(MatcherError::Expected(
                Position { line: 0, column: 0 },
                "one of [Expected(Position { line: 0, column: 0 }, \"a\"), Expected(Position { line: 0, column: 0 }, \"b\"), Expected(Position { line: 0, column: 0 }, \"c\")]".to_owned()
            ))
        )
    }

    // TODO tests for any4
}
