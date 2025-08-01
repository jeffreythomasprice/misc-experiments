use std::{
    fmt::Debug,
    num::TryFromIntError,
    ops::{Add, AddAssign},
    rc::Rc,
};

use color_eyre::eyre::Result;

use crate::{
    math::*,
    simulation::{language::*, physics},
};

#[derive(Debug, Clone, Copy)]
pub struct ClockTime(u64);

impl Add for ClockTime {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for ClockTime {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

struct ResolvedValue<T> {
    value: T,
    clock_cost: ClockTime,
}

pub enum StepError {
    Halted,
    TryFromIntError(TryFromIntError),
    StackUnderflow,
    AddressOutOfBounds,
}

#[derive(Debug, Clone, Copy)]
enum StackOrHeapValue {
    U64(u64),
    F64(f64),
}

pub struct VirtualMachine {
    program: Rc<Program>,

    stack: Vec<StackOrHeapValue>,
    heap: Vec<StackOrHeapValue>,

    program_counter: ProgramPointer,
    clock: ClockTime,
    halted: bool,

    health: f64,
    energy: f64,

    position: Vec2<f64>,
    velocity: Vec2<f64>,
    turret_angle: Radians<f64>,
    turrent_angular_velocity: Radians<f64>,

    register_general_purpose_u64: [u64; 8],
    register_general_purpose_f64: [f64; 8],
}

impl VirtualMachine {
    pub fn new(program: Rc<Program>) -> Self {
        let stack = vec![StackOrHeapValue::U64(0); program.stack_size];
        let heap = vec![StackOrHeapValue::U64(0); program.heap_size];
        Self {
            program,
            stack,
            heap,

            program_counter: 0.into(),
            clock: ClockTime(0),
            halted: false,

            // TODO health should be configurable?
            health: 100.0,
            // TODO energy should be configurable?
            energy: 100.0,

            position: Vec2::new(0., 0.),
            velocity: Vec2::new(0., 0.),
            turret_angle: Radians(0.),
            turrent_angular_velocity: Radians(0.),

            register_general_purpose_u64: [0; 8],
            register_general_purpose_f64: [0.; 8],
        }
    }

    pub fn update_to_match_actor<ActorData>(
        &mut self,
        actor: &physics::Actor<ActorData>,
    ) -> Result<()>
    where
        ActorData: Clone,
    {
        self.position = actor.position()?;
        self.velocity = actor.velocity()?;
        self.turret_angle = actor.turret_angle();
        self.turrent_angular_velocity = actor.turret_angular_velocity();
        Ok(())
    }

    pub fn update_actor_match_vm<ActorData>(
        &self,
        actor: &mut physics::Actor<ActorData>,
    ) -> Result<()>
    where
        ActorData: Clone,
    {
        actor.set_velocity(self.velocity)?;
        actor.set_turret_angular_velocity(self.turrent_angular_velocity);
        Ok(())
    }

