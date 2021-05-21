mod addressing_mode;
mod addressable;
mod bus;
mod dma;
mod opcode;
mod instruction;
mod error;
mod register;
mod status;
mod interrupt;

use instruction::Instruction;
use opcode::Opcode;
use error::Error;
use register::Register;
use interrupt::Interrupt;
use std::collections::HashMap;

pub use bus::Bus;
pub use bus::RamBus16kb;
pub use dma::{DMA, ActiveDMA, DMAStatus};
pub use status::{Status, StatusFlag};
pub use interrupt::{NMI_VECTOR_ADDRESS, IRQ_VECTOR_ADDRESS, RESET_VECTOR_ADDRESS};

pub type Result<A> = std::result::Result<A, Error>;

pub type Address = u16;
pub type BytesUsed = u16;
pub type CyclesTaken = u32;

const STACK_START_ADDRESS: u16 = 0x0100;
// const STACK_END_ADDRESS: u16 = 0x01FF;

/// `MOS6502` emulates the functionality of the MOS Technology 6502 microprocessor.
///
/// The NES uses a Ricoh 2A03 which is basically a MOS6502 without the decimal mode.
/// This means this class can be used to emulate the NES.
#[derive(Debug)]
pub struct MOS6502 {
    /// `a` is the accumulator register. It has many uses including:
    ///
    /// - transferring data from memory to the accumulator
    /// - transferring data from the accumulator to memory
    /// - perform various operations such as AND/OR and test the results of those operations
    /// - temporary storage for some operations such as adding two numbers together
    ///
    /// In essence the accumulator _is the primary storage point for the machine_ and _it is
    /// where intermediate results are usually stored_.
    pub a: u8,

    /// An 8-bit index register. It is mainly used to hold counters or offsets for accessing memory.
    pub x: u8,

    /// An 8-bit index register. It is mainly used to hold counters or offsets for accessing memory.
    pub y: u8,

    pub p: Status,

    /// `pc` is the program counter. It points to the current executing address in `memory`.
    ///
    /// `pch` refers to the upper 8 bits of this field while `pcl` refers to the lower 8 bits.
    pub pc: u16,

    /// `sp` is the stack pointer. It points to the top of the 256 byte call stack in memory.
    ///
    /// The 6502 uses a _descending_ stack which means the stack pointer starts at the end (higher address)
    /// of the array. This means pushing to the stack decrements the stack pointer and pulling increments it.
    ///
    /// The stack _must_ be located between `0x0100` and `0x01FF` of the addressable memory. Which means `sp`
    /// ranges between `00` to `FF`
    pub sp: u8,

    /// `nmi` indicates whether the non maskable interrupt line is active on the CPU.
    ///
    /// When set to true the next cycle will trigger the interrupt behavior
    pub nmi: bool,

    /// `irq` indicates whether the maskable interrupt line is active on the CPU.
    ///
    /// When set to true the next cycle will trigger the interrupt behavior
    pub irq: bool,

    /// The total number of cycles that have elapsed since the CPU started running.
    pub elapsed_cycles: u64,

    /// The amount of cycles to wait for until performing the next instruction.
    pub wait_cycles: u32,

    /// The 6502 doesn't have any direct memory access (DMA) capability by default but it's a common
    /// requirement in embedded systems.
    dma: HashMap<Address, DMA>,

    /// Stores the current state of DMA. `None` if no DMA is happening right now.
    active_dma: Option<ActiveDMA>,
}

impl MOS6502 {
    pub fn new() -> MOS6502 {
        MOS6502 {
            a: 0,
            x: 0,
            y: 0,

            p: Status::default(),

            pc: 0,
            sp: 0,

            nmi: false,
            irq: false,

            elapsed_cycles: 0,
            wait_cycles: 0,

            dma: HashMap::new(),
            active_dma: None,
        }
    }

    /// When called: Simulates the `reset` input of the 6502.
    pub fn reset(&mut self, bus: &mut impl Bus) -> Result<()> {
        self.interrupt(bus, Interrupt::RESET)
    }

    /// Execute one clock cycle.
    pub fn cycle(&mut self, bus: &mut impl Bus) -> Result<()> {
        if self.wait_cycles > 0 {
            self.wait_cycles -= 1;
            self.elapsed_cycles += 1;
            return Ok(())
        }

        let dma_status = self.step_active_dma(bus);
        if dma_status == DMAStatus::Active {
            self.elapsed_cycles += 1;
            return Ok(())
        }

        self.execute_interrupts(bus)?;

        let instruction = self.read_instruction(bus)?;
        self.execute_instruction(bus, instruction)?;

        self.elapsed_cycles += 1;

        Ok(())
    }

    pub fn with_dma(mut self, dma: DMA) -> MOS6502 {
        self.dma.insert(dma.trigger_address, dma);
        self
    }

    pub fn step_active_dma(&mut self, bus: &mut impl Bus) -> DMAStatus {
        if let Some(active_dma) = &mut self.active_dma {
            let source_address = active_dma.start_address + active_dma.bytes_transferred;
            let target_address = active_dma.target_address;
            active_dma.bytes_transferred += 1;

            if active_dma.bytes_transferred >= active_dma.bytes_to_transfer {
                self.active_dma = None;
            }

            // We read straight from the bus since dma ignores the usual
            // CPU timings.
            let byte = bus.read_u8(source_address);
            bus.write_u8(target_address, byte);

            // We only need one additional `wait_cycle` for the write since
            // the read is part of this cycle.
            self.wait_cycles += 1;

            DMAStatus::Active
        } else {
            DMAStatus::Inactive
        }
    }

