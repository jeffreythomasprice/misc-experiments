use std::{cell::RefCell, ops::RangeInclusive, rc::Rc, time::Duration};

use crate::{
    math::Ray2,
    simulation::{
        language::Program,
        physics,
        vm::{StepError, VirtualMachine},
    },
};

struct Robots {
    actor: Rc<RefCell<physics::Actor>>,
    vm: VirtualMachine,
}

pub struct Simulation {
    physics_environment: physics::Environment,
    robots: Vec<Robots>,
    total_time: Duration,
}

impl Simulation {
    pub fn new(
        mut physics_environment: physics::Environment,
        programs: Vec<Rc<Program>>,
        actor_size: RangeInclusive<f64>,
    ) -> Self {
        physics_environment.clear_actors();
        let mut robots = Vec::new();
        for program in programs.iter() {
            let robot = physics_environment.add_random_actor(actor_size.clone());
            robots.push(Robots {
                actor: robot,
                vm: VirtualMachine::new(program.clone()),
            });
        }
        Self {
            physics_environment,
            robots,
            total_time: Duration::ZERO,
        }
    }

    pub fn physics_environment(&self) -> &physics::Environment {
        &self.physics_environment
    }

    pub fn update(&mut self, elapsed_time: Duration) {
        // update physics environment
        self.total_time += elapsed_time;
        self.physics_environment.step(self.total_time.as_secs_f64());

        // TODO need to update robots only until they match current time
        for robot in self.robots.iter_mut() {
            let mut actor = robot.actor.borrow_mut();
            robot.vm.update_to_match_actor(&actor);
            match robot.vm.step(&self.physics_environment, &actor) {
                Ok(new_clock) => {
                    // TODO what to do with new_clock?
                }
                Err(StepError::Halted) => {
                    // TODO what to do about halted robot?
                }
                _ => (),
            }
            robot.vm.update_actor_match_vm(&mut actor);
        }
    }
}
