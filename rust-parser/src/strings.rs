use std::{
    cmp::Ordering,
    str::{CharIndices, Chars},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

impl Position {
    pub fn advance(&self, c: &char) -> Position {
        match c {
            '\n' => Position {
                line: self.line + 1,
                column: 0,
            },
            _ => Position {
                line: self.line,
                column: self.column + 1,
            },
        }
    }
}

impl PartialOrd for Position {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Position {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.line, self.column).cmp(&(other.line, other.column))
    }
}

pub struct PosStrChars<'a> {
    pos: Position,
    iterator: Chars<'a>,
}

impl<'a> Iterator for PosStrChars<'a> {
    type Item = (Position, char);

    fn next(&mut self) -> Option<Self::Item> {
        self.iterator.next().map(|c| {
            let result = (self.pos, c);
            self.pos = self.pos.advance(&c);
            result
        })
    }
}

pub struct PosStrCharIndices<'a> {
    pos: Position,
    iterator: CharIndices<'a>,
}

impl<'a> Iterator for PosStrCharIndices<'a> {
    type Item = (Position, usize, char);

    fn next(&mut self) -> Option<Self::Item> {
        self.iterator.next().map(|(i, c)| {
            let result = (self.pos, i, c);
            self.pos = self.pos.advance(&c);
            result
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct Match<'a, T> {
    pub value: T,
    pub remainder: PosStr<'a>,
}

impl<'a, T> Match<'a, T> {
    pub fn map<R, F>(self, f: F) -> Match<'a, R>
    where
        F: FnOnce(T) -> R,
    {
        Match {
            value: f(self.value),
            remainder: self.remainder,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BadPositionError;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PosStr<'a> {
    pub pos: Position,
    pub s: &'a str,
}

impl<'a> PosStr<'a> {
    pub fn new_str(s: &'a str) -> Self {
        Self {
            pos: Position { line: 0, column: 0 },
            s,
        }
    }

    pub fn is_empty(&self) -> bool {
        return self.s.is_empty();
    }

    pub fn len(&self) -> usize {
        return self.s.len();
    }

    pub fn chars<'b>(&self) -> PosStrChars<'b>
    where
        'a: 'b,
    {
        PosStrChars {
            pos: self.pos,
            iterator: self.s.chars(),
        }
    }

    pub fn char_indices<'b>(&self) -> PosStrCharIndices<'b>
    where
        'a: 'b,
    {
        PosStrCharIndices {
            pos: self.pos,
            iterator: self.s.char_indices(),
        }
    }

    pub fn take_single_char(self: PosStr<'a>) -> Option<Match<'a, char>> {
        self.s.chars().next().map(|c| Match {
            value: c,
            remainder: PosStr {
                pos: self.pos.advance(&c),
                s: &self.s[c.len_utf8()..],
            },
        })
    }

    pub fn take_while_and_remainder<F>(self: PosStr<'a>, mut f: F) -> Match<'a, PosStr<'a>>
    where
        F: FnMut(&Position, &char) -> bool,
    {
        let mut last_good = None;
        for (pos, i, c) in self.char_indices() {
            if f(&pos, &c) {
                last_good = Some((pos.advance(&c), i));
            } else {
                break;
            }
        }
        match last_good {
            Some((pos, i)) => Match {
                value: PosStr {
                    pos: self.pos,
                    s: &self.s[0..=i],
                },
                remainder: PosStr {
                    pos,
                    s: &self.s[(i + 1)..],
                },
            },
            None => Match {
                value: PosStr {
                    pos: self.pos,
                    s: "",
                },
                remainder: self,
            },
        }
    }

    pub fn skip_while<F>(self: PosStr<'a>, f: F) -> PosStr<'a>
    where
        F: FnMut(&Position, &char) -> bool,
    {
        self.take_while_and_remainder(f).remainder
    }

    pub fn take_until_position_and_remainder(
        self: PosStr<'a>,
        pos: &Position,
    ) -> Result<Match<'a, PosStr<'a>>, BadPositionError> {
        let target_pos = pos;
        let result = self.take_while_and_remainder(|cur_pos, _| cur_pos < target_pos);
        if result.remainder.pos == *target_pos {
            Ok(result)
        } else {
            Err(BadPositionError)
        }
    }
}

impl<'a> From<&'a str> for PosStr<'a> {
    fn from(value: &'a str) -> Self {
        Self::new_str(value)
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use crate::strings::{BadPositionError, Match, Position};

    use super::PosStr;

    #[test]
    fn chars() {
        let s: PosStr = "abc\n12\n\n3".into();
        let v: Vec<_> = s.chars().collect();
        assert_eq!(
            v,
            vec![
                (Position { line: 0, column: 0 }, 'a'),
                (Position { line: 0, column: 1 }, 'b'),
                (Position { line: 0, column: 2 }, 'c'),
                (Position { line: 0, column: 3 }, '\n'),
                (Position { line: 1, column: 0 }, '1'),
                (Position { line: 1, column: 1 }, '2'),
                (Position { line: 1, column: 2 }, '\n'),
                (Position { line: 2, column: 0 }, '\n'),
                (Position { line: 3, column: 0 }, '3'),
            ]
        );
    }

    #[test]
    fn char_indices() {
        let s: PosStr = "a😀bc".into();
        let v: Vec<_> = s.char_indices().collect();
        assert_eq!(
            v,
            vec![
                (Position { line: 0, column: 0 }, 0, 'a'),
                (Position { line: 0, column: 1 }, 1, '😀'),
                (Position { line: 0, column: 2 }, 5, 'b'),
                (Position { line: 0, column: 3 }, 6, 'c'),
            ]
        );
    }

    #[test]
    fn take_single_char() {
        let s: PosStr = "123".into();
        let m = s.take_single_char();
        assert_eq!(
            m,
            Some(Match {
                value: '1',
                remainder: PosStr {
                    pos: Position { line: 0, column: 1 },
                    s: "23"
                },
            })
        );
        let m = m.unwrap().remainder.take_single_char();
        assert_eq!(
            m,
            Some(Match {
                value: '2',
                remainder: PosStr {
                    pos: Position { line: 0, column: 2 },
                    s: "3"
                },
            })
        );
        let m = m.unwrap().remainder.take_single_char();
        assert_eq!(
            m,
            Some(Match {
                value: '3',
                remainder: PosStr {
                    pos: Position { line: 0, column: 3 },
                    s: ""
                },
            })
        );
        let m = m.unwrap().remainder.take_single_char();
        assert_eq!(m, None);
    }

    #[test]
    fn take_while_and_remainder_no_remainder() {
        let s: PosStr = "123".into();
        let called_with = Rc::new(RefCell::new(Vec::new()));
        let result = {
            let called_with = called_with.clone();
            s.take_while_and_remainder(move |pos, c| {
                called_with.borrow_mut().push((pos.clone(), c.clone()));
                ('0'..='9').contains(c)
            })
        };
        assert_eq!(
            result,
            Match {
                value: PosStr {
                    pos: Position { line: 0, column: 0 },
                    s: "123"
                },
                remainder: PosStr {
                    pos: Position { line: 0, column: 3 },
                    s: ""
                },
            }
        );
        assert_eq!(
            called_with.take(),
            vec![
                (Position { line: 0, column: 0 }, '1'),
                (Position { line: 0, column: 1 }, '2'),
                (Position { line: 0, column: 2 }, '3')
            ]
        );
    }

    #[test]
    fn take_while_and_remainder_has_remainder() {
        let s: PosStr = "123abc".into();
        let called_with = Rc::new(RefCell::new(Vec::new()));
        let result = {
            let called_with = called_with.clone();
            s.take_while_and_remainder(move |pos, c| {
                called_with.borrow_mut().push((pos.clone(), c.clone()));
                ('0'..='9').contains(c)
            })
        };
        assert_eq!(
            result,
            Match {
                value: PosStr {
                    pos: Position { line: 0, column: 0 },
                    s: "123"
                },
                remainder: PosStr {
                    pos: Position { line: 0, column: 3 },
                    s: "abc"
                },
            }
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

    #[test]
    fn skip_while() {
        let s: PosStr = "123abc".into();
        let called_with = Rc::new(RefCell::new(Vec::new()));
        let result = {
            let called_with = called_with.clone();
            s.skip_while(move |pos, c| {
                called_with.borrow_mut().push((pos.clone(), c.clone()));
                ('0'..='9').contains(c)
            })
        };
        assert_eq!(
            result,
            PosStr {
                pos: Position { line: 0, column: 3 },
                s: "abc"
            }
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

    #[test]
    fn take_to_position() {
        let s: PosStr = "123\n456\n789".into();
        assert_eq!(
            s.take_until_position_and_remainder(&Position { line: 0, column: 1 }),
            Ok(Match {
                value: PosStr {
                    pos: Position { line: 0, column: 0 },
                    s: "1"
                },
                remainder: PosStr {
                    pos: Position { line: 0, column: 1 },
                    s: "23\n456\n789"
                },
            })
        );
        assert_eq!(
            s.take_until_position_and_remainder(&Position { line: 0, column: 3 }),
            Ok(Match {
                value: PosStr {
                    pos: Position { line: 0, column: 0 },
                    s: "123"
                },
                remainder: PosStr {
                    pos: Position { line: 0, column: 3 },
                    s: "\n456\n789"
                },
            })
        );
        assert_eq!(
            s.take_until_position_and_remainder(&Position { line: 1, column: 0 }),
            Ok(Match {
                value: PosStr {
                    pos: Position { line: 0, column: 0 },
                    s: "123\n"
                },
                remainder: PosStr {
                    pos: Position { line: 1, column: 0 },
                    s: "456\n789"
                },
            })
        );
        assert_eq!(
            s.take_until_position_and_remainder(&Position { line: 2, column: 3 }),
            Ok(Match {
                value: PosStr {
                    pos: Position { line: 0, column: 0 },
                    s: "123\n456\n789"
                },
                remainder: PosStr {
                    pos: Position { line: 2, column: 3 },
                    s: ""
                },
            })
        );
        assert_eq!(
            s.take_until_position_and_remainder(&Position { line: 0, column: 4 }),
            Err(BadPositionError)
        );
        assert_eq!(
            s.take_until_position_and_remainder(&Position { line: 3, column: 0 }),
            Err(BadPositionError)
        );
    }
}