    /// Cycle the CPU until we hit a BRK (opcode 0).
    ///
    /// This is used for testing
    pub fn cycle_until_brk(&mut self, bus: &mut impl Bus) -> Result<()> {
        loop {
            self.cycle(bus)?;

            if self.next_instruction(bus).map(|i| i.opcode)? == Opcode::BRK {
                return Ok(())
            }
        }
    }

    /// Cycle one instruction plus however many cycles it takes to execute
    /// that instruction.
    ///
    /// This is used for testing.
    pub fn cycle_to_next_instruction(&mut self, bus: &mut impl Bus) -> Result<()> {
        loop {
            self.cycle(bus)?;

            if self.wait_cycles == 0 {
                return Ok(())
            }
        }
    }

    fn execute_interrupts(&mut self, bus: &mut impl Bus) -> Result<()> {
        if self.nmi {
            self.interrupt(bus, Interrupt::NMI)?;
            self.nmi = false;
        } else if self.irq {
            self.interrupt(bus, Interrupt::IRQ)?;
        }

        Ok(())
    }

    /// Simulates maskable and non-maskable interrupts on the 6502
    fn interrupt(&mut self, bus: &mut impl Bus, interrupt: Interrupt) -> Result<()> {
        if interrupt.maskable() && self.p.get(StatusFlag::InterruptDisable) {
            return Ok(())
        }

        self.read_instruction(bus)?;
        self.read_instruction(bus)?;

        // RESET decrements the stack three times but doesn't write the values to the stack.
        if interrupt != Interrupt::RESET {
            self.push_stack_u16(bus, self.pc);
            self.push_stack_u8(bus, self.p.with(StatusFlag::Break, interrupt == Interrupt::BRK).0);
        } else {
            self.sp = self.sp.wrapping_sub(3);
            self.wait_cycles += 3;
        }

        let target_address = bus.read_u16(interrupt.vector_address());
        self.wait_cycles += 2;

        // The InterruptDisable bit is set for all interrupts, including `RESET`
        self.p.set(StatusFlag::InterruptDisable, true);

        self.pc = target_address;

        Ok(())
    }

    pub fn next_instruction(&self, bus: &impl Bus) -> Result<Instruction> {
        let (instruction, _, _) = Instruction::try_from_bus(self.pc, bus)?;
        Ok(instruction)
    }

    fn read_instruction(&mut self, bus: &impl Bus) -> Result<Instruction> {
        // We always read an address, even for `implied` and `accumulate` addressing modes
        // to mimic the cycle behavior of the 6502.
        let (instruction, bytes_read, bytes_used) = Instruction::try_from_bus(self.pc, bus)?;
        self.pc += bytes_used;

        // We don't need to wait for the first cycle, we're in it!
        self.wait_cycles += (bytes_read as u32) - 1;
        Ok(instruction)
    }


    fn read_u8(&mut self, bus: &impl Bus, address: Address) -> u8 {
        let byte = bus.read_u8(address);
        self.wait_cycles += 1;

        byte
    }

    fn write_u8(&mut self, bus: &mut impl Bus, address: Address, value: u8) {
        if let Some(dma) = self.dma.get(&address) {
            self.active_dma = Some(ActiveDMA::from_dma(dma, (value as u16) << 8));

            // Normally writing to the dma port takes 1 cycle. But it costs an extra
            // cycle if the CPU is executing on an odd cycle count.
            //
            // In hardware this is caused by the behavior of the `rdy` pin but we
            // cheat and just add the correct number of wait cycles.
            self.wait_cycles += 1;
            if self.elapsed_cycles % 2 != 0 {
                self.wait_cycles += 1;
            }
        } else {
            bus.write_u8(address, value);
        }

        self.wait_cycles += 1;
    }

