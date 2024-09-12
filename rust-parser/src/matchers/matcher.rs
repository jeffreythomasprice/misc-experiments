use std::{cell::RefCell, rc::Rc, sync::Arc};

use crate::strings::{Match, PosStr, Position};

#[derive(Debug, Clone, PartialEq)]
pub enum MatcherError {
    Expected(Position, String),
    NotEnoughRemainingInput(Position),
}

pub trait Matcher<'a, T> {
    fn apply(&self, input: PosStr<'a>) -> Result<Match<'a, T>, MatcherError>;

    fn skip(&self, input: PosStr<'a>) -> Result<PosStr<'a>, MatcherError> {
        Ok(match self.apply(input) {
            Ok(Match {
                source: _,
                matched: _,
                remainder,
                value: _,
            }) => remainder,
            Err(_) => input,
        })
    }
}

impl<'a, T, M> Matcher<'a, T> for Box<M>
where
    M: Matcher<'a, T>,
{
    fn apply(&self, input: PosStr<'a>) -> Result<Match<'a, T>, MatcherError> {
        self.as_ref().apply(input)
    }
}

impl<'a, T, M> Matcher<'a, T> for Rc<M>
where
    M: Matcher<'a, T>,
{
    fn apply(&self, input: PosStr<'a>) -> Result<Match<'a, T>, MatcherError> {
        self.as_ref().apply(input)
    }
}

impl<'a, T, M> Matcher<'a, T> for Arc<M>
where
    M: Matcher<'a, T>,
{
    fn apply(&self, input: PosStr<'a>) -> Result<Match<'a, T>, MatcherError> {
        self.as_ref().apply(input)
    }
}

impl<'a, T, M> Matcher<'a, T> for RefCell<M>
where
    M: Matcher<'a, T>,
{
    fn apply(&self, input: PosStr<'a>) -> Result<Match<'a, T>, MatcherError> {
        self.borrow().apply(input)
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
