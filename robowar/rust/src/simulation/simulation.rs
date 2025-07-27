use std::{cell::RefCell, ops::RangeInclusive, rc::Rc, time::Duration};

use color_eyre::eyre::{Result, eyre};
use tracing::*;

use crate::simulation::{
    ecs,
    language::Program,
    physics,
    vm::{StepError, VirtualMachine},
};

struct Robot {
    // TODO actors should have a better user data than just their index?
    actor: Rc<RefCell<physics::Actor<ecs::Id>>>,
    vm: VirtualMachine,
}

pub struct Simulation {
    physics_environment: physics::Environment<ecs::Id>,
    robots: ecs::ComponentSystem<Robot>,
    total_time: Duration,
}

impl Simulation {
    pub fn new(
        mut physics_environment: physics::Environment<ecs::Id>,
        programs: Vec<Rc<Program>>,
        actor_size: RangeInclusive<f64>,
    ) -> Result<Self> {
        physics_environment.clear_actors();
        let mut robots = ecs::ComponentSystem::new();
        for program in programs.iter() {
            robots.insert_factory(|id| {
                Ok(Robot {
                    actor: physics_environment.add_random_actor(actor_size.clone(), id)?,
                    vm: VirtualMachine::new(program.clone()),
                })
            })?;
        }
        Ok(Self {
            physics_environment,
            robots,
            total_time: Duration::ZERO,
        })
    }

    pub fn physics_environment(&self) -> &physics::Environment<ecs::Id> {
        &self.physics_environment
    }

    pub fn update(&mut self, elapsed_time: Duration) -> Result<()> {
        // update physics environment
        self.total_time += elapsed_time;
        self.physics_environment
            .step(self.total_time.as_secs_f64(), |e| {
                match e {
                    physics::CollisionEvent::Started(actor1, actor2) => {
                        match (actor1, actor2) {
                            (physics::Collidable::Actor(a1), physics::Collidable::Actor(a2)) => {
                                let actor1 = a1.borrow();
                                let actor2 = a2.borrow();
                                // TODO de-duplicate all the robot lookups
                                let robot1 =
                                    self.robots.get(*actor1.user_data()).ok_or_else(|| {
                                        eyre!("Actor with id {:?} not found", actor1.user_data())
                                    })?;
                                let robot2 =
                                    self.robots.get(*actor2.user_data()).ok_or_else(|| {
                                        eyre!("Actor with id {:?} not found", actor2.user_data())
                                    })?;
                                info!(
                                    "TODO collision STARTED between two robots {:?} and {:?}",
                                    actor1.user_data(),
                                    actor2.user_data()
                                );
                                // TODO actually do something with robots
                            }
                            (physics::Collidable::Actor(a), physics::Collidable::Environment)
                            | (physics::Collidable::Environment, physics::Collidable::Actor(a)) => {
                                let actor = a.borrow();
                                let robot =
                                    self.robots.get(*actor.user_data()).ok_or_else(|| {
                                        eyre!("Actor with id {:?} not found", actor.user_data())
                                    })?;
                                info!(
                                    "TODO collision STARTED between robot {:?} and environment",
                                    actor.user_data()
                                );
                                // TODO actually do something with robot
                            }
                            // no robots involved, impossible?
                            _ => (),
                        }
                    }
                    // TODO handle collision ends?
                    physics::CollisionEvent::Stopped(actor1, actor2) => (),
                };
                Ok(())
            });

        // TODO need to update robots only until they match current time
        for (_, robot) in self.robots.iter_mut() {
            let mut actor = robot.actor.borrow_mut();
            robot.vm.update_to_match_actor(&actor)?;
            match robot.vm.step(&self.physics_environment, &actor) {
                Ok(new_clock) => {
                    // TODO what to do with new_clock?
                }
                Err(StepError::Halted) => {
                    // TODO what to do about halted robot?
                }
                _ => (),
            }
            robot.vm.update_actor_match_vm(&mut actor)?;
        }

        Ok(())
    }
}