    fn execute_instruction(&mut self, bus: &mut impl Bus, instruction: Instruction) -> Result<()> {
        match instruction.opcode {
            // Register Operations
            Opcode::LDA => self.op_load(bus, Register::A, instruction),
            Opcode::LDX => self.op_load(bus, Register::X, instruction),
            Opcode::LDY => self.op_load(bus, Register::Y, instruction),
            Opcode::LAX => self.op_lax(bus, instruction),
            Opcode::STA => self.op_store(bus, Register::A, instruction),
            Opcode::STX => self.op_store(bus, Register::X, instruction),
            Opcode::STY => self.op_store(bus, Register::Y, instruction),
            Opcode::SAX => self.op_sax(bus, instruction),
            Opcode::TAX => self.op_transfer(Register::A, Register::X),
            Opcode::TAY => self.op_transfer(Register::A, Register::Y),
            Opcode::TXA => self.op_transfer(Register::X, Register::A),
            Opcode::TYA => self.op_transfer(Register::Y, Register::A),

            // Stack Operations
            Opcode::TSX => self.op_transfer(Register::SP, Register::X),
            Opcode::TXS => self.op_transfer(Register::X, Register::SP),
            Opcode::PHA => self.op_push_stack(bus, Register::A),
            Opcode::PHP => self.op_push_stack(bus, Register::P),
            Opcode::PLA => self.op_pull_stack(bus, Register::A),
            Opcode::PLP => self.op_pull_stack(bus, Register::P),

            // Logical Operations
            Opcode::AND => self.op_logical(bus, instruction, |a, b| a & b),
            Opcode::EOR => self.op_logical(bus, instruction, |a, b| a ^ b),
            Opcode::ORA => self.op_logical(bus, instruction, |a, b| a | b),
            Opcode::BIT => self.op_bit(bus, instruction),

            // Arithmetic
            Opcode::ADC => self.op_add(bus, instruction),
            Opcode::SBC => self.op_sub(bus, instruction),
            Opcode::CMP => self.op_compare(bus, Register::A, instruction),
            Opcode::CPX => self.op_compare(bus, Register::X, instruction),
            Opcode::CPY => self.op_compare(bus, Register::Y, instruction),

            // Increments & Decrements
            Opcode::INC => self.try_modify_instruction_value(bus, instruction, |v| v.wrapping_add(1)).map(|_| ()),
            Opcode::INX => Ok(self.modify_register(Register::X, |x| x.wrapping_add(1))),
            Opcode::INY => Ok(self.modify_register(Register::Y, |y| y.wrapping_add(1))),
            Opcode::ISC => self.op_increment_subtract(bus, instruction),
            Opcode::DEC => self.try_modify_instruction_value(bus, instruction, |v| v.wrapping_sub(1)).map(|_| ()),
            Opcode::DEX => Ok(self.modify_register(Register::X, |x| x.wrapping_sub(1))),
            Opcode::DEY => Ok(self.modify_register(Register::Y, |y| y.wrapping_sub(1))),
            Opcode::DCP => self.op_decrement_compare(bus, instruction),

            // Shifts
            Opcode::ASL => self.op_shift_left(bus, instruction).map(|_| ()),
            Opcode::LSR => self.op_shift_right(bus, instruction).map(|_| ()),
            Opcode::ROR => self.op_rotate_right(bus, instruction).map(|_| ()),
            Opcode::ROL => self.op_rotate_left(bus, instruction).map(|_| ()),
            Opcode::SLO => self.op_shift_left_then_or(bus, instruction),
            Opcode::SRE => self.op_shift_right_then_xor(bus, instruction),
            Opcode::RLA => self.op_rotate_left_then_and(bus, instruction),
            Opcode::RRA => self.op_rotate_right_then_add(bus, instruction),

            // Jumps & Calls
            Opcode::JMP => self.op_jump(bus, instruction),
            Opcode::JSR => self.op_jump_subroutine(bus, instruction),
            Opcode::RTS => self.op_return(bus),

            // Branches
            Opcode::BCS => self.op_branch_if(bus, instruction, self.p.get(StatusFlag::Carry)),
            Opcode::BCC => self.op_branch_if(bus, instruction, !self.p.get(StatusFlag::Carry)),
            Opcode::BEQ => self.op_branch_if(bus, instruction, self.p.get(StatusFlag::Zero)),
            Opcode::BNE => self.op_branch_if(bus, instruction, !self.p.get(StatusFlag::Zero)),
            Opcode::BMI => self.op_branch_if(bus, instruction, self.p.get(StatusFlag::Negative)),
            Opcode::BPL => self.op_branch_if(bus, instruction, !self.p.get(StatusFlag::Negative)),
            Opcode::BVS => self.op_branch_if(bus, instruction, self.p.get(StatusFlag::Overflow)),
            Opcode::BVC => self.op_branch_if(bus, instruction, !self.p.get(StatusFlag::Overflow)),

            // Status Flag Functions
            Opcode::CLC => Ok(self.p.set(StatusFlag::Carry, false)),
            Opcode::CLD => Ok(self.p.set(StatusFlag::DecimalMode, false)),
            Opcode::CLI => Ok(self.p.set(StatusFlag::InterruptDisable, false)),
            Opcode::CLV => Ok(self.p.set(StatusFlag::Overflow, false)),
            Opcode::SEC => Ok(self.p.set(StatusFlag::Carry, true)),
            Opcode::SED => Ok(self.p.set(StatusFlag::DecimalMode, true)),
            Opcode::SEI => Ok(self.p.set(StatusFlag::InterruptDisable, true)),

            // System Functions
            Opcode::NOP => self.op_nop(bus, instruction),
            Opcode::RTI => self.op_return_from_interrupt(bus),
            Opcode::BRK => self.interrupt(bus, Interrupt::BRK),
        }
    }

    fn read_register(&self, register: Register) -> u8 {
        match register {
            Register::A => self.a,
            Register::X => self.x,
            Register::Y => self.y,
            Register::P => self.p.0,
            Register::SP => self.sp,
        }
    }

    /// Write a value to a register and update any status flags if necessary
    fn write_register(&mut self, register: Register, value: u8) {
        let register_ref = match register {
            Register::A => &mut self.a,
            Register::X => &mut self.x,
            Register::Y => &mut self.y,
            Register::P => &mut self.p.0,
            Register::SP => &mut self.sp,
        };

        *register_ref = value;

        // Writing to `P` and `SP` doesn't trigger the status flag calcilation
        if register != Register::P && register != Register::SP {
            self.p.set(StatusFlag::Zero, value == 0);
            self.p.set(StatusFlag::Negative, value & 0b1000_0000 > 0);
        }

        // `P` doesn't actually have any storage for `Break` and `Unused`. This means
        // if we're writing to `P` `Break` should always be `0` and `Unused` should
        // always be `1`
        if register == Register::P {
            self.p.set(StatusFlag::Break, false);
            self.p.set(StatusFlag::Unused, true);
        }
    }

    fn modify_register(&mut self, register: Register, f: impl FnOnce(u8) -> u8) {
        let value = self.read_register(register);
        let result = f(value);
        self.write_register(register, result);
    }

    fn push_stack(&mut self, bus: &mut impl Bus, values: &[u8]) {
        for &value in values {
            self.write_u8(bus, STACK_START_ADDRESS + self.sp as u16, value);
            self.sp = self.sp.wrapping_sub(1);
        }
    }

