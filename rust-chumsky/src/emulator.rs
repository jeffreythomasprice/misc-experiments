use std::{collections::HashMap, fmt::Debug};

use crate::instruction_set::{
    Instruction, Program, Register32, Register64, RegisterOrLiteral32, RegisterOrLiteral64,
};

pub struct Emulator<'a> {
    registers_32: HashMap<Register32, u32>,
    registers_64: HashMap<Register64, u64>,
    program: Program<'a>,
    program_counter: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepResult {
    NotHalted,
    Halted,
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

    pub fn step(&mut self) -> StepResult {
        match self.next_instruction() {
            Some(Instruction::Add32 {
                destination,
                left,
                right,
            }) => {
                self.registers_32.insert(
                    destination,
                    self.get_register_or_literal_32(&left)
                        + self.get_register_or_literal_32(&right),
                );
            }
            Some(Instruction::Add64 {
                destination,
                left,
                right,
            }) => {
                self.registers_64.insert(
                    destination,
                    self.get_register_or_literal_64(&left)
                        + self.get_register_or_literal_64(&right),
                );
            }
            None => return StepResult::Halted,
        }
        StepResult::NotHalted
    }

    fn get_register_32(&self, r: Register32) -> u32 {
        *self.registers_32.get(&r).unwrap_or(&0)
    }

    fn get_register_64(&self, r: Register64) -> u64 {
        *self.registers_64.get(&r).unwrap_or(&0)
    }

    fn get_register_or_literal_32(&self, value: &RegisterOrLiteral32) -> u32 {
        match value {
            RegisterOrLiteral32::Register(r) => self.get_register_32(*r),
            RegisterOrLiteral32::U32(value) => *value,
            RegisterOrLiteral32::I32(value) => *value as u32,
        }
    }

    fn get_register_or_literal_64(&self, value: &RegisterOrLiteral64) -> u64 {
        match value {
            RegisterOrLiteral64::Register(r) => self.get_register_64(*r),
            RegisterOrLiteral64::U64(value) => *value,
            RegisterOrLiteral64::I64(value) => *value as u64,
        }
    }

    fn next_instruction(&mut self) -> Option<Instruction> {
        let result = self.program.instructions.get(self.program_counter).cloned();
        if result.is_some() {
            self.program_counter += 1;
        }
        result
    }
}

impl<'a> Debug for Emulator<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO better debug for emulator?
        f.debug_struct("Emulator")
            .field("registers_32", &self.registers_32)
            .field("registers_64", &self.registers_64)
            // .field("program", &self.program)
            .field("program_counter", &self.program_counter)
            .finish()
    }
}