    pub fn step<ActorData>(
        &mut self,
        environment: &physics::Environment<ActorData>,
        actor: &physics::Actor<ActorData>,
    ) -> Result<ClockTime, StepError>
    where
        ActorData: Clone,
    {
        if self.halted {
            return Err(StepError::Halted);
        }
        match self.read_next_instruction() {
            Some(Instruction::SetU64 {
                destination,
                source,
            }) => {
                let source = self.resolve_source_u64(source);
                self.write_destination_u64(destination, source.value);
                self.clock += source.clock_cost;
            }
            Some(Instruction::SetF64 {
                destination,
                source,
            }) => {
                let source = self.resolve_source_f64(source, environment, actor);
                self.write_destination_f64(destination, source.value);
                self.clock += source.clock_cost;
            }
            Some(Instruction::AddU64 {
                destination,
                left,
                right,
            }) => self
                .binary_operator_common_u64(destination, left, right, |left, right| left + right)?,
            Some(Instruction::AddF64 {
                destination,
                left,
                right,
            }) => self.binary_operator_common_f64(
                destination,
                left,
                right,
                |left, right| left + right,
                environment,
                actor,
            )?,
            Some(Instruction::SubU64 {
                destination,
                left,
                right,
            }) => self
                .binary_operator_common_u64(destination, left, right, |left, right| left - right)?,
            Some(Instruction::SubF64 {
                destination,
                left,
                right,
            }) => self.binary_operator_common_f64(
                destination,
                left,
                right,
                |left, right| left - right,
                environment,
                actor,
            )?,
            Some(Instruction::MulU64 {
                destination,
                left,
                right,
            }) => self
                .binary_operator_common_u64(destination, left, right, |left, right| left * right)?,
            Some(Instruction::MulF64 {
                destination,
                left,
                right,
            }) => self.binary_operator_common_f64(
                destination,
                left,
                right,
                |left, right| left * right,
                environment,
                actor,
            )?,
            Some(Instruction::DivU64 {
                destination,
                left,
                right,
            }) => self
                .binary_operator_common_u64(destination, left, right, |left, right| left / right)?,
            Some(Instruction::DivF64 {
                destination,
                left,
                right,
            }) => self.binary_operator_common_f64(
                destination,
                left,
                right,
                |left, right| left / right,
                environment,
                actor,
            )?,
            Some(Instruction::Jump { address }) => {
                let address = self.resolve_source_u64(address);
                self.program_counter = address
                    .value
                    .try_into()
                    .map_err(StepError::TryFromIntError)?;
                self.clock += address.clock_cost;
            }
            Some(Instruction::JumpEqualU64 {
                address,
                left,
                right,
            }) => self.jump_comparison_u64(address, left, right, |left, right| left == right)?,
            Some(Instruction::JumpEqualF64 {
                address,
                left,
                right,
            }) => self.jump_comparison_f64(
                address,
                left,
                right,
                |left, right| left != right,
                environment,
                actor,
            )?,
            Some(Instruction::JumpNotEqualU64 {
                address,
                left,
                right,
            }) => self.jump_comparison_u64(address, left, right, |left, right| left != right)?,
            Some(Instruction::JumpNotEqualF64 {
                address,
                left,
                right,
            }) => self.jump_comparison_f64(
                address,
                left,
                right,
                |left, right| left != right,
                environment,
                actor,
            )?,
            Some(Instruction::JumpLessThanU64 {
                address,
                left,
                right,
            }) => self.jump_comparison_u64(address, left, right, |left, right| left < right)?,
            Some(Instruction::JumpLessThanF64 {
                address,
                left,
                right,
            }) => self.jump_comparison_f64(
                address,
                left,
                right,
                |left, right| left < right,
                environment,
                actor,
            )?,
            Some(Instruction::JumpLessThanOrEqualToU64 {
                address,
                left,
                right,
            }) => self.jump_comparison_u64(address, left, right, |left, right| left <= right)?,
            Some(Instruction::JumpLessThanOrEqualToF64 {
                address,
                left,
                right,
            }) => self.jump_comparison_f64(
                address,
                left,
                right,
                |left, right| left <= right,
                environment,
                actor,
            )?,
            Some(Instruction::JumpGreaterThanU64 {
                address,
                left,
                right,
            }) => self.jump_comparison_u64(address, left, right, |left, right| left > right)?,
            Some(Instruction::JumpGreaterThanF64 {
                address,
                left,
                right,
            }) => self.jump_comparison_f64(
                address,
                left,
                right,
                |left, right| left > right,
                environment,
                actor,
            )?,
            Some(Instruction::JumpGreaterThanOrEqualToU64 {
                address,
                left,
                right,
            }) => self.jump_comparison_u64(address, left, right, |left, right| left >= right)?,
            Some(Instruction::JumpGreaterThanOrEqualToF64 {
                address,
                left,
                right,
            }) => self.jump_comparison_f64(
                address,
                left,
                right,
                |left, right| left >= right,
                environment,
                actor,
            )?,
            Some(Instruction::ShiftLeft {
                destination,
                source,
            }) => self.unary_operator_common_u64(destination, source, |source| source << 1)?,
            Some(Instruction::ShiftRight {
                destination,
                source,
            }) => self.unary_operator_common_u64(destination, source, |source| source >> 1)?,
            Some(Instruction::PushU64 { source }) => {
                let source = self.resolve_source_u64(source);
                self.push_u64(source.value);
                self.clock += source.clock_cost;
            }
            Some(Instruction::PushF64 { source }) => {
                let source = self.resolve_source_f64(source, environment, actor);
                self.push_f64(source.value);
                self.clock += source.clock_cost;
            }
            Some(Instruction::PopU64 { destination }) => {
                let value = self.pop_u64()?;
                self.write_destination_u64(destination, value);
                self.clock += ClockTime(1);
            }
            Some(Instruction::PopF64 { destination }) => {
                let value = self.pop_f64()?;
                self.write_destination_f64(destination, value);
                self.clock += ClockTime(1);
            }
            Some(Instruction::LoadU64 {
                destination,
                source_address,
            }) => {
                let source_address = self.resolve_source_u64(source_address);
                let value = self.load_u64(source_address.value)?;
                self.write_destination_u64(destination, value);
                self.clock += source_address.clock_cost;
            }
            Some(Instruction::LoadF64 {
                destination,
                source_address,
            }) => {
                let source_address = self.resolve_source_u64(source_address);
                let value = self.load_f64(source_address.value)?;
                self.write_destination_f64(destination, value);
                self.clock += source_address.clock_cost;
            }
            Some(Instruction::StoreU64 {
                source,
                destination_address,
            }) => {
                let destination_address = self.resolve_source_u64(destination_address);
                let source = self.resolve_source_u64(source);
                self.store_u64(destination_address.value, source.value)
                    .map_err(|e| {
                        self.halted = true;
                        e
                    })?;
                self.clock += destination_address.clock_cost;
                self.clock += source.clock_cost;
            }
            Some(Instruction::StoreF64 {
                source,
                destination_address,
            }) => {
                let destination_address = self.resolve_source_u64(destination_address);
                let source = self.resolve_source_f64(source, environment, actor);
                self.store_f64(destination_address.value, source.value)
                    .map_err(|e| {
                        self.halted = true;
                        e
                    })?;
                self.clock += destination_address.clock_cost;
                self.clock += source.clock_cost;
            }
            None => Err(StepError::Halted)?,
        };
        Ok(self.clock)
    }