    fn pull_stack(&mut self, bus: &impl Bus, n: u32) -> Vec<u8> {
        // Incrementing the stack pointer costs a cycle on the 6502
        self.sp = self.sp.wrapping_add(1);
        self.wait_cycles += 1;

        let mut vec = Vec::new();
        for _ in 0..n-1 {
            vec.push(self.read_u8(bus, STACK_START_ADDRESS + self.sp as u16));
            self.sp = self.sp.wrapping_add(1);
        }

        // The last read doesn't do `wrapping_add`
        vec.push(self.read_u8(bus, STACK_START_ADDRESS + self.sp as u16));

        vec
    }

    fn push_stack_u8(&mut self, bus: &mut impl Bus, value: u8) {
        self.push_stack(bus, &[value]);
    }

    fn pull_stack_u8(&mut self, bus: &impl Bus) -> u8{
        match self.pull_stack(bus, 1)[..] {
            [byte] => byte,
            _ => panic!("self.pull_stack(1) returned unexpected number of elements")
        }
    }

    fn push_stack_u16(&mut self, bus: &mut impl Bus, value: u16) {
        let [lo, hi] = value.to_le_bytes();

        // When pushing addresses to the stack we push the `hi` bit first
        self.push_stack(bus, &[hi, lo]);
    }

    fn pull_stack_u16(&mut self, bus: &impl Bus) -> u16 {
        match self.pull_stack(bus, 2)[..] {
            [lo, hi] => u16::from_le_bytes([lo, hi]),
            _ => panic!("self.pull_stack(1) returned unexpected number of elements")
        }
    }

    fn try_read_instruction_target_address(&mut self, bus: &impl Bus, instruction: Instruction) -> Result<Address> {
        let (addressable, read_addressable_cycles) = instruction.addressing.read_addressable(&self, bus)?;
        self.wait_cycles += read_addressable_cycles;

        let address = addressable.address()?;
        Ok(address)
    }

    fn try_read_instruction_value(&mut self, bus: &impl Bus, instruction: Instruction) -> Result<u8> {
        let (addressable, read_addressable_cycles) = instruction.addressing.read_addressable(&self, bus)?;
        self.wait_cycles += read_addressable_cycles;

        let value = addressable.read(self, bus);

        Ok(value)
    }

    fn try_write_instruction_value(&mut self, bus: &mut impl Bus, instruction: Instruction, value: u8) -> Result<()> {
        let (addressable, read_addressable_cycles) = instruction.addressing.read_addressable(&self, bus)?;
        self.wait_cycles += read_addressable_cycles;

        addressable.try_write(self, bus, value)?;

        Ok(())
    }

    fn try_modify_instruction_value(
        &mut self,
        bus: &mut impl Bus,
        instruction: Instruction,
        f: impl FnOnce(u8) -> u8
    ) -> Result<(u8, u8)> {
        let (addressable, read_addressable_cycles) = instruction.addressing.read_addressable(&self, bus)?;
        self.wait_cycles += read_addressable_cycles;

        let (input, output) = addressable.try_modify(self, bus, f)?;

        Ok((input, output))
    }

    fn op_nop(&mut self, bus: &impl Bus, instruction: Instruction) -> Result<()> {
        // Nop is identical to any other read instruction except it throws away the value
        //
        // We ignore errors with reading during NOP since the "legal" NOP has an implied addressing
        // mode which usually results in an invalid read
        let _ = self.try_read_instruction_value(bus, instruction);

        Ok(())
    }

    fn op_load(&mut self, bus: &impl Bus, register: Register, instruction: Instruction) -> Result<()> {
        let value = self.try_read_instruction_value(bus, instruction)?;
        self.write_register(register, value);
        Ok(())
    }

    /// Special variant of `op_load` that loads into `A` and `X`
    ///
    /// Takes the same amount of time as a single `op_load`
    fn op_lax(&mut self, bus: &impl Bus, instruction: Instruction) -> Result<()> {
        let value = self.try_read_instruction_value(bus, instruction)?;
        self.write_register(Register::A, value);
        self.write_register(Register::X, value);
        Ok(())
    }

    fn op_store(&mut self, bus: &mut impl Bus, register: Register, instruction: Instruction) -> Result<()> {
        let value = self.read_register(register);
        self.try_write_instruction_value(bus, instruction, value)?;
        Ok(())
    }

    /// Special variant of `op_store` that stores `A & X` into the target address
    fn op_sax(&mut self, bus: &mut impl Bus, instruction: Instruction) -> Result<()> {
        let value = self.a & self.x;
        self.try_write_instruction_value(bus, instruction, value)?;
        Ok(())
    }

    /// Copy the contents of `source` into `target`
    fn op_transfer(&mut self, source: Register, target: Register) -> Result<()> {
        let value = self.read_register(source);
        self.write_register(target, value);
        Ok(())
    }

    fn op_push_stack(&mut self, bus: &mut impl Bus, source: Register) -> Result<()> {
        let mut value = self.read_register(source);
        // If we push `P` it sets `Break` to `true` for the value that is pushed to the stack
        if source == Register::P {
            let mut status = Status(value);
            status.set(StatusFlag::Break, true);
            value = status.0
        }

        self.push_stack_u8(bus, value);

        Ok(())
    }

    fn op_pull_stack(&mut self, bus: &impl Bus, target: Register) -> Result<()> {
        let value = self.pull_stack_u8(bus);
        self.write_register(target, value);
        Ok(())
    }

