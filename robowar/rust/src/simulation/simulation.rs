use std::{cell::RefCell, ops::RangeInclusive, rc::Rc};

use crate::simulation::{language::Program, physics, vm::VirtualMachine};

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

    // TODO some way to get current state for display

    pub fn step(&mut self) {
        // TODO step simulation
        // TODO step each robot
    }
}
