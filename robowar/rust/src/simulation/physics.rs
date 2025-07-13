use std::{cell::RefCell, ops::RangeInclusive, rc::Rc};

use color_eyre::eyre::{Result, eyre};
use rand::Rng;
use rapier2d_f64::{
    na::{Isometry2, Matrix2x1, OPoint, Point2, iter::ColumnIter},
    prelude::*,
};

use crate::math::*;

pub struct Actor {
    rigid_body_set: Rc<RefCell<RigidBodySet>>,
    rigid_body_handle: RigidBodyHandle,
    radius: f64,
    turret_angle: Radians<f64>,
    turret_angular_velocity: Radians<f64>,
}

pub struct Environment {
    rigid_body_set: Rc<RefCell<RigidBodySet>>,
    collider_set: ColliderSet,
    polyline_handle: ColliderHandle,
    gravity: nalgebra::Matrix<
        f64,
        nalgebra::Const<2>,
        nalgebra::Const<1>,
        nalgebra::ArrayStorage<f64, 2, 1>,
    >,
    integration_parameters: IntegrationParameters,
    physics_pipeline: PhysicsPipeline,
    island_manager: IslandManager,
    broad_phase: BroadPhaseMultiSap,
    narrow_phase: NarrowPhase,
    impulse_joint_set: ImpulseJointSet,
    multibody_joint_set: MultibodyJointSet,
    ccd_solver: CCDSolver,
    query_pipeline: QueryPipeline,
    physics_hooks: (),
    event_handler: (),
    actors: Vec<Rc<RefCell<Actor>>>,
}

impl Actor {
    // TODO de-duplicate all the bits that get the rigid body

    pub fn position(&self) -> Result<Vec2<f64>> {
        let rigid_body_set = self.rigid_body_set.borrow();
        let rigid_body = rigid_body_set.get(self.rigid_body_handle).ok_or(eyre!(
            "failed to find rigid body on environment, as this actor been removed?"
        ))?;
        let result = rigid_body.translation();
        Ok(Vec2::new(result.x, result.y))
    }

    pub fn velocity(&self) -> Result<Vec2<f64>> {
        let rigid_body_set = self.rigid_body_set.borrow();
        let rigid_body = rigid_body_set.get(self.rigid_body_handle).ok_or(eyre!(
            "failed to find rigid body on environment, as this actor been removed?"
        ))?;
        let result = rigid_body.vels().linvel;
        Ok(Vec2::new(result.x, result.y))
    }

    pub fn set_velocity(&mut self, value: Vec2<f64>) -> Result<()> {
        let mut rigid_body_set = self.rigid_body_set.borrow_mut();
        let rigid_body = rigid_body_set.get_mut(self.rigid_body_handle).ok_or(eyre!(
            "failed to find rigid body on environment, as this actor been removed?"
        ))?;
        rigid_body.set_vels(
            RigidBodyVelocity::new(Matrix2x1::new(value.x, value.y), 0.),
            true,
        );
        Ok(())
    }

    pub fn radius(&self) -> f64 {
        self.radius
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
        let mut rigid_body_set = RigidBodySet::new();
        let mut collider_set = ColliderSet::new();

        let polyline = ColliderBuilder::polyline(
            vec![
                Point2::new(bounding_box.minimum().x, bounding_box.minimum().y),
                Point2::new(bounding_box.maximum().x, bounding_box.minimum().y),
                Point2::new(bounding_box.maximum().x, bounding_box.maximum().y),
                Point2::new(bounding_box.minimum().x, bounding_box.maximum().y),
            ],
            Some(vec![[0, 1], [1, 2], [2, 3], [3, 0]]),
        )
        .build();
        let polyline_handle = collider_set.insert(polyline);

        let gravity = vector![0., 0.];
        let integration_parameters = IntegrationParameters::default();
        let physics_pipeline = PhysicsPipeline::new();
        let island_manager = IslandManager::new();
        let broad_phase = DefaultBroadPhase::new();
        let narrow_phase = NarrowPhase::new();
        let impulse_joint_set = ImpulseJointSet::new();
        let multibody_joint_set = MultibodyJointSet::new();
        let ccd_solver = CCDSolver::new();
        let query_pipeline = QueryPipeline::new();
        let physics_hooks = ();
        let event_handler = ();

        Self {
            rigid_body_set: Rc::new(RefCell::new(rigid_body_set)),
            collider_set,
            polyline_handle,
            gravity,
            integration_parameters,
            physics_pipeline,
            island_manager,
            broad_phase,
            narrow_phase,
            impulse_joint_set,
            multibody_joint_set,
            ccd_solver,
            query_pipeline,
            physics_hooks,
            event_handler,
            actors: Vec::new(),
        }
    }

