use std::{cell::RefCell, ops::RangeInclusive, rc::Rc};

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
        }
    }

    pub fn physics_environment(&self) -> &physics::Environment {
        &self.physics_environment
    }

    pub fn step(&mut self) {
        /*
        TODO how step should really work

        get current clock of every vm
        find all the ones tied for lowest clock
        step all those robots
        find the lowest clock out of those, and take the different between the old value
        step physics that amount
        */

        // TODO how much time to step each physics simulation?
        self.physics_environment.step(1.);

        for robot in self.robots.iter_mut() {
            let actor = robot.actor.borrow();
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
        }

        // TODO step each robot
    }
}
