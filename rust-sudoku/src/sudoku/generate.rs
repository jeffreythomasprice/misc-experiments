use std::fmt::Display;

use super::{Cell, GameState, Number, Point};
use crate::Result;
use log::*;
use rand::{seq::SliceRandom, Rng};

struct Possible {
    data: [[Number; 9]; 9],
}

impl Possible {
    pub fn new_random<R>(rng: &mut R) -> Result<Self>
    where
        R: Rng,
    {
        let mut result = Self {
            data: [[
                1.try_into()?,
                2.try_into()?,
                3.try_into()?,
                4.try_into()?,
                5.try_into()?,
                6.try_into()?,
                7.try_into()?,
                8.try_into()?,
                9.try_into()?,
            ]; 9],
        };
        for row in result.data.iter_mut() {
            row.shuffle(rng);
        }
        Ok(result)
    }

    /// how many cells have no conflicts
    pub fn score(&self) -> i32 {
        let mut result = 0;
        for row in 0..9 {
            for column in 0..9 {
                let mut good = true;

                for column2 in 0..9 {
                    if column == column2 {
                        continue;
                    }
                    if self.data[row][column] == self.data[row][column2] {
                        good = false;
                        break;
                    }
                }

                if good {
                    let square_start_row = row / 3 * 3;
                    let square_start_column = column / 3 * 3;
                    for row2 in square_start_row..(square_start_row + 3) {
                        for column2 in square_start_column..(square_start_column + 3) {
                            if row == row2 && column == column2 {
                                continue;
                            }
                            if self.data[row][column] == self.data[row2][column2] {
                                good = false;
                                break;
                            }
                        }
                    }
                }

                if good {
                    result += 1;
                }
            }
        }
        result
    }
}

// TODO no
impl Display for Possible {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in self.data.iter() {
            write!(f, "{row:?}\n")?;
        }
        Ok(())
    }
}

impl GameState {
    pub fn new_random<R>(rng: &mut R) -> Result<Self>
    where
        R: Rng,
    {
        // generate a few possible solutions
        let population = (0..10)
            .map(|_| Possible::new_random(rng))
            .collect::<Result<Vec<_>, _>>()?;

        /*
        TODO finish

        while nobody has a perfect score {
            generate a new random population of the same size
            each new element is a random perturbation of an existing item
            chance of selecting any given original population member is proportional to their score
        }
        */

        todo!()
    }
}
