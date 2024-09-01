use crate::strings::{Match, PosStr, Position};

use super::Matcher;

pub struct TakeWhileMatcher<F>
where
    F: Fn(&Position, &char) -> bool,
{
    f: F,
}

pub fn take_while<F>(f: F) -> TakeWhileMatcher<F>
where
    F: Fn(&Position, &char) -> bool,
{
    TakeWhileMatcher::new(f)
}

impl<F> TakeWhileMatcher<F>
where
    F: Fn(&Position, &char) -> bool,
{
    pub fn new(f: F) -> Self {
        Self { f }
    }
}

impl<'a, F> Matcher<'a, PosStr<'a>> for TakeWhileMatcher<F>
where
    F: Fn(&Position, &char) -> bool,
{
    fn apply(&self, input: PosStr<'a>) -> Option<Match<'a, PosStr<'a>>> {
        Some(input.take_while_and_remainder(&self.f))
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use crate::{
        matchers::{take_while::take_while, Matcher},
        strings::{Match, PosStr, Position},
    };

    #[test]
    fn test() {
        let called_with = Rc::new(RefCell::new(Vec::new()));
        let m = {
            let called_with = called_with.clone();
            take_while(move |pos, c| {
                called_with.borrow_mut().push((pos.clone(), c.clone()));
                ('0'..='9').contains(c)
            })
        };
        let result = m.apply("123abc".into());
        assert_eq!(
            result,
            Some(Match {
                remainder: PosStr {
                    pos: Position { line: 0, column: 3 },
                    s: "abc"
                },
                value: PosStr {
                    pos: Position { line: 0, column: 0 },
                    s: "123"
                }
            })
        );
        assert_eq!(
            called_with.take(),
            vec![
                (Position { line: 0, column: 0 }, '1'),
                (Position { line: 0, column: 1 }, '2'),
                (Position { line: 0, column: 2 }, '3'),
                (Position { line: 0, column: 3 }, 'a')
            ]
        );
    }
}
