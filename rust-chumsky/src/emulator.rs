use std::{collections::HashMap, fmt::Debug};

use crate::instruction_set::{
    Instruction, Program, Register32, Register64, RegisterOrLiteral32, RegisterOrLiteral64,
};

pub struct Emulator<'a> {
    registers: [u32; 8],
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
            registers: [0; 8],
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
                self.set_register_32(
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
                self.set_register_64(
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
        self.registers[register_32_to_index(r)]
    }

    fn set_register_32(&mut self, r: Register32, value: u32) {
        self.registers[register_32_to_index(r)] = value;
    }

    fn get_register_64(&self, r: Register64) -> u64 {
        let low = self.get_register_32(r.low()) as u64;
        let high = self.get_register_32(r.high()) as u64;
        (high << 32) | low
    }

    fn set_register_64(&mut self, r: Register64, value: u64) {
        let low = (value & 0xffffffff) as u32;
        let high = (value >> 32) as u32;
        self.set_register_32(r.low(), low);
        self.set_register_32(r.high(), high);
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
            .field("registers", &self.registers)
            .field("program_counter", &self.program_counter)
            .finish()
    }
}

fn register_32_to_index(r: Register32) -> usize {
    match r {
        Register32::R1 => 0,
        Register32::R2 => 1,
        Register32::R3 => 2,
        Register32::R4 => 3,
        Register32::R5 => 4,
        Register32::R6 => 5,
        Register32::R7 => 6,
        Register32::R8 => 7,
    }
}
