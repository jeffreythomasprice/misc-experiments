use std::ops::{Index, IndexMut};

use crate::{
    sudoku::{ColumnIterator, SquareIterator},
    Error,
};

use super::{AllPointsIterator, Cell, Point, RowIterator};

#[derive(Debug)]
pub enum CellStatus {
    Conflict,
    NoConflict,
}

#[derive(Debug)]
pub enum Status {
    NoConflict,
    Conflict,
    Solved,
}

#[derive(Clone)]
pub struct GameState {
    // indices are [row][column]
    data: [[Cell; 9]; 9],
}

impl GameState {
    pub fn new() -> Self {
        Self {
            data: [[Cell::Empty; 9]; 9],
        }
    }

    pub fn status_at(&self, p: &Point) -> (&Cell, CellStatus) {
        let p_cell = &self[*p];
        match p_cell.number() {
            Some(p_num) => {
                for q in RowIterator::new_containing_point(p) {
                    if q == *p {
                        continue;
                    }
                    if let Some(q_num) = self[q].number() {
                        if p_num == q_num {
                            return (p_cell, CellStatus::Conflict);
                        }
                    }
                }
                for q in ColumnIterator::new_containing_point(p) {
                    if q == *p {
                        continue;
                    }
                    if let Some(q_num) = self[q].number() {
                        if p_num == q_num {
                            return (p_cell, CellStatus::Conflict);
                        }
                    }
                }
                for q in SquareIterator::new_containing_point(p) {
                    if q == *p {
                        continue;
                    }
                    if let Some(q_num) = self[q].number() {
                        if p_num == q_num {
                            return (p_cell, CellStatus::Conflict);
                        }
                    }
                }
                (p_cell, CellStatus::NoConflict)
            }
            None => (p_cell, CellStatus::NoConflict),
        }
    }

    pub fn status(&self) -> Status {
        let mut empty_or_pencil_mark_count = 0;
        for p in AllPointsIterator::new() {
            match self.status_at(&p) {
                (_, CellStatus::Conflict) => return Status::Conflict,
                (Cell::Empty, _) | (Cell::PencilMark(_), _) => empty_or_pencil_mark_count += 1,
                _ => (),
            };
        }
        if empty_or_pencil_mark_count == 0 {
            Status::Solved
        } else {
            Status::NoConflict
        }
    }
}

impl Index<Point> for GameState {
    type Output = Cell;

    fn index(&self, index: Point) -> &Self::Output {
        &self.data[index.row.0 as usize][index.column.0 as usize]
    }
}

impl IndexMut<Point> for GameState {
    fn index_mut(&mut self, index: Point) -> &mut Self::Output {
        &mut self.data[index.row.0 as usize][index.column.0 as usize]
    }
}
