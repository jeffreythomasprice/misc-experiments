use std::str::{CharIndices, Chars};

#[derive(Debug, Clone, PartialEq)]
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

pub struct PosStrChars<'a> {
    pos: Position,
    iterator: Chars<'a>,
}

impl<'a> Iterator for PosStrChars<'a> {
    type Item = (Position, char);

    fn next(&mut self) -> Option<Self::Item> {
        self.iterator.next().map(|c| {
            let result = (self.pos.clone(), c);
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
            let result = (self.pos.clone(), i, c);
            self.pos = self.pos.advance(&c);
            result
        })
    }
}

#[derive(Debug)]
pub struct Match<'a, T> {
    pub remainder: PosStr<'a>,
    pub value: T,
}

impl<'a, T> PartialEq for Match<'a, T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.remainder == other.remainder && self.value == other.value
    }
}

#[derive(Debug, Clone, PartialEq)]
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

    pub fn chars<'b>(&self) -> PosStrChars<'b>
    where
        'a: 'b,
    {
        PosStrChars {
            pos: self.pos.clone(),
            iterator: self.s.chars(),
        }
    }

    pub fn char_indices<'b>(&self) -> PosStrCharIndices<'b>
    where
        'a: 'b,
    {
        PosStrCharIndices {
            pos: self.pos.clone(),
            iterator: self.s.char_indices(),
        }
    }

    pub fn take_while_and_remainder<F>(self: PosStr<'a>, mut f: F) -> Match<'a, Option<PosStr<'a>>>
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
                remainder: PosStr {
                    pos,
                    s: &self.s[(i + 1)..],
                },
                value: Some(PosStr {
                    pos: self.pos.clone(),
                    s: &self.s[0..=i],
                }),
            },
            None => Match {
                remainder: self.clone(),
                value: None,
            },
        }
    }

    pub fn skip_while<F>(self: PosStr<'a>, f: F) -> PosStr<'a>
    where
        F: FnMut(&Position, &char) -> bool,
    {
        self.take_while_and_remainder(f).remainder
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

    use crate::strings::{Match, Position};

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
                remainder: PosStr {
                    pos: Position { line: 0, column: 3 },
                    s: ""
                },
                value: Some(PosStr {
                    pos: Position { line: 0, column: 0 },
                    s: "123"
                })
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
                remainder: PosStr {
                    pos: Position { line: 0, column: 3 },
                    s: "abc"
                },
                value: Some(PosStr {
                    pos: Position { line: 0, column: 0 },
                    s: "123"
                })
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
}
