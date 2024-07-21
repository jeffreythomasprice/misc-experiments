use std::fmt::{Debug, Display};

use web_sys::js_sys::Reflect::is_extensible;

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

impl Coordinate {
    pub fn all_possible_values() -> [Coordinate; 9] {
        core::array::from_fn(|i| Coordinate::try_from(i as i8).unwrap())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point {
    pub row: Coordinate,
    pub column: Coordinate,
}

impl Point {
    pub fn all_possible_values() -> Vec<Point> {
        Coordinate::all_possible_values()
            .map(|row| Coordinate::all_possible_values().map(|column| Point { row, column }))
            .as_flattened()
            .to_vec()
    }
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
}
