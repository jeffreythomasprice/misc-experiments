use std::ops::{Index, IndexMut};

use crate::Error;

use super::{Cell, Point};

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
