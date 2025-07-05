use crate::{
    emulator::{Emulator, StepResult},
    parser::program,
};
use chumsky::prelude::*;

mod emulator;
mod instruction_set;
mod parser;

fn main() {
    let input = r"
        main:
            add r1, r2, 42
        foo:
        bar:
            add r34, r34, -1
    "
    .to_string();

    let parser = program();

    let program = parser.parse(&input).into_result().unwrap();
    println!("program: {:?}", program);

    let mut emulator = Emulator::new(program);
    println!("before execution: {:?}", emulator);
    while emulator.step() != StepResult::Halted {
        println!("after stepping: {:?}", emulator);
    }
}
