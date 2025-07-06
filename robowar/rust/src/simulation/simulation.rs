use crate::{
    math::{Rect, Vec2},
    simulation::robot::Robot,
};

pub struct Simulation {
    bounds: Rect<f64>,
    robots: Vec<Robot>,
}

// TODO simulation
