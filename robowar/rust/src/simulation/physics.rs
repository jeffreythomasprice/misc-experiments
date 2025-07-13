use std::{cell::RefCell, ops::RangeInclusive, rc::Rc};

use rand::Rng;

use crate::math::*;

pub struct Actor {
    collider: Circle<f64>,
    velocity: Vec2<f64>,
    turret_angle: Radians<f64>,
    turret_angular_velocity: Radians<f64>,
}

pub struct Environment {
    bounding_box: Rect<f64>,
    static_line_segments: Vec<Ray2<f64>>,
    actors: Vec<Rc<RefCell<Actor>>>,
}

impl Actor {
    pub fn circle(&self) -> &Circle<f64> {
        &self.collider
    }

    pub fn set_circle(&mut self, value: Circle<f64>) {
        self.collider = value;
    }

    pub fn velocity(&self) -> Vec2<f64> {
        self.velocity
    }

    pub fn set_velocity(&mut self, value: Vec2<f64>) {
        self.velocity = value;
    }

    pub fn turret_angle(&self) -> Radians<f64> {
        self.turret_angle
    }

    pub fn set_turret_angle(&mut self, value: Radians<f64>) {
        self.turret_angle = value;
    }

    pub fn turret_angular_velocity(&self) -> Radians<f64> {
        self.turret_angular_velocity
    }

    pub fn set_turret_angular_velocity(&mut self, value: Radians<f64>) {
        self.turret_angular_velocity = value;
    }
}

impl Environment {
    pub fn new_standard_rectangle(bounding_box: Rect<f64>) -> Self {
        Self {
            bounding_box,
            static_line_segments: vec![
                Ray2::new_between_points(
                    Vec2::new(bounding_box.minimum().x, bounding_box.minimum().y),
                    Vec2::new(bounding_box.maximum().x, bounding_box.minimum().y),
                ),
                Ray2::new_between_points(
                    Vec2::new(bounding_box.maximum().x, bounding_box.minimum().y),
                    Vec2::new(bounding_box.maximum().x, bounding_box.maximum().y),
                ),
                Ray2::new_between_points(
                    Vec2::new(bounding_box.maximum().x, bounding_box.maximum().y),
                    Vec2::new(bounding_box.minimum().x, bounding_box.maximum().y),
                ),
                Ray2::new_between_points(
                    Vec2::new(bounding_box.minimum().x, bounding_box.maximum().y),
                    Vec2::new(bounding_box.minimum().x, bounding_box.minimum().y),
                ),
            ],
            actors: Vec::new(),
        }
    }

    pub fn bounding_box(&self) -> &Rect<f64> {
        &self.bounding_box
    }

    pub fn get_line_segments(&self) -> &[Ray2<f64>] {
        &self.static_line_segments
    }

    pub fn get_actors(&self) -> &Vec<Rc<RefCell<Actor>>> {
        &self.actors
    }

    pub fn clear_actors(&mut self) {
        self.actors.clear();
    }

    /// Creates a new actor with a random position and radius within the bounding box.
    ///
    /// Adds actor to the internal list and also returns a reference to it.
    pub fn add_random_actor(&mut self, actor_size: RangeInclusive<f64>) -> Rc<RefCell<Actor>> {
        // TODO pick a position and radius such that we don't initially collide

        let radius = rand::rng().random_range(actor_size);

        // find a random location by sampling within the bounding box
        let x1 = self.bounding_box.minimum().x + radius;
        let y1 = self.bounding_box.minimum().y + radius;
        let x2 = self.bounding_box.maximum().x - radius;
        let y2 = self.bounding_box.maximum().y - radius;
        let position = Vec2::new(
            rand::rng().random_range(x1..=x2),
            rand::rng().random_range(y1..=y2),
        );

        let velocity = Vec2::new(0., 0.);

        let turret_angle = Radians::from_degrees(rand::rng().random_range((0.)..360.0));

        let turret_angular_velocity = Radians::from_degrees(0.);

        let result = Rc::new(RefCell::new(Actor {
            collider: Circle::new(position, radius),
            velocity,
            turret_angle,
            turret_angular_velocity,
        }));
        self.actors.push(result.clone());
        result
    }

    pub fn step(&mut self, time: f64) {
        /*
        TODO step

        move all actors according to their velocity and turret angle
        slide along walls, don't update velocity
        */
        todo!()
    }

    /// Finds the first intersection with another actor or the world, starting from the actor's position and extending in the direction of
    /// the actor's turret.
    pub fn actor_scan(&self, starting_actor: &Actor) -> f64 {
        /*
        TODO actor scan

        let r = the ray starting from starting_actor.position and extending in the direction the turret is pointing

        find first intersection:
        - between r and each line segment
        - between r and each other actor's collider

        if no such intersections exist, return float max
        */

        let scan_ray = Ray2::new(
            *starting_actor.circle().center(),
            starting_actor.turret_angle().cos_sin_vec2(),
        );

        todo!()
    }
}
