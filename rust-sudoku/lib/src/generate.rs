use std::mem::swap;

use super::{AllPointsIterator, Cell, CellStatus, Coordinate, GameState, Point};
use crate::Result;
use chrono::{prelude::*, TimeDelta};
use rand::{seq::SliceRandom, Rng};

#[derive(Clone)]
struct Possible {
    state: GameState,
}

impl Possible {
    pub fn new_random<R>(rng: &mut R) -> Result<Self>
    where
        R: Rng,
    {
        let mut state = GameState::new();
        for row in 0..9 {
            let mut row_data = [
                Cell::PuzzleInput(1.try_into()?),
                Cell::PuzzleInput(2.try_into()?),
                Cell::PuzzleInput(3.try_into()?),
                Cell::PuzzleInput(4.try_into()?),
                Cell::PuzzleInput(5.try_into()?),
                Cell::PuzzleInput(6.try_into()?),
                Cell::PuzzleInput(7.try_into()?),
                Cell::PuzzleInput(8.try_into()?),
                Cell::PuzzleInput(9.try_into()?),
            ];
            row_data.shuffle(rng);
            for column in 0..9 {
                state[Point {
                    row: row.try_into()?,
                    column: column.try_into()?,
                }] = row_data[column as usize];
            }
        }
        Ok(Self { state })
    }

    pub fn new_with_some_random_change<R>(parent: &Possible, rng: &mut R) -> Result<Self>
    where
        R: Rng,
    {
        let mut result = Self {
            state: parent.state.clone(),
        };
        let row: Coordinate = rng.gen_range(0..9).try_into()?;
        let column1: Coordinate = rng.gen_range(0..9).try_into()?;
        let column2: Coordinate = rng.gen_range(0..8).try_into()?;
        let column2 = if column1 == column2 {
            (column2.0 + 1).try_into()?
        } else {
            column2
        };
        let p1 = Point {
            row,
            column: column1,
        };
        let p2 = Point {
            row,
            column: column2,
        };
        let temp = result.state[p1];
        result.state[p1] = result.state[p2];
        result.state[p2] = temp;
        Ok(result)
    }

    /// how many cells have no conflicts
    pub fn score(&self) -> i32 {
        AllPointsIterator::new()
            .map(|p| self.state.status_at(&p))
            .fold(0, |sum, (_, status)| {
                sum + match status {
                    CellStatus::Conflict => 0,
                    CellStatus::NoConflict => 1,
                }
            })
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
        let mut last_log_time = Utc::now();
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

            if best_score == 81 {
                log::trace!("success on generation {}", generations);
                let mut result = best.state.clone();
                let mut all_possible_points = AllPointsIterator::new().collect::<Vec<_>>();
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
                        .ok_or("expected to always find a next item when generating new possible solutions".to_string())?;
                    Possible::new_with_some_random_change(parent, rng)
                })
                .collect::<Result<Vec<_>, _>>()?;
            // always keep the current best around unchanged, just in case
            new_generation.push(best.clone());
            swap(&mut new_generation, &mut population);

            generations += 1;
            let now = Utc::now();
            let time_since_last_log = now.signed_duration_since(last_log_time);
            if time_since_last_log > TimeDelta::seconds(1) {
                log::trace!(
                    "end of generation {}, best score = {}",
                    generations,
                    best_score
                );
                last_log_time = now;
            }
        }
    }
}
