use crate::strings::{Match, PosStr};

use super::Matcher;

pub struct StrMatcher<'a> {
    s: &'a str,
}

pub fn str(s: &str) -> StrMatcher {
    StrMatcher::new(s)
}

impl<'a> StrMatcher<'a> {
    pub fn new(s: &'a str) -> Self {
        Self { s }
    }
}

impl<'a> Matcher<'a, &'a str> for StrMatcher<'a> {
    fn apply(&self, input: PosStr<'a>) -> Option<Match<'a, &'a str>> {
        if input.s.len() < self.s.len() {
            return None;
        }
        let mut pos = input.pos.clone();
        for (input, check) in input.s.chars().zip(self.s.chars()) {
            if input != check {
                return None;
            }
            pos = pos.advance(&check);
        }
        Some(Match {
            remainder: PosStr {
                pos,
                s: &input.s[self.s.len()..],
            },
            value: &input.s[0..self.s.len()],
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        matchers::Matcher,
        strings::{Match, PosStr, Position},
    };

    #[test]
    fn some() {
        assert_eq!(
            super::str("foo").apply("foobar".into()),
            Some(Match {
                remainder: PosStr {
                    pos: Position { line: 0, column: 3 },
                    s: "bar"
                },
                value: "foo"
            })
        );
    }

    #[test]
    fn none() {
        assert_eq!(super::str("foo").apply("barfoo".into()), None);
    }

    #[test]
    fn none_input_is_too_small() {
        assert_eq!(super::str("foo").apply("f".into()), None,);
    }
}
