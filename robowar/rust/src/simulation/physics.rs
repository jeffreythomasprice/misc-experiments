use std::{
    cell::RefCell,
    ops::{RangeBounds, RangeInclusive},
    rc::Rc,
};

use rand::Rng;

use crate::math::*;

pub struct LineSegment {
    a: Vec2<f64>,
    b: Vec2<f64>,
}

pub struct Circle {
    center: Vec2<f64>,
    radius: f64,
}

pub struct Actor {
    collider: Circle,
    velocity: Vec2<f64>,
    // TODO newtype for radians?
    turret_angle: f64,
    turret_angular_velocity: f64,
}

pub struct Environment {
    bounding_box: Rect<f64>,
    static_line_segments: Vec<LineSegment>,
    actors: Vec<Rc<RefCell<Actor>>>,
}

impl Actor {
    pub fn zero() -> Self {
        Self {
            collider: Circle {
                center: Vec2::new(0., 0.),
                radius: 0.,
            },
            velocity: Vec2::new(0., 0.),
            turret_angle: 0.,
            turret_angular_velocity: 0.,
        }
    }

    pub fn position(&self) -> Vec2<f64> {
        self.collider.center
    }

    pub fn set_position(&mut self, value: Vec2<f64>) {
        self.collider.center = value;
    }

    pub fn velocity(&self) -> Vec2<f64> {
        self.velocity
    }

    pub fn set_velocity(&mut self, value: Vec2<f64>) {
        self.velocity = value;
    }

    pub fn turret_angle(&self) -> f64 {
        self.turret_angle
    }

    pub fn set_turret_angle(&mut self, value: f64) {
        self.turret_angle = value;
    }

    pub fn turret_angular_velocity(&self) -> f64 {
        self.turret_angular_velocity
    }

    pub fn set_turret_angular_velocity(&mut self, value: f64) {
        self.turret_angular_velocity = value;
    }
}

impl Environment {
    pub fn new_standard_rectangle(
        bounding_box: Rect<f64>,
        num_actors: usize,
        actor_size: RangeInclusive<f64>,
    ) -> Self {
        let mut result = Self {
            bounding_box,
            static_line_segments: vec![
                LineSegment {
                    a: Vec2::new(bounding_box.minimum().x, bounding_box.minimum().y),
                    b: Vec2::new(bounding_box.maximum().x, bounding_box.minimum().y),
                },
                LineSegment {
                    a: Vec2::new(bounding_box.maximum().x, bounding_box.minimum().y),
                    b: Vec2::new(bounding_box.maximum().x, bounding_box.maximum().y),
                },
                LineSegment {
                    a: Vec2::new(bounding_box.maximum().x, bounding_box.maximum().y),
                    b: Vec2::new(bounding_box.minimum().x, bounding_box.maximum().y),
                },
                LineSegment {
                    a: Vec2::new(bounding_box.minimum().x, bounding_box.maximum().y),
                    b: Vec2::new(bounding_box.minimum().x, bounding_box.minimum().y),
                },
            ],
            actors: Vec::with_capacity(num_actors),
        };
        for _ in 0..num_actors {
            result
                .actors
                .push(result.create_random_actor(actor_size.clone()));
        }
        return result;
    }

    /// Finds the first intersection with another actor or the world, starting from the actor's position and extending in the direction of
    /// the actor's turret.
    pub fn actor_scan(&self, starting_actor: Rc<RefCell<Actor>>) -> f64 {
        /*
        TODO actor scan

        let r = the ray starting from starting_actor.position and extending in the direction the turret is pointing

        find first intersection:
        - between r and each line segment
        - between r and each other actor's collider

        if no such intersections exist, return float max
        */
        todo!()
    }

    fn create_random_actor(&self, actor_size: RangeInclusive<f64>) -> Rc<RefCell<Actor>> {
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

        let turret_angle = rand::rng().random_range((0.)..std::f64::consts::TAU);

        let turret_angular_velocity = 0.;

        Rc::new(RefCell::new(Actor {
            collider: Circle {
                center: position,
                radius,
            },
            velocity,
            turret_angle,
            turret_angular_velocity,
        }))
    }
}
