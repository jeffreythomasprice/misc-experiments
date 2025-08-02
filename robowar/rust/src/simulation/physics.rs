use std::{cell::RefCell, ops::RangeInclusive, rc::Rc};

use color_eyre::eyre::{Result, eyre};
use rand::Rng;
use rapier2d_f64::{
    crossbeam,
    na::{Matrix2x1, Point2},
    prelude::*,
};
use tracing::*;

use crate::{
    math::*,
    simulation::ecs::{self},
};

#[derive(Clone)]
pub enum Collidable<ActorData> {
    Actor(Rc<RefCell<Actor<ActorData>>>),
    Environment,
}

pub struct Actor<T> {
    rigid_body_set: Rc<RefCell<RigidBodySet>>,
    rigid_body_handle: RigidBodyHandle,
    radius: f64,
    turret_angle: Radians<f64>,
    turret_angular_velocity: Radians<f64>,
    user_data: T,
}

#[derive(Clone)]
pub enum CollisionEvent<ActorData> {
    Started(Collidable<ActorData>, Collidable<ActorData>),
    Stopped(Collidable<ActorData>, Collidable<ActorData>),
}

pub struct Environment<ActorData> {
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
    collision_recv: crossbeam::channel::Receiver<rapier2d_f64::geometry::CollisionEvent>,
    contact_force_recv: crossbeam::channel::Receiver<ContactForceEvent>,
    event_handler: ChannelEventCollector,
    actors: Vec<Rc<RefCell<Actor<ActorData>>>>,
    collidables: ecs::ComponentSystem<Collidable<ActorData>>,
}

impl<T> Actor<T> {
    pub fn mass(&self) -> Result<f64> {
        Ok(self
            .rigid_body(&self.rigid_body_set.borrow(), self.rigid_body_handle)?
            .mass())
    }

    pub fn position(&self) -> Result<Vec2<f64>> {
        let result = *self
            .rigid_body(&self.rigid_body_set.borrow(), self.rigid_body_handle)?
            .translation();
        Ok(Vec2::new(result.x, result.y))
    }

    pub fn velocity(&self) -> Result<Vec2<f64>> {
        let result = self
            .rigid_body(&self.rigid_body_set.borrow(), self.rigid_body_handle)?
            .vels()
            .linvel;
        Ok(Vec2::new(result.x, result.y))
    }

    pub fn set_velocity(&mut self, value: Vec2<f64>) -> Result<()> {
        self.rigid_body_mut(
            &mut self.rigid_body_set.borrow_mut(),
            self.rigid_body_handle,
        )?
        .set_linvel(Matrix2x1::new(value.x, value.y), true);
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

    pub fn user_data(&self) -> &T {
        &self.user_data
    }

    fn rigid_body<'a: 'b, 'b>(
        &self,
        rigid_body_set: &'a RigidBodySet,
        handle: RigidBodyHandle,
    ) -> Result<&'b RigidBody> {
        rigid_body_set
            .get(handle)
            .ok_or(eyre!("rigid body not found: {handle:?}"))
    }

    fn rigid_body_mut<'a: 'b, 'b>(
        &self,
        rigid_body_set: &'a mut RigidBodySet,
        handle: RigidBodyHandle,
    ) -> Result<&'b mut RigidBody> {
        rigid_body_set
            .get_mut(handle)
            .ok_or(eyre!("rigid body not found: {handle:?}"))
    }
}

impl<ActorData> Environment<ActorData>
where
    ActorData: Clone,
{
    pub fn new_standard_rectangle(bounding_box: Rect<f64>) -> Self {
        let mut collidables = ecs::ComponentSystem::new();
        let environment_id = collidables.insert(Collidable::Environment);

        let rigid_body_set = RigidBodySet::new();
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
        .user_data(environment_id.0 as u128)
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
        let (collision_send, collision_recv) = crossbeam::channel::unbounded();
        let (contact_force_send, contact_force_recv) = crossbeam::channel::unbounded();
        let event_handler = ChannelEventCollector::new(collision_send, contact_force_send);

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
            collision_recv,
            contact_force_recv,
            event_handler,
            actors: Vec::new(),
            collidables,
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

    pub fn actors_iter(&self) -> impl Iterator<Item = &Rc<RefCell<Actor<ActorData>>>> {
        self.actors.iter()
    }

    pub fn clear_actors(&mut self) {
        self.actors.clear();
        // TODO clear actors from collidables
    }

    /// Creates a new actor with a random position and radius within the bounding box.
    ///
    /// Adds actor to the internal list and also returns a reference to it.
    pub fn add_random_actor(
        &mut self,
        actor_size: RangeInclusive<f64>,
        user_data: ActorData,
    ) -> Result<Rc<RefCell<Actor<ActorData>>>> {
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
        let mut rigid_body_set = self.rigid_body_set.borrow_mut();
        let rigid_body_handle = rigid_body_set.insert(rigid_body);

        let result = Rc::new(RefCell::new(Actor {
            rigid_body_set: self.rigid_body_set.clone(),
            rigid_body_handle,
            radius,
            turret_angle,
            turret_angular_velocity,
            user_data,
        }));
        let id = self.collidables.insert(Collidable::Actor(result.clone()));
        self.actors.push(result.clone());

        let collider = ColliderBuilder::ball(radius)
            .restitution(0.7)
            .active_events(ActiveEvents::COLLISION_EVENTS)
            .user_data(id.0 as u128)
            .build();
        self.collider_set
            .insert_with_parent(collider, rigid_body_handle, &mut rigid_body_set);

        Ok(result)
    }

    pub fn step<F>(&mut self, time: f64, collision_callback: F)
    where
        F: Fn(CollisionEvent<ActorData>) -> Result<()>,
    {
        // TODO needs some way to detect collisions, update robot health

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

        // collision events
        while let Ok(collision_event) = self.collision_recv.try_recv() {
            trace!("collision event: {:?}", collision_event);
            if let Err(e) = self.handle_collision_event(&collision_event, &collision_callback) {
                error!("failed to handle collision event: {:?}", e);
            }
        }
        while let Ok(contact_force_event) = self.contact_force_recv.try_recv() {
            trace!("contact force event: {:?}", contact_force_event);
        }
    }

    /// Finds the first intersection with another actor or the world, starting from the actor's position and extending in the direction of
    /// the actor's turret.
    pub fn actor_scan(&self, starting_actor: &Actor<ActorData>) -> f64 {
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

    fn handle_collision_event<F>(
        &self,
        collision_event: &rapier2d_f64::geometry::CollisionEvent,
        collision_callback: F,
    ) -> Result<()>
    where
        F: Fn(CollisionEvent<ActorData>) -> Result<()>,
    {
        // TODO de-duplicate collider exists checks?
        let collider1 = self
            .collider_set
            .get(collision_event.collider1())
            .ok_or(eyre!("failed to find collider1 in collision event"))?;
        let collider2 = self
            .collider_set
            .get(collision_event.collider2())
            .ok_or(eyre!("failed to find collider2 in collision event"))?;
        if let Some(a) = self.collidables.get(ecs::Id(collider1.user_data as usize))
            && let Some(b) = self.collidables.get(ecs::Id(collider2.user_data as usize))
        {
            let a = a.clone();
            let b = b.clone();
            if collision_event.started() {
                collision_callback(CollisionEvent::Started(a, b))?;
            } else if collision_event.stopped() {
                collision_callback(CollisionEvent::Stopped(a, b))?;
            }
        }

        Ok(())
    }
}