    fn unary_operator_common_u64<F>(
        &mut self,
        destination: DestinationU64,
        source: SourceU64,
        f: F,
    ) -> Result<(), StepError>
    where
        F: FnOnce(u64) -> u64,
    {
        let source = self.resolve_source_u64(source);
        self.write_destination_u64(destination, f(source.value));
        self.clock += source.clock_cost;
        Ok(())
    }

    fn binary_operator_common_u64<F>(
        &mut self,
        destination: DestinationU64,
        left: SourceU64,
        right: SourceU64,
        f: F,
    ) -> Result<(), StepError>
    where
        F: FnOnce(u64, u64) -> u64,
    {
        let left = self.resolve_source_u64(left);
        let right = self.resolve_source_u64(right);
        self.write_destination_u64(destination, f(left.value, right.value));
        self.clock += left.clock_cost + right.clock_cost;
        Ok(())
    }

    fn binary_operator_common_f64<F, ActorData>(
        &mut self,
        destination: DestinationF64,
        left: SourceF64,
        right: SourceF64,
        f: F,
        environment: &physics::Environment<ActorData>,
        actor: &physics::Actor<ActorData>,
    ) -> Result<(), StepError>
    where
        F: FnOnce(f64, f64) -> f64,
        ActorData: Clone,
    {
        let left = self.resolve_source_f64(left, environment, actor);
        let right = self.resolve_source_f64(right, environment, actor);
        self.write_destination_f64(destination, f(left.value, right.value));
        self.clock += left.clock_cost + right.clock_cost;
        Ok(())
    }

    fn jump_comparison_u64<F>(
        &mut self,
        address: SourceU64,
        left: SourceU64,
        right: SourceU64,
        f: F,
    ) -> Result<(), StepError>
    where
        F: FnOnce(u64, u64) -> bool,
    {
        let left = self.resolve_source_u64(left);
        let right = self.resolve_source_u64(right);
        if f(left.value, right.value) {
            let address = self.resolve_source_u64(address);
            self.program_counter = address
                .value
                .try_into()
                .map_err(StepError::TryFromIntError)?;
            self.clock += address.clock_cost;
        }
        self.clock += left.clock_cost + right.clock_cost;
        Ok(())
    }

    fn jump_comparison_f64<F, ActorData>(
        &mut self,
        address: SourceU64,
        left: SourceF64,
        right: SourceF64,
        f: F,
        environment: &physics::Environment<ActorData>,
        actor: &physics::Actor<ActorData>,
    ) -> Result<(), StepError>
    where
        F: FnOnce(f64, f64) -> bool,
        ActorData: Clone,
    {
        let left = self.resolve_source_f64(left, environment, actor);
        let right = self.resolve_source_f64(right, environment, actor);
        if f(left.value, right.value) {
            let address = self.resolve_source_u64(address);
            self.program_counter = address
                .value
                .try_into()
                .map_err(StepError::TryFromIntError)?;
            self.clock += address.clock_cost;
        }
        self.clock += left.clock_cost + right.clock_cost;
        Ok(())
    }

