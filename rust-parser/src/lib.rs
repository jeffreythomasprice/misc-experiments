pub mod matchers;
pub mod strings;

// TODO replace this with a proper test

#[cfg(test)]
pub mod test {
    use crate::{
        matchers::{match2, repeat, take_while, MapError, Mappable, Matcher, MatcherError},
        strings::{Match, PosStr, Position},
    };

    fn match_u32<'a>() -> impl Matcher<'a, u32> {
        take_while(|_pos, c| ('0'..='9').contains(c)).map(|value| {
            value
                .s
                .parse::<u32>()
                .map_err(|e| MapError(format!("failed to parse as u32: {e:?}")))
        })
    }

    fn tokenize<'a, M, T>(m: M) -> impl Matcher<'a, T>
    where
        M: Matcher<'a, T>,
    {
        match2(m, take_while(|_pos, c| -> bool { c.is_whitespace() })).map(|(a, b)| Ok(a))
    }

    fn match_u32_list<'a>() -> impl Matcher<'a, Vec<u32>> {
        repeat(tokenize(match_u32()), ..)
    }

    #[test]
    pub fn u32_success() {
        assert_eq!(
            match_u32().apply("123".into()),
            Ok(Match {
                remainder: PosStr {
                    pos: Position { line: 0, column: 3 },
                    s: ""
                },
                value: 123
            })
        );
    }

    #[test]
    pub fn u32_failure() {
        assert_eq!(
            match_u32().apply("foobar".into()),
            Err(MatcherError::Expected(
                Position { line: 0, column: 0 },
                "failed to parse as u32: ParseIntError { kind: Empty }".to_owned()
            ))
        );
    }

    #[test]
    pub fn u32_failure_too_big() {
        assert_eq!(
            match_u32().apply("4294967296".into()),
            Err(MatcherError::Expected(
                Position { line: 0, column: 0 },
                "failed to parse as u32: ParseIntError { kind: PosOverflow }".to_owned()
            ))
        );
    }

    #[test]
    pub fn u32_list_success() {
        assert_eq!(
            match_u32_list().apply("123 456    789   foobar".into()),
            Ok(Match {
                remainder: PosStr {
                    pos: Position {
                        line: 0,
                        column: 17
                    },
                    s: "foobar"
                },
                value: vec![123, 456, 789],
            })
        );
    }
}
