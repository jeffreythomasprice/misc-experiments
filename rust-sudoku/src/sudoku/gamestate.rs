use std::ops::{Index, IndexMut};

use crate::Error;

use super::{Cell, Point};

#[derive(Debug)]
pub enum Status {
    Valid,
    Invalid,
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

    pub fn status(&self) -> Status {
        // will be set to false when we definitely have an incomplete section
        let mut complete = true;

        // each kind of section will early exit when we definitely have a conflict

        // rows
        for row in 0..9 {
            let mut numbers = [false; 9];
            let mut count = 0;
            for column in 0..9 {
                match self.data[row][column] {
                    Cell::PuzzleInput(value) | Cell::Solution(value) => {
                        let index: i8 = value.into();
                        let n = &mut numbers[(index - 1) as usize];
                        if *n {
                            return Status::Invalid;
                        }
                        *n = true;
                        count += 1;
                    }
                    _ => (),
                }
            }
            if count < 9 {
                complete = false;
            }
        }

        // columns
        for column in 0..9 {
            let mut numbers = [false; 9];
            let mut count = 0;
            for row in 0..9 {
                match self.data[row][column] {
                    Cell::PuzzleInput(value) | Cell::Solution(value) => {
                        let index: i8 = value.into();
                        let n = &mut numbers[(index - 1) as usize];
                        if *n {
                            return Status::Invalid;
                        }
                        *n = true;
                        count += 1;
                    }
                    _ => (),
                }
            }
            if count < 9 {
                complete = false;
            }
        }

        // squares
        for square_row in 0..3 {
            for square_column in 0..3 {
                let mut numbers = [false; 9];
                let mut count = 0;
                for row in (square_row * 3)..(square_row * 3 + 3) {
                    for column in (square_column * 3)..(square_column * 3 + 3) {
                        match self.data[row][column] {
                            Cell::PuzzleInput(value) | Cell::Solution(value) => {
                                let index: i8 = value.into();
                                let n = &mut numbers[(index - 1) as usize];
                                if *n {
                                    return Status::Invalid;
                                }
                                *n = true;
                                count += 1;
                            }
                            _ => (),
                        }
                    }
                }
                if count < 9 {
                    complete = false;
                }
            }
        }

        // if every section was complete then it's a solved puzzle
        if complete {
            Status::Solved
        } else {
            Status::Valid
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
