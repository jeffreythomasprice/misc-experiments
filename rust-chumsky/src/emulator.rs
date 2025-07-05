use std::collections::HashMap;

use crate::instruction_set::{Program, Register32, Register64};

pub struct Emulator<'a> {
    registers_32: HashMap<Register32, u32>,
    registers_64: HashMap<Register64, u64>,
    program: Program<'a>,
    program_counter: usize,
}

impl<'a> Emulator<'a> {
    pub fn new(program: Program<'a>) -> Self {
        Self {
            registers_32: HashMap::new(),
            registers_64: HashMap::new(),
            program,
            program_counter: 0,
        }
    }

    pub fn step(&mut self) {
        todo!()
    }
}