    fn read_next_instruction(&mut self) -> Option<Instruction> {
        self.program.get(self.program_counter).map(|result| {
            self.program_counter.advance();
            result.clone()
        })
    }

    fn resolve_source_u64(&self, source: SourceU64) -> ResolvedValue<u64> {
        match source {
            SourceU64::Register(r) => ResolvedValue {
                value: self.read_register_u64(r),
                clock_cost: ClockTime(2),
            },
            SourceU64::Literal(value) => ResolvedValue {
                value,
                clock_cost: ClockTime(1),
            },
        }
    }

    fn resolve_source_f64<ActorData>(
        &self,
        source: SourceF64,
        environment: &physics::Environment<ActorData>,
        actor: &physics::Actor<ActorData>,
    ) -> ResolvedValue<f64>
    where
        ActorData: Clone,
    {
        match source {
            SourceF64::Register(r) => ResolvedValue {
                value: self.read_register_f64(r, environment, actor),
                clock_cost: ClockTime(4),
            },
            SourceF64::Literal(value) => ResolvedValue {
                value,
                clock_cost: ClockTime(2),
            },
        }
    }

    fn write_destination_u64(&mut self, destination: DestinationU64, value: u64) {
        match destination {
            DestinationU64::Register(r) => self.write_register_u64(r, value),
        }
    }

    fn write_destination_f64(&mut self, destination: DestinationF64, value: f64) {
        match destination {
            DestinationF64::Register(r) => self.write_register_f64(r, value),
        }
    }

    fn read_register_u64(&self, r: ReadableRegisterU64) -> u64 {
        match r {
            ReadableRegisterU64::GeneralPurpose0 => self.register_general_purpose_u64[0],
            ReadableRegisterU64::GeneralPurpose1 => self.register_general_purpose_u64[1],
            ReadableRegisterU64::GeneralPurpose2 => self.register_general_purpose_u64[2],
            ReadableRegisterU64::GeneralPurpose3 => self.register_general_purpose_u64[3],
            ReadableRegisterU64::GeneralPurpose4 => self.register_general_purpose_u64[4],
            ReadableRegisterU64::GeneralPurpose5 => self.register_general_purpose_u64[5],
            ReadableRegisterU64::GeneralPurpose6 => self.register_general_purpose_u64[6],
            ReadableRegisterU64::GeneralPurpose7 => self.register_general_purpose_u64[7],
        }
    }

    fn write_register_u64(&mut self, r: WritableRegisterU64, value: u64) {
        *match r {
            WritableRegisterU64::GeneralPurpose0 => &mut self.register_general_purpose_u64[0],
            WritableRegisterU64::GeneralPurpose1 => &mut self.register_general_purpose_u64[1],
            WritableRegisterU64::GeneralPurpose2 => &mut self.register_general_purpose_u64[2],
            WritableRegisterU64::GeneralPurpose3 => &mut self.register_general_purpose_u64[3],
            WritableRegisterU64::GeneralPurpose4 => &mut self.register_general_purpose_u64[4],
            WritableRegisterU64::GeneralPurpose5 => &mut self.register_general_purpose_u64[5],
            WritableRegisterU64::GeneralPurpose6 => &mut self.register_general_purpose_u64[6],
            WritableRegisterU64::GeneralPurpose7 => &mut self.register_general_purpose_u64[7],
        } = value;
    }

