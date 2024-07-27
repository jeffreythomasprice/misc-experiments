use std::fmt::{Debug, Display};

use crate::Error;

/*
A value from 1 to 9, inclusive.
*/
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Number(i8);

impl Number {
    pub fn all() -> [Number; 9] {
        return [
            Number(1),
            Number(2),
            Number(3),
            Number(4),
            Number(5),
            Number(6),
            Number(7),
            Number(8),
            Number(9),
        ];
    }
}

impl Display for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<i8> for Number {
    type Error = Error;

    fn try_from(value: i8) -> Result<Self, Self::Error> {
        if (1..=9).contains(&value) {
            Ok(Self(value))
        } else {
            Err(format!("{value}"))?
        }
    }
}

impl Into<i8> for Number {
    fn into(self) -> i8 {
        self.0
    }
}

/*
A value from 0 to 8, inclusive.
*/
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Coordinate(pub i8);

impl TryFrom<i8> for Coordinate {
    type Error = Error;

    fn try_from(value: i8) -> Result<Self, Self::Error> {
        if (0..=8).contains(&value) {
            Ok(Self(value))
        } else {
            Err(format!("{value}"))?
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point {
    pub row: Coordinate,
    pub column: Coordinate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PencilMarkMask(u16);

impl PencilMarkMask {
    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }

    pub fn is_set(&self, number: Number) -> bool {
        (self.0 & PencilMarkMask::mask_from_number(number)) != 0
    }

    pub fn toggle(&self, number: Number) -> Self {
        if self.is_set(number) {
            Self(self.0 & !PencilMarkMask::mask_from_number(number))
        } else {
            Self(self.0 | PencilMarkMask::mask_from_number(number))
        }
    }

    fn mask_from_number(number: Number) -> u16 {
        1 << (number.0 as u16)
    }
}

impl From<Number> for PencilMarkMask {
    fn from(value: Number) -> Self {
        Self(PencilMarkMask::mask_from_number(value))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cell {
    Empty,
    PuzzleInput(Number),
    Solution(Number),
    PencilMark(PencilMarkMask),
}

impl Cell {
    pub fn from_input(existing: Cell, input: Number, is_pencil_marking: bool) -> Self {
        match (existing, is_pencil_marking) {
            (Cell::PuzzleInput(existing), _) => Cell::PuzzleInput(existing),
            (Cell::Solution(existing), false) if existing == input => Cell::Empty,
            (Cell::Empty, false) | (Cell::Solution(_), false) | (Cell::PencilMark(_), false) => {
                Cell::Solution(input)
            }
            (Cell::Empty, true) | (Cell::Solution(_), true) => Cell::PencilMark(input.into()),
            (Cell::PencilMark(existing), true) => {
                let result = Cell::PencilMark(existing.toggle(input));
                if let Cell::PencilMark(result) = result {
                    if result.is_empty() {
                        Cell::Empty
                    } else {
                        Cell::PencilMark(result)
                    }
                } else {
                    result
                }
            }
        }
    }

    pub fn number(&self) -> Option<Number> {
        match self {
            Cell::Empty | Cell::PencilMark(_) => None,
            Cell::PuzzleInput(result) | Cell::Solution(result) => Some(*result),
        }
    }
}

pub struct RowIterator {
    row: Coordinate,
    cur: Option<Coordinate>,
}

impl RowIterator {
    pub fn new(row: Coordinate) -> Self {
        Self {
            row,
            cur: Some(Coordinate(0)),
        }
    }

    pub fn new_containing_point(p: &Point) -> Self {
        Self::new(p.row)
    }
}

impl Iterator for RowIterator {
    type Item = Point;

    fn next(&mut self) -> Option<Self::Item> {
        let (result, next) = match self.cur {
            Some(Coordinate(8)) => (
                Some(Point {
                    row: self.row,
                    column: Coordinate(8),
                }),
                None,
            ),
            Some(cur) => (
                Some(Point {
                    row: self.row,
                    column: cur,
                }),
                Some(Coordinate(cur.0 + 1)),
            ),
            None => (None, None),
        };
        self.cur = next;
        result
    }
}

pub struct ColumnIterator {
    column: Coordinate,
    cur: Option<Coordinate>,
}

impl ColumnIterator {
    pub fn new(column: Coordinate) -> Self {
        Self {
            column,
            cur: Some(Coordinate(0)),
        }
    }

    pub fn new_containing_point(p: &Point) -> Self {
        Self::new(p.column)
    }
}

impl Iterator for ColumnIterator {
    type Item = Point;

    fn next(&mut self) -> Option<Self::Item> {
        let (result, next) = match self.cur {
            Some(Coordinate(8)) => (
                Some(Point {
                    column: self.column,
                    row: Coordinate(8),
                }),
                None,
            ),
            Some(cur) => (
                Some(Point {
                    column: self.column,
                    row: cur,
                }),
                Some(Coordinate(cur.0 + 1)),
            ),
            None => (None, None),
        };
        self.cur = next;
        result
    }
}

pub struct SquareIterator<'a> {
    start: Point,
    cur: std::slice::Iter<'a, Point>,
}

impl<'a> SquareIterator<'a> {
    pub fn new_containing_point(p: &Point) -> Self {
        static SQUARE_OFFSETS: [Point; 9] = [
            Point {
                row: Coordinate(0),
                column: Coordinate(0),
            },
            Point {
                row: Coordinate(1),
                column: Coordinate(0),
            },
            Point {
                row: Coordinate(2),
                column: Coordinate(0),
            },
            Point {
                row: Coordinate(0),
                column: Coordinate(1),
            },
            Point {
                row: Coordinate(1),
                column: Coordinate(1),
            },
            Point {
                row: Coordinate(2),
                column: Coordinate(1),
            },
            Point {
                row: Coordinate(0),
                column: Coordinate(2),
            },
            Point {
                row: Coordinate(1),
                column: Coordinate(2),
            },
            Point {
                row: Coordinate(2),
                column: Coordinate(2),
            },
        ];
        Self {
            start: Point {
                row: Coordinate(p.row.0 / 3 * 3),
                column: Coordinate(p.column.0 / 3 * 3),
            },
            cur: SQUARE_OFFSETS.iter(),
        }
    }
}

impl<'a> Iterator for SquareIterator<'a> {
    type Item = Point;

    fn next(&mut self) -> Option<Self::Item> {
        self.cur.next().map(|offset| Point {
            row: Coordinate(self.start.row.0 + offset.row.0),
            column: Coordinate(self.start.column.0 + offset.column.0),
        })
    }
}

pub struct AllPointsIterator {
    cur: Option<Point>,
}

impl AllPointsIterator {
    pub fn new() -> Self {
        Self {
            cur: Some(Point {
                row: Coordinate(0),
                column: Coordinate(0),
            }),
        }
    }
}

impl Iterator for AllPointsIterator {
    type Item = Point;

    fn next(&mut self) -> Option<Self::Item> {
        match self.cur {
            Some(Point {
                row: Coordinate(8),
                column: Coordinate(8),
            }) => {
                self.cur = None;
                Some(Point {
                    row: Coordinate(8),
                    column: Coordinate(8),
                })
            }
            Some(Point {
                row,
                column: Coordinate(8),
            }) => {
                self.cur = Some(Point {
                    row: Coordinate(row.0 + 1),
                    column: Coordinate(0),
                });
                Some(Point {
                    row,
                    column: Coordinate(8),
                })
            }
            Some(Point { row, column }) => {
                self.cur = Some(Point {
                    row,
                    column: Coordinate(column.0 + 1),
                });
                Some(Point { row, column })
            }
            None => None,
        }
    }
}