    fn op_jump(&mut self, bus: &impl Bus, instruction: Instruction) -> Result<()> {
        let address = self.try_read_instruction_target_address(bus, instruction)?;
        self.pc = address;
        Ok(())
    }

    fn op_jump_subroutine(&mut self, bus: &mut impl Bus, instruction: Instruction) -> Result<()> {
        let address = self.try_read_instruction_target_address(bus, instruction)?;

        // Calculating the return_address costs 1 cycle on the 6502
        let return_address = self.pc - 1;
        self.wait_cycles += 1;

        self.push_stack_u16(bus, return_address);

        self.pc = address;
        Ok(())
    }

    fn op_return(&mut self, bus: &impl Bus) -> Result<()> {
        let address = self.pull_stack_u16(bus);

        // Calculating the offset address costs 1 cycle on the 6502
        self.pc = address + 1;
        self.wait_cycles += 1;
        Ok(())
    }

    fn op_return_from_interrupt(&mut self, bus: &impl Bus) -> Result<()> {
        if let [p, pcl, pch] = self.pull_stack(bus, 3)[..] {
            self.write_register(Register::P, p);
            self.pc = u16::from_le_bytes([pcl, pch]);
            Ok(())
        } else {
            panic!("self.pull_stack(3) returned unexpected number of elements");
        }
    }

    fn op_branch_if(&mut self, bus: &impl Bus, instruction: Instruction, condition: bool) -> Result<()> {
        let (addressable, read_addressable_cycles) = instruction.addressing.read_addressable(&self, bus)?;
        self.wait_cycles += read_addressable_cycles;

        let address = addressable.address()?;
        if condition {
            self.pc = address;
            self.wait_cycles += 1;

            if addressable.page_boundary_crossed {
                self.wait_cycles += 1;
            }
        }
        Ok(())
    }

    fn op_logical(&mut self, bus: &impl Bus, instruction: Instruction, f: fn(u8, u8) -> u8) -> Result<()> {
        let value = self.try_read_instruction_value(bus, instruction)?;
        let result = f(self.a, value);
        self.write_register(Register::A, result);
        Ok(())
    }

    fn op_bit(&mut self, bus: &impl Bus, instruction: Instruction) -> Result<()> {
        let value = self.try_read_instruction_value(bus, instruction)?;
        let result = value & self.a;

        self.p.set(StatusFlag::Zero, result == 0);
        self.p.set(StatusFlag::Overflow, value & 0b0100_0000 > 0);
        self.p.set(StatusFlag::Negative, value & 0b1000_0000 > 0);
        Ok(())
    }

    fn op_add(&mut self, bus: &impl Bus, instruction: Instruction) -> Result<()> {
        let rhs = self.try_read_instruction_value(bus, instruction)?;
        self.add(Register::A, rhs)
    }

    fn add(&mut self, lhs_register: Register, rhs: u8) -> Result<()> {
        let lhs = self.read_register(lhs_register);
        let carry: u8 = self.p.get(StatusFlag::Carry).into();

        let (result, result_overflow) = self.a.overflowing_add(rhs);
        let (result, carry_overflow) = result.overflowing_add(carry);

        let result_carry = result_overflow || carry_overflow;
        self.p.set(StatusFlag::Carry, result_carry);

        // When adding overflow is true if there's a _signed_ overflow, i.e. if we have:
        // `Positive + Positive = Negative` or `Negative + Negative = Positive`
        //
        // This means if:
        //
        // - `lhs` and `rhs` have the same sign
        // - _and_ `lhs` does not have the same sign as `result_sign`
        //
        // Then we have an overflow!
        let lhs_sign = lhs & 0b1000_0000;
        let rhs_sign = rhs & 0b1000_0000;
        let result_sign = result & 0b1000_0000;
        let overflow = (lhs_sign == rhs_sign) && (lhs_sign != result_sign);
        self.p.set(StatusFlag::Overflow, overflow);

        self.write_register(lhs_register, result);

        Ok(())
    }

    fn op_sub(&mut self, bus: &impl Bus, instruction: Instruction) -> Result<()> {
        let rhs = self.try_read_instruction_value(bus, instruction)?;
        self.subtract(Register::A, rhs)
    }

    /// Increment the addressed memory then subtract the result from `a`
    ///
    /// This is an unofficial opcode
    fn op_increment_subtract(&mut self, bus: &mut impl Bus, instruction: Instruction) -> Result<()> {
        let (_, output) = self.try_modify_instruction_value(bus, instruction, |v| v.wrapping_add(1))?;
        self.subtract(Register::A, output)
    }

    fn subtract(&mut self, lhs_register: Register, rhs: u8) -> Result<()> {
        let lhs = self.read_register(lhs_register);
        let carry: u8 = self.p.get(StatusFlag::Carry).into();

        let (result, result_overflow) = self.a.overflowing_sub(rhs);
        let (result, carry_overflow) = result.overflowing_sub(1 - carry);

        let result_carry = result_overflow || carry_overflow;
        self.p.set(StatusFlag::Carry, !result_carry);

        // For subtraction we know an overflow has occured if:
        //
        // - the signs of `lhs` and `rhs` differ
        // - _and_ the sign of `lhs` and `result` differ
        //
        let lhs_sign = lhs & 0b1000_0000;
        let rhs_sign = rhs & 0b1000_0000;
        let result_sign = result & 0b1000_0000;
        let overflow = (lhs_sign != rhs_sign) && (lhs_sign != result_sign);
        self.p.set(StatusFlag::Overflow, overflow);

        self.write_register(lhs_register, result);

        Ok(())
    }