    fn read_register_f64<ActorData>(
        &self,
        r: ReadableRegisterF64,
        environment: &physics::Environment<ActorData>,
        actor: &physics::Actor<ActorData>,
    ) -> f64
    where
        ActorData: Clone,
    {
        match r {
            ReadableRegisterF64::PositionX => self.position.x,
            ReadableRegisterF64::PositionY => self.position.y,
            ReadableRegisterF64::VelocityX => self.velocity.x,
            ReadableRegisterF64::VelocityY => self.velocity.y,
            ReadableRegisterF64::TurretAngle => self.turret_angle.0,
            ReadableRegisterF64::TurretAngularVelocity => self.turrent_angular_velocity.0,
            ReadableRegisterF64::ScannerDistance => environment.actor_scan(actor),
            ReadableRegisterF64::Health => self.health,
            ReadableRegisterF64::Energy => self.energy,
            ReadableRegisterF64::GeneralPurpose0 => self.register_general_purpose_f64[0],
            ReadableRegisterF64::GeneralPurpose1 => self.register_general_purpose_f64[1],
            ReadableRegisterF64::GeneralPurpose2 => self.register_general_purpose_f64[2],
            ReadableRegisterF64::GeneralPurpose3 => self.register_general_purpose_f64[3],
            ReadableRegisterF64::GeneralPurpose4 => self.register_general_purpose_f64[4],
            ReadableRegisterF64::GeneralPurpose5 => self.register_general_purpose_f64[5],
            ReadableRegisterF64::GeneralPurpose6 => self.register_general_purpose_f64[6],
            ReadableRegisterF64::GeneralPurpose7 => self.register_general_purpose_f64[7],
        }
    }

    fn write_register_f64(&mut self, r: WritableRegisterF64, value: f64) {
        *match r {
            WritableRegisterF64::VelocityX => {
                self.velocity.x = value;
                return;
            }
            WritableRegisterF64::VelocityY => {
                self.velocity.y = value;
                return;
            }
            WritableRegisterF64::TurretAngularVelocity => {
                self.turrent_angular_velocity = Radians::from_radians(value);
                return;
            }
            WritableRegisterF64::GeneralPurpose0 => &mut self.register_general_purpose_f64[0],
            WritableRegisterF64::GeneralPurpose1 => &mut self.register_general_purpose_f64[1],
            WritableRegisterF64::GeneralPurpose2 => &mut self.register_general_purpose_f64[2],
            WritableRegisterF64::GeneralPurpose3 => &mut self.register_general_purpose_f64[3],
            WritableRegisterF64::GeneralPurpose4 => &mut self.register_general_purpose_f64[4],
            WritableRegisterF64::GeneralPurpose5 => &mut self.register_general_purpose_f64[5],
            WritableRegisterF64::GeneralPurpose6 => &mut self.register_general_purpose_f64[6],
            WritableRegisterF64::GeneralPurpose7 => &mut self.register_general_purpose_f64[7],
        } = value;
    }

    fn push_u64(&mut self, value: u64) {
        self.stack.push(StackOrHeapValue::U64(value));
    }

    fn push_f64(&mut self, value: f64) {
        self.stack.push(StackOrHeapValue::F64(value));
    }

    fn pop_u64(&mut self) -> Result<u64, StepError> {
        match self.stack.pop() {
            Some(StackOrHeapValue::U64(value)) => Ok(value),
            Some(StackOrHeapValue::F64(value)) => Ok(value as u64),
            None => Err(StepError::StackUnderflow),
        }
    }

    fn pop_f64(&mut self) -> Result<f64, StepError> {
        match self.stack.pop() {
            Some(StackOrHeapValue::F64(value)) => Ok(value),
            Some(StackOrHeapValue::U64(value)) => Ok(value as f64),
            None => Err(StepError::StackUnderflow),
        }
    }

    fn load_u64(&self, address: u64) -> Result<u64, StepError> {
        match self.heap.get(address as usize) {
            Some(StackOrHeapValue::U64(value)) => Ok(*value),
            Some(StackOrHeapValue::F64(value)) => Ok(*value as u64),
            None => Err(StepError::AddressOutOfBounds),
        }
    }

    fn load_f64(&self, address: u64) -> Result<f64, StepError> {
        match self.heap.get(address as usize) {
            Some(StackOrHeapValue::F64(value)) => Ok(*value),
            Some(StackOrHeapValue::U64(value)) => Ok(*value as f64),
            None => Err(StepError::AddressOutOfBounds),
        }
    }

    fn store_u64(&mut self, address: u64, value: u64) -> Result<(), StepError> {
        match self.heap.get_mut(address as usize) {
            Some(ptr) => *ptr = StackOrHeapValue::U64(value),
            None => Err(StepError::AddressOutOfBounds)?,
        }
        Ok(())
    }

    fn store_f64(&mut self, address: u64, value: f64) -> Result<(), StepError> {
        match self.heap.get_mut(address as usize) {
            Some(ptr) => *ptr = StackOrHeapValue::F64(value),
            None => Err(StepError::AddressOutOfBounds)?,
        }
        Ok(())
    }
}
