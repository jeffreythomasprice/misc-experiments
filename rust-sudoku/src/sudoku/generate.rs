use super::{Cell, GameState, Number, Point};
use crate::Result;
use rand::seq::SliceRandom;

impl GameState {
    pub fn new_random() -> Result<Self> {
        // TODO accept rng as a param
        let mut rng = rand::thread_rng();

        let mut result = GameState::new();

        // TODO ensure puzzle is solvable
        for i in 0..20 {
            let all_possible_points = Point::all_possible_values();
            let possible_points = all_possible_points
                .iter()
                .filter(|p| result[**p] == Cell::Empty)
                .collect::<Vec<_>>();
            let p = possible_points.choose(&mut rng).ok_or(format!(
                "expected at least one remaining empty spot to choose from during puzzle creation"
            ))?;
            let all_possible_numbers = Number::all();
            let n = all_possible_numbers
                .choose(&mut rng)
                .ok_or(format!("expected to be able to pick random number"))?;
            result[**p] = Cell::PuzzleInput(*n);
        }

        Ok(result)
    }
}