    pub fn bounding_box(&self) -> Result<Rect<f64>> {
        // TODO include actors too?
        let bbox = self
            .collider_set
            .iter()
            .map(|c| c.1.compute_aabb())
            .reduce(|result, bbox| result.merged(&bbox))
            .ok_or(eyre!("no colliders, expected at least one"))?;
        Ok(Rect::new_with_points(&[
            Vec2::new(bbox.mins.x, bbox.mins.y),
            Vec2::new(bbox.maxs.x, bbox.maxs.y),
        ])
        .unwrap())
    }

    pub fn get_line_segments(&self) -> Result<Vec<Ray2<f64>>> {
        let collider = self
            .collider_set
            .get(self.polyline_handle)
            .ok_or(eyre!("failed to get polyline"))?;
        let polyline = collider
            .shape()
            .as_polyline()
            .ok_or(eyre!("failed to unwrap collider as polyline"))?;
        Ok(polyline
            .segments()
            .map(|s| Ray2::new_between_points(Vec2::new(s.a.x, s.a.y), Vec2::new(s.b.x, s.b.y)))
            .collect::<Vec<_>>())
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
    pub fn add_random_actor(
        &mut self,
        actor_size: RangeInclusive<f64>,
    ) -> Result<Rc<RefCell<Actor>>> {
        // TODO pick a position and radius such that we don't initially collide

        let radius = rand::rng().random_range(actor_size);

        // find a random location by sampling within the bounding box
        let bbox = self.bounding_box()?;
        let x1 = bbox.minimum().x + radius;
        let y1 = bbox.minimum().y + radius;
        let x2 = bbox.maximum().x - radius;
        let y2 = bbox.maximum().y - radius;
        let position = Vec2::new(
            rand::rng().random_range(x1..=x2),
            rand::rng().random_range(y1..=y2),
        );

        let turret_angle = Radians::from_degrees(rand::rng().random_range((0.)..360.0));

        let turret_angular_velocity = Radians::from_degrees(0.);

        let rigid_body = RigidBodyBuilder::dynamic()
            .translation(Matrix2x1::new(position.x, position.y))
            .build();
        let collider = ColliderBuilder::ball(radius).restitution(0.7).build();
        let mut rigid_body_set = self.rigid_body_set.borrow_mut();
        let rigid_body_handle = rigid_body_set.insert(rigid_body);
        self.collider_set
            .insert_with_parent(collider, rigid_body_handle, &mut rigid_body_set);

        let result = Rc::new(RefCell::new(Actor {
            rigid_body_set: self.rigid_body_set.clone(),
            rigid_body_handle,
            radius,
            turret_angle,
            turret_angular_velocity,
        }));
        self.actors.push(result.clone());
        Ok(result)
    }

    pub fn step(&mut self, time: f64) {
        // update the physics engine
        let mut rigid_body_set = self.rigid_body_set.borrow_mut();
        // TODO only step if enough time has passed?
        self.physics_pipeline.step(
            &self.gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            Some(&mut self.query_pipeline),
            &self.physics_hooks,
            &self.event_handler,
        );

        // turrets
        for actor in self.actors.iter_mut() {
            let mut actor = actor.borrow_mut();
            let new_turret_angle = actor.turret_angle() + actor.turret_angular_velocity() * time;
            actor.set_turret_angle(new_turret_angle);
        }
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

        todo!()
    }
}