    fn op_compare(&mut self, bus: &impl Bus, register: Register, instruction: Instruction) -> Result<()> {
        let register = self.read_register(register);
        let value = self.try_read_instruction_value(bus, instruction)?;
        let result = register.wrapping_sub(value);

        // Compare can be thought of a subtraction that doesn't affect the register. I.e. these
        // flags are the result of (register - value).
        self.p.set(StatusFlag::Carry, register >= value);
        self.p.set(StatusFlag::Zero, result == 0);
        self.p.set(StatusFlag::Negative, result & 0b1000_0000 > 0);
        Ok(())
    }

    /// Decrements the addressed memory then compares the result with `a`
    ///
    /// This is an unofficial opcode
    fn op_decrement_compare(&mut self, bus: &mut impl Bus, instruction: Instruction) -> Result<()> {
        let (_, output) = self.try_modify_instruction_value(bus, instruction, |v| v.wrapping_sub(1))?;

        // Compare can be thought of a subtraction that doesn't affect the register. I.e. these
        // flags are the result of (register - value).
        let comparison = self.a.wrapping_sub(output);
        self.p.set(StatusFlag::Carry, self.a >= comparison);
        self.p.set(StatusFlag::Zero, comparison == 0);
        self.p.set(StatusFlag::Negative, comparison & 0b1000_0000 > 0);

        Ok(())
    }

    fn op_shift_left(&mut self, bus: &mut impl Bus, instruction: Instruction) -> Result<u8> {
        let (value, result) = self.try_modify_instruction_value(bus, instruction, |value| value.wrapping_shl(1))?;
        self.p.set(StatusFlag::Carry, value & 0b1000_0000 > 0);

        Ok(result)
    }

    fn op_shift_left_then_or(&mut self, bus: &mut impl Bus, instruction: Instruction) -> Result<()> {
        let result = self.op_shift_left(bus, instruction)?;
        self.modify_register(Register::A, |a| a | result);
        Ok(())
    }

    fn op_shift_right(&mut self, bus: &mut impl Bus, instruction: Instruction) -> Result<u8> {
        let (value, result) = self.try_modify_instruction_value(bus, instruction, |value| value.wrapping_shr(1))?;
        self.p.set(StatusFlag::Carry, value & 0b0000_0001 > 0);

        Ok(result)
    }

    fn op_shift_right_then_xor(&mut self, bus: &mut impl Bus, instruction: Instruction) -> Result<()> {
        let result = self.op_shift_right(bus, instruction)?;
        self.modify_register(Register::A, |a| a ^ result);
        Ok(())
    }

    fn op_rotate_left(&mut self, bus: &mut impl Bus, instruction: Instruction) -> Result<u8> {
        let carry = u8::from(self.p.get(StatusFlag::Carry));
        let (value, result) = self.try_modify_instruction_value(bus, instruction, |value| {
            let result = value.wrapping_shl(1);
            let result = result | carry;
            result
        })?;

        self.p.set(StatusFlag::Carry, value & 0b1000_0000 > 0);

        Ok(result)
    }

    fn op_rotate_right(&mut self, bus: &mut impl Bus, instruction: Instruction) -> Result<u8> {
        let carry = u8::from(self.p.get(StatusFlag::Carry)) << 7;
        let (value, result) = self.try_modify_instruction_value(bus, instruction, |value| {
            let result = value.wrapping_shr(1);
            let result = result | carry;
            result
        })?;

        self.p.set(StatusFlag::Carry, value & 0b0000_0001 > 0);

        Ok(result)
    }

    fn op_rotate_left_then_and(&mut self, bus: &mut impl Bus, instruction: Instruction) -> Result<()> {
        let result = self.op_rotate_left(bus, instruction)?;
        self.modify_register(Register::A, |a| a & result);
        Ok(())
    }

