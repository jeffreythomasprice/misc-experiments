use crate::strings::{Match, PosStr, Position};

#[derive(Debug, Clone, PartialEq)]
pub enum MatcherError {
    Expected(Position, String),
    NotEnoughRemainingInput(Position),
}

pub trait Matcher<'a, T> {
    fn apply(&self, input: PosStr<'a>) -> Result<Match<'a, T>, MatcherError>;

    fn skip(&self, input: PosStr<'a>) -> Result<PosStr<'a>, MatcherError> {
        Ok(match self.apply(input.clone()) {
            Ok(Match {
                pos: _,
                remainder,
                value: _,
            }) => remainder,
            Err(_) => input,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        matchers::str,
        strings::{PosStr, Position},
    };

    use super::Matcher;

    #[test]
    fn found_something_to_skip() {
        assert_eq!(
            str("foo").skip("foobar".into()),
            Ok(PosStr {
                pos: Position { line: 0, column: 3 },
                s: "bar",
            })
        );
    }

    #[test]
    fn matcher_didnt_match_no_skip_possible() {
        assert_eq!(
            str("foo").skip("bar".into()),
            Ok(PosStr {
                pos: Position { line: 0, column: 0 },
                s: "bar",
            })
        );
    }
}
