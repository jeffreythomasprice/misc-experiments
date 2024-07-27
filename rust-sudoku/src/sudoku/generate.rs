use std::{fmt::Display, mem::swap, ops::Index};

use super::{Cell, GameState, Number, Point};
use crate::Result;
use log::*;
use rand::{seq::SliceRandom, Rng};

#[derive(Clone)]
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

    pub fn new_with_some_random_change<R>(parent: &Possible, rng: &mut R) -> Result<Self>
    where
        R: Rng,
    {
        let mut result = Self {
            data: parent.data.clone(),
        };
        let row = rng.gen_range(0..9);
        let column1 = rng.gen_range(0..9);
        let column2 = rng.gen_range(0..8);
        let column2 = if column1 == column2 {
            column2 + 1
        } else {
            column2
        };
        let temp = result.data[row][column1];
        result.data[row][column1] = result.data[row][column2];
        result.data[row][column2] = temp;
        Ok(result)
    }

    /// how many cells have no conflicts
    pub fn score(&self) -> i32 {
        let mut result = 0;
        for row in 0..9 {
            for column in 0..9 {
                let mut good = true;

                for row2 in 0..9 {
                    if row == row2 {
                        continue;
                    }
                    if self.data[row][column] == self.data[row2][column] {
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

impl Index<Point> for Possible {
    type Output = Number;

    fn index(&self, index: Point) -> &Self::Output {
        &self.data[index.row.0 as usize][index.column.0 as usize]
    }
}

impl Into<GameState> for &Possible {
    fn into(self) -> GameState {
        let mut result = GameState::new();
        Point::all_possible_values().iter().for_each(|p| {
            result[*p] = Cell::PuzzleInput(self[*p]);
        });
        result
    }
}

impl GameState {
    pub fn new_random<R>(rng: &mut R, num_puzzle_inputs: i32) -> Result<Self>
    where
        R: Rng,
    {
        if !(0..=81).contains(&num_puzzle_inputs) {
            Err(format!(
                "number of puzzle inputs outside value range {}",
                num_puzzle_inputs
            ))?;
        }

        // generate a few possible solutions
        let mut population = (0..10)
            .map(|_| Possible::new_random(rng))
            .collect::<Result<Vec<_>, _>>()?;

        // while we haven't found a valid puzzle yet
        let mut generations = 0;
        loop {
            // that the current population and score them all
            // we'll be doing random selection weighted by score, so each element in the scores is the sum of all the scores before it and
            // this one
            // then when we generate the next number we can find the last one we're less than
            let mut total = 0;
            let mut scores = Vec::new();
            let mut best = None;
            for p in population.iter() {
                let score = p.score();
                total += score;
                scores.push((p, score, total));
                // remember the best one for later
                best = match best {
                    Some((best_score, best_ref)) => {
                        if score > best_score {
                            Some((score, p))
                        } else {
                            Some((best_score, best_ref))
                        }
                    }
                    None => Some((score, p)),
                };
            }
            let (best_score, best) = best.unwrap();
            log::trace!("generation {}, best score = {}", generations, best_score);

            if best_score == 81 {
                log::trace!("success on generation {}", generations);
                let mut result: GameState = best.into();
                let mut all_possible_points = Point::all_possible_values();
                all_possible_points.shuffle(rng);
                for p in all_possible_points
                    .iter()
                    .take((81 - num_puzzle_inputs) as usize)
                {
                    result[*p] = Cell::Empty;
                }
                return Ok(result);
            }

            // we're going to iterate backwards so we find the last one we're less than
            scores.reverse();

            let mut new_generation = (0..(population.len() - 1))
                .map(|_| -> Result<Possible> {
                    let target = rng.gen_range(0..total);
                    // TODO binary search
                    let (parent, _, _) = scores
                        .iter()
                        .find(|(_, _, total)| target < *total)
                        .ok_or(format!(
                    "expected to always find a next item when generating new possible solutions"
                ))?;
                    Ok(Possible::new_with_some_random_change(parent, rng)?)
                })
                .collect::<Result<Vec<_>, _>>()?;
            // always keep the current best around unchanged, just in case
            new_generation.push(best.clone());
            swap(&mut new_generation, &mut population);

            generations += 1;
            log::trace!("end of generation {}", generations);
        }
    }
}