    fn op_rotate_right_then_add(&mut self, bus: &mut impl Bus, instruction: Instruction) -> Result<()> {
        let result = self.op_rotate_right(bus, instruction)?;
        self.add(Register::A, result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::bus::RamBus16kb;

    /// When the `MOS6502` initializes it should start the program counter
    /// at the address stored in 0xFFFC
    #[test]
    pub fn program_counter_is_initialized_correctly() {
        let mut bus = RamBus16kb::new();
        bus.write_u16(0xFFFC, 0xFF00);

        let mut cpu = MOS6502::new();
        cpu.reset(&mut bus).expect("CPU Reset Failed");

        assert_eq!(cpu.pc, 0xFF00);
    }

    #[test]
    pub fn op_load_immediate() {
        let program = vec![
            0xA9, 0xBB,  // LDA #$BB
            0xA2, 0x55,  // LDX #$55
            0xA0, 0x25,  // LDY #$25
        ];
        let mut bus = RamBus16kb::new().with_program(program);

        let mut cpu = MOS6502::new();
        cpu.reset(&mut bus).expect("CPU Reset Failed");
        cpu.cycle_until_brk(&mut bus).unwrap();

        assert_eq!(cpu.a, 0xBB);
        assert_eq!(cpu.x, 0x55);
        assert_eq!(cpu.y, 0x25);
    }

    #[test]
    pub fn op_store_zero_page() {
        let program = vec![
            0xA9, 0xBE,  // LDA #$BE
            0xA2, 0x40,  // LDX #$40
            0xA0, 0xFF,  // LDY #$FF
            0x85, 0x00,  // STA $00
            0x86, 0x01,  // STX $01
            0x84, 0x02,  // STY $02
        ];
        let mut bus = RamBus16kb::new()
            .with_program(program);
        let mut cpu = MOS6502::new();
        cpu.reset(&mut bus).expect("CPU Reset Failed");
        cpu.cycle_until_brk(&mut bus).unwrap();

        assert_eq!(bus.memory[0x00], 0xBE);
        assert_eq!(bus.memory[0x01], 0x40);
        assert_eq!(bus.memory[0x02], 0xFF);
    }

    /// Pushing a 16 bit address on the stack is a bit fiddly. This test is to check that `JSR` and `RTS` have the correct
    /// interactions and that they write exactly the right bytes to the stack _in the right order_.
    #[test]
    pub fn op_jump_subroutine_and_return() {
        let main_program = vec![
            // Stage 1: Reset the stack pointer. Add something to `A` so we know it actually changes over time
            0xA2, 0xFF,        // 0xF000: LDX #$FF
            0x9A,              // 0xF002: TXS
            0xA9, 0xBB,        // 0xF003: LDA #$BB

            // Stage 2: Jump to `0x0200`
            0x20, 0x00, 0x02,  // 0xF005: JSR $0200

            // Stage 4: After returning from jump. Load `0xBE` into `X`, halt program.
            0xA2, 0xBE,        // 0xF008: LDX #$BE
        ];

        let sub_program = vec![
            // Stage 3: Load `0xFF` into `A`, return from jump
            0xA9, 0xFF,       // 0x0200: LDA #$FF
            0x60,             // 0x0202: RTS
        ];

        let mut bus = RamBus16kb::new()
            .with_memory_at(0xF000, main_program)
            .with_memory_at(0x0200, sub_program);
        let mut cpu = MOS6502::new();
        cpu.reset(&mut bus).expect("CPU Reset Failed");
        println!("{:X?}", Vec::from(&bus.memory[0x0200..0x0205]));

        // Pretend we already ran the reset sequence
        cpu.pc = 0xF000; // We've put the program in 0xF000 so we can see the high byte on the stack
        cpu.wait_cycles = 0;

        // Stage 1 checks
        cpu.cycle_to_next_instruction(&mut bus).unwrap(); // LDX #$FF
        cpu.cycle_to_next_instruction(&mut bus).unwrap(); // TXS
        cpu.cycle_to_next_instruction(&mut bus).unwrap(); // LDA #$BB
        assert_eq!(cpu.a, 0xBB);
        assert_eq!(cpu.sp, 0xFF);

        // Stage 2 checks: We expect the stack to contain [0xF0, 0x07]. We expect 0x07 instead of 0x08 because `JSR` pushes
        // the current address _minus 1_ to the stack.
        assert_eq!(cpu.pc, 0xF005);
        cpu.cycle_to_next_instruction(&mut bus).unwrap(); // JSR $0200
        assert_eq!(cpu.pc, 0x0200);
        assert_eq!(bus.memory[0x01FF], 0xF0, "found {:X} at SP 0xFF, expected {:X}", bus.memory[0x01FF], 0xF0);
        assert_eq!(bus.memory[0x01FE], 0x07, "found {:X} at SP 0xFE, expected {:X}", bus.memory[0x01FE], 0x07);

        // Stage 3 checks: We expect to jump back to `0xF008` because `RTS` adds 1 to the address retrieved from the stack
        println!("{:X}: {:X?}", cpu.pc, Vec::from(&bus.memory[0x0200..0x0205]));
        cpu.cycle_to_next_instruction(&mut bus).unwrap(); // LDA #$FF
        cpu.cycle_to_next_instruction(&mut bus).unwrap(); // RTS
        assert_eq!(cpu.a, 0xFF);
        assert_eq!(cpu.pc, 0xF008);

        // Stage 4 checks
        cpu.cycle_to_next_instruction(&mut bus).unwrap(); // LDX #$BE
        assert_eq!(cpu.x, 0xBE);
    }

    #[test]
    pub fn op_push_pop() {
        let program = vec![
            // Set `SP` to `0xFF`         (stage 1)
            0xA2, 0xFF,  // LDX #$FF
            0x9A,        // TXS

            // Push `0xE0` onto the stack (stage 2)
            0xA9, 0xE0,  // LDA #$E0
            0x48,        // PHA

            // Push `0xBB` onto the stack (stage 3)
            0xA9, 0xBB,  // LDA #$BB
            0x48,        // PHA

            // Push `0xFF` onto the stack (stage 4)
            0x8A,        // TXA
            0x48,        // PHA

            // Clear `A`. Pull `0xFF` from the stack (stage 5)
            0xA9, 0x00,  // LDA #$00
            0x68,        // PLA

            // Pull `0xBB` from the stack (stage 6)
            0x68,        // PLA

            // Pull `0xE)` from the stack (stage 6)
            0x68,        // PLA
        ];

        let mut bus = RamBus16kb::new()
            .with_program(program);
        let mut cpu = MOS6502::new();
        cpu.reset(&mut bus).expect("CPU Reset Failed");

        // Cycle the reset instructions
        cpu.cycle_to_next_instruction(&mut bus).unwrap();

        // Stage 1 checks
        cpu.cycle_to_next_instruction(&mut bus).unwrap();
        cpu.cycle_to_next_instruction(&mut bus).unwrap();
        assert_eq!(cpu.sp, 0xFF);

        // Stage 2 checks
        cpu.cycle_to_next_instruction(&mut bus).unwrap();
        cpu.cycle_to_next_instruction(&mut bus).unwrap();
        assert_eq!(cpu.sp, 0xFE);
        assert_eq!(bus.memory[0x01FF], 0xE0);

        // Stage 3 checks
        cpu.cycle_to_next_instruction(&mut bus).unwrap();
        cpu.cycle_to_next_instruction(&mut bus).unwrap();
        assert_eq!(cpu.sp, 0xFD);
        assert_eq!(bus.memory[0x01FE], 0xBB);

        // Stage 4 checks
        cpu.cycle_to_next_instruction(&mut bus).unwrap();
        cpu.cycle_to_next_instruction(&mut bus).unwrap();
        assert_eq!(cpu.sp, 0xFC);
        assert_eq!(bus.memory[0x01FD], 0xFF);

        // Stage 5 checks
        cpu.cycle_to_next_instruction(&mut bus).unwrap();
        cpu.cycle_to_next_instruction(&mut bus).unwrap();
        assert_eq!(cpu.sp, 0xFD);
        assert_eq!(cpu.a, 0xFF);

        // Stage 6 checks
        cpu.cycle_to_next_instruction(&mut bus).unwrap();
        assert_eq!(cpu.sp, 0xFE);
        assert_eq!(cpu.a, 0xBB);

        // Stage 7 checks
        cpu.cycle_to_next_instruction(&mut bus).unwrap();
        assert_eq!(cpu.sp, 0xFF);
        assert_eq!(cpu.a, 0xE0);
    }

    /// When the NES executes a DMA on an even CPU cycle we expect
    #[test]
    pub fn nes_style_ppu_dma_on_odd_cycle() {
        let program = vec![
            // Write 0x02 to 0x4014 to trigger DMA from
            // 0x0200-0x02FF
            0xA2, 0x02,       // LDX #$02
            0x8E, 0x14, 0x40, // STX $4014

            // Do something after the DMA to make sure resuming
            // still works
            0xA9, 0xE0,  // LDA #$E0
        ];

        // We want to see this data be copied from `0x0200` to
        // 0x2004.
        let oam_data: Vec<u8> = (0..=255).collect();
        print!("oam_data: ");
        oam_data.iter().for_each(|x| print!("{} ", x));
        println!("");
        println!("oam_data len: {}", oam_data.len());

        let mut bus = RamBus16kb::new()
            .with_program(program)
            .with_memory_at(0x0200, oam_data.clone());

        print!("bus_data: ");
        bus.memory[0x0200..0x02FF].iter().for_each(|x| print!("{} ", x));
        println!("");

        let nes_dma = DMA {
            trigger_address: 0x4014,
            target_address: 0x2004,
            bytes_to_transfer: 256,
        };

        let mut cpu = MOS6502::new().with_dma(nes_dma);
        cpu.reset(&mut bus).expect("CPU Reset Failed");
        cpu.cycle_to_next_instruction(&mut bus).unwrap();
        println!("cpu: {:?}", cpu);

        // Step 1: Trigger the DMA
        cpu.cycle_to_next_instruction(&mut bus).unwrap();
        println!("cpu: {:?}", cpu);
        cpu.cycle_to_next_instruction(&mut bus).unwrap();
        println!("cpu: {:?}", cpu);

        // - +7 cycles for reset
        // - +2 cycles for immediate LDX
        // - +4 cycles for absolute STX
        // - +2 cycles to start DMA on an odd cycle
        assert_eq!(cpu.elapsed_cycles, 15);

        // Step 2: Make sure each write to `0x2004` is what we expect.
        for byte in oam_data {
            cpu.cycle(&mut bus).unwrap();
            cpu.cycle(&mut bus).unwrap();
            assert_eq!(bus.memory[0x2004], byte);
            println!("cpu: {:?}", cpu);
        }

        // Step 3: Make sure the elapsed time is correct.
        //
        // We expect:
        //
        // - +7 cycles for reset
        // - +2 cycles for immediate LDX
        // - +4 cycles for absolute STX
        // - +2 cycles to start DMA on an odd cycle
        // - +512 cycles for DMA transfer
        assert_eq!(cpu.elapsed_cycles, 514 + 13);

        // Step 4: Make sure we resume instructions correctly after DMA finishes.
        cpu.cycle_to_next_instruction(&mut bus).unwrap();
        assert_eq!(cpu.a, 0xE0);
    }

    /// When the NES executes a DMA on an even CPU cycle we expect
    #[test]
    pub fn nes_style_ppu_dma_on_even_cycle() {
        let program = vec![
            // Write 0x02 to 0x4014 to trigger DMA from
            // 0x0200-0x02FF
            0xA2, 0x02,       // LDX #$02    (+2 cycles)
            0xA4, 0x00,       // LDY $00     (+3 cycles, to make cycle count even)
            0x8E, 0x14, 0x40, // STX $4014   (+4 cycles)
        ];

        let mut bus = RamBus16kb::new()
            .with_program(program);

        let nes_dma = DMA {
            trigger_address: 0x4014,
            target_address: 0x2004,
            bytes_to_transfer: 256,
        };

        let mut cpu = MOS6502::new().with_dma(nes_dma);
        cpu.reset(&mut bus).expect("CPU Reset Failed");
        cpu.cycle_to_next_instruction(&mut bus).unwrap();

        // Step 1: Trigger the DMA
        cpu.cycle_to_next_instruction(&mut bus).unwrap();
        cpu.cycle_to_next_instruction(&mut bus).unwrap();
        cpu.cycle_to_next_instruction(&mut bus).unwrap();

        // - +7 cycles for reset
        // - +2 cycles for immediate LDX
        // - +3 cycles for zero page LDY
        // - +4 cycles for absolute STX
        // - +1 cycles to start DMA on an odd cycle
        assert_eq!(cpu.elapsed_cycles, 17);
    }
}
