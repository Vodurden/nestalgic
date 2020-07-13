mod addressing_mode;
mod bus;
mod opcode;
mod instruction;
mod error;
mod register;
mod status;

use addressing_mode::AddressingMode;
use instruction::{Instruction, InstructionArgument};
use opcode::Opcode;
use error::Error;
use register::Register;

pub use bus::Bus;
pub use bus::RamBus16kb;
pub use status::{Status, StatusFlag};

pub type Result<A> = std::result::Result<A, Error>;

pub type Address = u16;

const INITIALIZATION_VECTOR_ADDRESS: u16 = 0xFFFC;

const NMI_VECTOR_ADDRESS: u16 = 0xFFFA;

const STACK_START_ADDRESS: u16 = 0x0100;
const STACK_END_ADDRESS: u16 = 0x01FF;

/// `MOS6502` emulates the functionality of the MOS Technology 6502 microprocessor.
///
/// The NES uses a Ricoh 2A03 which is basically a MOS6502 without the decimal mode.
/// This means this class can be used to emulate the NES.
///
/// This trait functions as an existential type for `MOS6502Cpu` which implements the actual functionality.
/// By using an existential type we can hide the <B: Bus> parameter which is used for nice storage of the `Bus` type.
pub struct MOS6502<B> {
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

    /// The total number of cycles that have elapsed since the CPU started running.
    pub elapsed_cycles: u64,

    /// The amount of cycles to wait for until performing the next instruction.
    pub wait_cycles: u32,

    pub bus: B,
}

#[derive(Eq, PartialEq, Debug)]
enum ReadWriteMode {
    Read,
    Write,
}

impl<B: Bus> MOS6502<B> {
    pub fn new(bus: B) -> MOS6502<B> {
        let mut cpu = MOS6502 {
            a: 0,
            x: 0,
            y: 0,

            p: Status(0),

            pc: 0,
            sp: 0,

            elapsed_cycles: 0,
            wait_cycles: 0,

            bus,
        };

        cpu.reset();
        cpu
    }

    /// When called: Simulates the `reset` input of the 6502.
    pub fn reset(&mut self) {
        // The InterruptDisable bit is set for all interrupts, including `RESET`
        self.p.set(StatusFlag::InterruptDisable, true);

        // Unused should always be true
        self.p.set(StatusFlag::Unused, true);

        // The 6502 performs these operations on reset:
        //
        // - Initialize the Stack Pointer to 0. Pull the instruction register to 0. (3 cycles, sp = 00)
        // - Try to push `pch`, which does nothing because the bus is in read-mode. Decrement `sp` (1 cycle, sp = FF)
        // - Try to push `pcl`, which does nothing again. Decrement `sp` (1 cycle, sp = FE)
        // - Try to push `p`, which does nothing _again_. Decrement `sp` (1 cycle, sp = FD)
        // - Read the low byte of INITIALIZATION_VECTOR_ADDRESS into `pcl` (1 cycle)
        // - Read the high byte of INITIALIZATION_VECTOR_ADDRESS into `pch` (1 cycle)
        //
        // (Source: https://www.pagetable.com/?p=410)
        //
        // We don't want to go through all this nonsense so let's just set `pc` and `sp` to
        // it's final value and wait for 7 cycles.
        let target_address = self.bus.read_u16(INITIALIZATION_VECTOR_ADDRESS);
        self.pc = target_address;

        self.sp = 0xFD;

        self.wait_cycles += 7;
    }

    /// Execute one clock cycle.
    pub fn cycle(&mut self) -> Result<()> {
        if self.wait_cycles == 0 {
            let instruction = self.read_instruction()?;
            self.execute_instruction(instruction)?;
        } else {
            self.wait_cycles -= 1;
        }

        self.elapsed_cycles += 1;
        Ok(())
    }

    /// Cycle the CPU until we hit a BRK (opcode 0).
    ///
    /// This is used for testing
    pub fn cycle_until_brk(&mut self) -> Result<()> {
        loop {
            self.cycle()?;

            if self.next_instruction().map(|i| i.opcode)? == Opcode::BRK {
                return Ok(())
            }
        }
    }

    /// Cycle one instruction plus however many cycles it takes to execute
    /// that instruction.
    ///
    /// This is used for testing.
    pub fn cycle_to_next_instruction(&mut self) -> Result<()> {
        loop {
            self.cycle()?;

            if self.wait_cycles == 0 {
                return Ok(())
            }
        }
    }

    pub fn next_instruction(&self) -> Result<Instruction> {
        let (instruction, _, _) = Instruction::try_from_bus(self.pc, &self.bus)?;
        Ok(instruction)
    }

    fn read_instruction(&mut self) -> Result<Instruction> {
        // We always read an address, even for `implied` and `accumulate` addressing modes
        // to mimic the cycle behavior of the 6502.
        let (instruction, bytes_read, bytes_used) = Instruction::try_from_bus(self.pc, &self.bus)?;
        self.pc += bytes_used;

        // We don't need to wait for the first cycle, we're in it!
        self.wait_cycles += (bytes_read as u32) - 1;
        Ok(instruction)
    }


    fn read_u8(&mut self, address: Address) -> u8 {
        let byte = self.bus.read_u8(address);
        self.wait_cycles += 1;
        byte
    }

    fn read_u16(&mut self, address: Address) -> u16 {
        let byte = self.bus.read_u16(address);
        self.wait_cycles += 2;
        byte
    }

    fn write_u8(&mut self, address: Address, value: u8) {
        self.bus.write_u8(address, value);
        self.wait_cycles += 1;
    }

    fn execute_instruction(&mut self, instruction: Instruction) -> Result<()> {
        match instruction.opcode {
            // Register Operations
            Opcode::LDA => self.op_load(Register::A, instruction),
            Opcode::LDX => self.op_load(Register::X, instruction),
            Opcode::LDY => self.op_load(Register::Y, instruction),
            Opcode::STA => self.op_store(Register::A, instruction),
            Opcode::STX => self.op_store(Register::X, instruction),
            Opcode::STY => self.op_store(Register::Y, instruction),
            Opcode::TAX => self.op_transfer(Register::A, Register::X),
            Opcode::TAY => self.op_transfer(Register::A, Register::Y),
            Opcode::TXA => self.op_transfer(Register::X, Register::A),
            Opcode::TYA => self.op_transfer(Register::Y, Register::A),

            // Stack Operations
            Opcode::TSX => self.op_transfer(Register::SP, Register::X),
            Opcode::TXS => self.op_transfer(Register::X, Register::SP),
            Opcode::PHA => self.op_push_stack(Register::A),
            Opcode::PHP => self.op_push_stack(Register::P),
            Opcode::PLA => self.op_pull_stack(Register::A),
            Opcode::PLP => self.op_pull_stack(Register::P),

            // Logical Operations
            Opcode::AND => self.op_logical(instruction, |a, b| a & b),
            Opcode::EOR => self.op_logical(instruction, |a, b| a ^ b),
            Opcode::ORA => self.op_logical(instruction, |a, b| a | b),
            Opcode::BIT => self.op_bit(instruction),

            // Arithmetic
            Opcode::ADC => self.op_add(instruction),
            Opcode::SBC => self.op_sub(instruction),
            Opcode::CMP => self.op_compare(Register::A, instruction),
            Opcode::CPX => self.op_compare(Register::X, instruction),
            Opcode::CPY => self.op_compare(Register::Y, instruction),

            // Increments & Decrements
            Opcode::INC => self.try_modify_instruction_value(instruction, |v| v.wrapping_add(1)).map(|_| ()),
            Opcode::INX => Ok(self.modify_register(Register::X, |x| x.wrapping_add(1))),
            Opcode::INY => Ok(self.modify_register(Register::Y, |y| y.wrapping_add(1))),
            Opcode::DEC => self.try_modify_instruction_value(instruction, |v| v.wrapping_sub(1)).map(|_| ()),
            Opcode::DEX => Ok(self.modify_register(Register::X, |x| x.wrapping_sub(1))),
            Opcode::DEY => Ok(self.modify_register(Register::Y, |y| y.wrapping_sub(1))),

            // Shifts
            Opcode::ASL => self.op_shift_left(instruction),
            Opcode::LSR => self.op_shift_right(instruction),
            Opcode::ROR => self.op_rotate_right(instruction),
            Opcode::ROL => self.op_rotate_left(instruction),

            // Jumps & Calls
            Opcode::JMP => self.op_jump(instruction),
            Opcode::JSR => self.op_jump_subroutine(instruction),
            Opcode::RTS => self.op_return(),

            // Branches
            Opcode::BCS => self.op_branch_if(instruction, self.p.get(StatusFlag::Carry)),
            Opcode::BCC => self.op_branch_if(instruction, !self.p.get(StatusFlag::Carry)),
            Opcode::BEQ => self.op_branch_if(instruction, self.p.get(StatusFlag::Zero)),
            Opcode::BNE => self.op_branch_if(instruction, !self.p.get(StatusFlag::Zero)),
            Opcode::BMI => self.op_branch_if(instruction, self.p.get(StatusFlag::Negative)),
            Opcode::BPL => self.op_branch_if(instruction, !self.p.get(StatusFlag::Negative)),
            Opcode::BVS => self.op_branch_if(instruction, self.p.get(StatusFlag::Overflow)),
            Opcode::BVC => self.op_branch_if(instruction, !self.p.get(StatusFlag::Overflow)),

            // Status Flag Functions
            Opcode::CLC => Ok(self.p.set(StatusFlag::Carry, false)),
            Opcode::CLD => Ok(self.p.set(StatusFlag::DecimalMode, false)),
            Opcode::CLI => Ok(self.p.set(StatusFlag::InterruptDisable, false)),
            Opcode::CLV => Ok(self.p.set(StatusFlag::Overflow, false)),
            Opcode::SEC => Ok(self.p.set(StatusFlag::Carry, true)),
            Opcode::SED => Ok(self.p.set(StatusFlag::DecimalMode, true)),
            Opcode::SEI => Ok(self.p.set(StatusFlag::InterruptDisable, true)),

            // System Functions
            Opcode::NOP => Ok(()),
            Opcode::RTI => self.op_return_from_interrupt(),

            opcode => panic!("Opcode {} not yet implemented", opcode),
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

    fn modify_register(&mut self, register: Register, f: fn(u8) -> u8) {
        let value = self.read_register(register);
        let result = f(value);
        self.write_register(register, result);
    }

    fn push_stack(&mut self, values: &[u8]) {
        for &value in values {
            self.write_u8(STACK_START_ADDRESS + self.sp as u16, value);
            self.sp = self.sp.wrapping_sub(1);
        }
    }

    fn pull_stack(&mut self, n: u32) -> Vec<u8> {
        // Incrementing the stack pointer costs a cycle on the 6502
        self.sp = self.sp.wrapping_add(1);
        self.wait_cycles += 1;

        let mut vec = Vec::new();
        for _ in 0..n-1 {
            vec.push(self.read_u8(STACK_START_ADDRESS + self.sp as u16));
            self.sp = self.sp.wrapping_add(1);
        }

        // The last read doesn't do `wrapping_add`
        vec.push(self.read_u8(STACK_START_ADDRESS + self.sp as u16));

        vec
    }

    fn push_stack_u8(&mut self, value: u8) {
        self.push_stack(&[value]);
    }

    fn pull_stack_u8(&mut self) -> u8{
        match self.pull_stack(1)[..] {
            [byte] => byte,
            _ => panic!("self.pull_stack(1) returned unexpected number of elements")
        }
    }

    fn push_stack_u16(&mut self, value: u16) {
        let [lo, hi] = value.to_le_bytes();

        // When pushing addresses to the stack we push the `hi` bit first
        self.push_stack(&[hi, lo]);
    }

    fn pull_stack_u16(&mut self) -> u16 {
        match self.pull_stack(2)[..] {
            [lo, hi] => u16::from_le_bytes([lo, hi]),
            _ => panic!("self.pull_stack(1) returned unexpected number of elements")
        }
    }

    pub fn print_stack(&self) {
        let start = STACK_START_ADDRESS + self.sp as u16;
        let end = STACK_END_ADDRESS;
        let bytes = self.bus.read_range(start, end + 1);
        println!("Stack (0x{:04X}..0x{:04X}): {:X?}", start, end, bytes);
    }

    fn try_access_instruction_target_address(&mut self, instruction: Instruction, read_write_mode: ReadWriteMode) -> Result<u16> {
        match instruction.argument {
            InstructionArgument::ZeroPage(address) => Ok(address as u16),

            InstructionArgument::ZeroPageX(address) => {
                Ok(address.wrapping_add(self.x) as u16)
            },

            InstructionArgument::ZeroPageY(address) => {
                Ok(address.wrapping_add(self.y) as u16)
            },

            InstructionArgument::Relative(offset) => {
                // TODO: +2 cycles if page boundary crossed

                Ok(self.pc.wrapping_add(offset as u16))
            }

            InstructionArgument::IndexedIndirect(indexed_address) => {
                // Adding `x` to the address costs 1 cycle on the 6502
                let target_address_lo = indexed_address.wrapping_add(self.x);
                self.wait_cycles += 1;
                let target_lo = self.read_u8(target_address_lo as u16);

                // Incrementing `target_address_lo` by one is done as part of the read cycle so it
                // doesn't cost an extra cycle
                let target_address_hi = target_address_lo.wrapping_add(1);
                let target_hi = self.read_u8(target_address_hi as u16);

                // We don't use `self.read_u16` here because we need each part of
                // the 8-bit address to wrap around on the zero page, rather then
                // the whole address space
                let target_address = u16::from_le_bytes([target_lo, target_hi]);

                Ok(target_address)
            },

            InstructionArgument::IndirectIndexed(indexed_address) => {
                let target_address_lo = indexed_address;
                let target_lo = self.read_u8(target_address_lo as u16);

                let target_address_hi = indexed_address.wrapping_add(1);
                let target_hi = self.read_u8(target_address_hi as u16);

                // We don't use `self.read_u16` here because we need each part of
                // the 8-bit address to wrap around on the zero page, rather then
                // the whole address space
                let target_address = u16::from_le_bytes([target_lo, target_hi]);

                // +1 cycle if page boundary is crossed or if we are performing this
                // read as part of a write
                let (_, page_boundary_crossed) = target_lo.overflowing_add(self.y);
                if page_boundary_crossed || read_write_mode == ReadWriteMode::Write {
                    self.wait_cycles += 1;
                }

                let adjusted_address = target_address.wrapping_add(self.y as u16);

                // let target_address = target_address.wrapping_add(self.y as u16);
                Ok(adjusted_address)
            },

            InstructionArgument::Indirect(target_address) => {
                let address_lo = self.read_u8(target_address);

                // This is a bug in the original 6502 that we need to emulate: If our address
                // spans two pages then the least signifiant byte (the "hi" byte) wraps around
                // and is fetched from the same page. It's known as the "JMP $xxFF" bug.
                //
                // For example: `JMP $02FF` will fetch byte `$02FF` as the low byte and `$0200` as
                // the high byte, instead of `$02FF` and `$0300` as we would normally expect.
                let [target_address_lo, target_address_hi] = target_address.to_le_bytes();
                let target_address_lo = target_address_lo.wrapping_add(1);
                let target_address_plus_one_with_bug = u16::from_le_bytes([target_address_lo, target_address_hi]);
                let address_hi = self.read_u8(target_address_plus_one_with_bug);

                let address = u16::from_le_bytes([address_lo, address_hi]);

                Ok(address)
            },

            InstructionArgument::Absolute(address) => Ok(address),

            InstructionArgument::AbsoluteX(address) => {
                // +1 cycle if page boundary crossed or if we are writing
                let (_, page_boundary_crossed) = (address as u8).overflowing_add(self.x);
                    if page_boundary_crossed || read_write_mode == ReadWriteMode::Write {
                    self.wait_cycles += 1;
                }

                Ok(address.wrapping_add(self.x as u16))
            },

            InstructionArgument::AbsoluteY(address) => {
                // +1 cycle if page boundary crossed or if we are writing
                let (_, page_boundary_crossed) = (address as u8).overflowing_add(self.y);
                if page_boundary_crossed || read_write_mode == ReadWriteMode::Write {
                    self.wait_cycles += 1;
                }

                Ok(address.wrapping_add(self.y as u16))
            },

            _ => Err(Error::InvalidReadAddress(instruction)),
        }
    }

    fn try_read_instruction_target_address(&mut self, instruction: Instruction) -> Result<u16> {
        self.try_access_instruction_target_address(instruction, ReadWriteMode::Read)
    }

    /// Attempt to read the u8 value targeted by this instruction.
    ///
    /// If the `InstructionArgument` of the `Instruction` is `Implied` this function will fail.
    fn try_read_instruction_value(&mut self, instruction: Instruction) -> Result<u8> {
        // `Implied` instructions don't have an argument so this method should never be called
        // with an implied instruction.
        match instruction.argument {
            InstructionArgument::Implied => Err(Error::InvalidReadValue(instruction)),
            InstructionArgument::Accumulator => Ok(self.a),
            InstructionArgument::Immediate(value) => Ok(value),

            _ => {
                let address = self.try_read_instruction_target_address(instruction)?;
                let value = self.read_u8(address);
                Ok(value)
            }
        }
    }

    fn try_write_instruction_value(&mut self, instruction: Instruction, value: u8) -> Result<()> {
        match instruction.argument {
            InstructionArgument::Implied => Err(Error::InvalidWriteValue(instruction)),
            InstructionArgument::Immediate(_) => Err(Error::InvalidWriteValue(instruction)),

            InstructionArgument::Accumulator => {
                self.write_register(Register::A, value);
                Ok(())
            },

            _ => {
                let address = self.try_access_instruction_target_address(instruction, ReadWriteMode::Write)?;
                self.write_u8(address, value);

                Ok(())
            }
        }
    }

    fn try_modify_instruction_value<F>(&mut self, instruction: Instruction, f: F) -> Result<(u8, u8)>
        where F: FnOnce(u8) -> u8
    {
        let value = self.try_read_instruction_value(instruction)?;
        let result = f(value);

        // For non-accumulator modify instructions the 6502 writes the result twice, incurring an extra cycle cost
        if instruction.addressing_mode != AddressingMode::Accumulator {
            self.wait_cycles += 1;
        }

        self.try_write_instruction_value(instruction, result)?;

        // When doing a `modify` we affect `Zero` and `Negative` even when
        // writing to memory
        self.p.set(StatusFlag::Zero, result == 0);
        self.p.set(StatusFlag::Negative, result & 0b1000_0000 > 0);

        Ok((value, result))
    }

    fn op_load(&mut self, register: Register, instruction: Instruction) -> Result<()> {
        let value = self.try_read_instruction_value(instruction)?;
        self.write_register(register, value);
        Ok(())
    }

    fn op_store(&mut self, register: Register, instruction: Instruction) -> Result<()> {
        let value = self.read_register(register);
        self.try_write_instruction_value(instruction, value)?;
        Ok(())
    }

    /// Copy the contents of `source` into `target`
    fn op_transfer(&mut self, source: Register, target: Register) -> Result<()> {
        let value = self.read_register(source);
        self.write_register(target, value);
        Ok(())
    }

    fn op_push_stack(&mut self, source: Register) -> Result<()> {
        let mut value = self.read_register(source);
        // If we push `P` it sets `Break` to `true` for the value that is pushed to the stack
        if source == Register::P {
            value = Status(value)
                .with(StatusFlag::Break, true)
                .0;
        }

        self.push_stack_u8(value);

        Ok(())
    }

    fn op_pull_stack(&mut self, target: Register) -> Result<()> {
        let value = self.pull_stack_u8();
        self.write_register(target, value);
        Ok(())
    }

    fn op_jump(&mut self, instruction: Instruction) -> Result<()> {
        let address = self.try_read_instruction_target_address(instruction)?;
        self.pc = address;
        Ok(())
    }

    fn op_jump_subroutine(&mut self, instruction: Instruction) -> Result<()> {
        let address = self.try_read_instruction_target_address(instruction)?;

        // Calculating the return_address costs 1 cycle on the 6502
        let return_address = self.pc - 1;
        self.wait_cycles += 1;

        self.push_stack_u16(return_address);

        self.pc = address;
        Ok(())
    }

    fn op_return(&mut self) -> Result<()> {
        let address = self.pull_stack_u16();

        // Calculating the offset address costs 1 cycle on the 6502
        self.pc = address + 1;
        self.wait_cycles += 1;
        Ok(())
    }

    fn op_return_from_interrupt(&mut self) -> Result<()> {
        if let [p, pcl, pch] = self.pull_stack(3)[..] {
            self.write_register(Register::P, p);
            self.pc = u16::from_le_bytes([pcl, pch]);
            Ok(())
        } else {
            panic!("self.pull_stack(3) returned unexpected number of elements");
        }

    }

    fn op_branch_if(&mut self, instruction: Instruction, condition: bool) -> Result<()> {
        let address = self.try_read_instruction_target_address(instruction)?;
        if condition {
            self.pc = address;
            self.wait_cycles += 1;
        }
        Ok(())
    }

    fn op_logical(&mut self, instruction: Instruction, f: fn(u8, u8) -> u8) -> Result<()> {
        let value = self.try_read_instruction_value(instruction)?;
        let result = f(self.a, value);
        self.write_register(Register::A, result);
        Ok(())
    }

    fn op_bit(&mut self, instruction: Instruction) -> Result<()> {
        let value = self.try_read_instruction_value(instruction)?;
        let result = value & self.a;

        self.p.set(StatusFlag::Zero, result == 0);
        self.p.set(StatusFlag::Overflow, value & 0b0100_0000 > 0);
        self.p.set(StatusFlag::Negative, value & 0b1000_0000 > 0);
        Ok(())
    }

    fn op_add(&mut self, instruction: Instruction) -> Result<()> {
        let lhs = self.a;
        let rhs = self.try_read_instruction_value(instruction)?;
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

        self.write_register(Register::A, result);

        Ok(())
    }

    fn op_sub(&mut self, instruction: Instruction) -> Result<()> {
        let lhs = self.a;
        let rhs = self.try_read_instruction_value(instruction)?;
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

        self.write_register(Register::A, result);

        Ok(())
    }

    fn op_compare(&mut self, register: Register, instruction: Instruction) -> Result<()> {
        let register = self.read_register(register);
        let value = self.try_read_instruction_value(instruction)?;
        let result = register.wrapping_sub(value);

        // Compare can be thought of a subtraction that doesn't affect the register. I.e. these
        // flags are the result of (register - value).
        self.p.set(StatusFlag::Carry, register >= value);
        self.p.set(StatusFlag::Zero, result == 0);
        self.p.set(StatusFlag::Negative, result & 0b1000_0000 > 0);
        Ok(())
    }

    fn op_shift_left(&mut self, instruction: Instruction) -> Result<()> {
        let (value, _) = self.try_modify_instruction_value(instruction, |value| value.wrapping_shl(1))?;
        self.p.set(StatusFlag::Carry, value & 0b1000_0000 > 0);

        Ok(())
    }

    fn op_shift_right(&mut self, instruction: Instruction) -> Result<()> {
        let (value, _) = self.try_modify_instruction_value(instruction, |value| value.wrapping_shr(1))?;
        self.p.set(StatusFlag::Carry, value & 0b0000_0001 > 0);

        Ok(())
    }

    fn op_rotate_left(&mut self, instruction: Instruction) -> Result<()> {
        let carry = u8::from(self.p.get(StatusFlag::Carry));
        let (value, _) = self.try_modify_instruction_value(instruction, |value| {
            let result = value.wrapping_shl(1);
            let result = result | carry;
            result
        })?;

        self.p.set(StatusFlag::Carry, value & 0b1000_0000 > 0);

        Ok(())
    }

    fn op_rotate_right(&mut self, instruction: Instruction) -> Result<()> {
        let carry = u8::from(self.p.get(StatusFlag::Carry)) << 7;
        let (value, _) = self.try_modify_instruction_value(instruction, |value| {
            let result = value.wrapping_shr(1);
            let result = result | carry;
            result
        })?;

        self.p.set(StatusFlag::Carry, value & 0b0000_0001 > 0);

        Ok(())
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

        let cpu = MOS6502::new(bus);

        assert_eq!(cpu.pc, 0xFF00);
    }

    #[test]
    pub fn op_load_immediate() {
        let program = vec![
            0xA9, 0xBB,  // LDA #$BB
            0xA2, 0x55,  // LDX #$55
            0xA0, 0x25,  // LDY #$25
        ];
        let bus = RamBus16kb::new().with_program(program);
        let mut cpu = MOS6502::new(bus);
        cpu.cycle_until_brk().unwrap();

        assert_eq!(cpu.a, 0xBB);
        assert_eq!(cpu.x, 0x55);
        assert_eq!(cpu.y, 0x25);
    }

    #[test]
    pub fn op_store_zero_page() {
        let program = vec![
            0xA9, 0xBE,  // LDA #$BB
            0xA2, 0x40,  // LDX #$40
            0xA0, 0xFF,  // LDY #$FF
            0x85, 0x00,  // STA $00
            0x86, 0x01,  // STX $01
            0x84, 0x02,  // STY $02
        ];
        let bus = RamBus16kb::new()
            .with_program(program);
        let mut cpu = MOS6502::new(bus);
        cpu.cycle_until_brk().unwrap();

        assert_eq!(cpu.bus.memory[0x00], 0xBE);
        assert_eq!(cpu.bus.memory[0x01], 0x40);
        assert_eq!(cpu.bus.memory[0x02], 0xFF);
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

        let bus = RamBus16kb::new()
            .with_memory_at(0xF000, main_program)
            .with_memory_at(0x0200, sub_program);
        let mut cpu = MOS6502::new(bus);
        println!("{:X?}", Vec::from(&cpu.bus.memory[0x0200..0x0205]));

        // Pretend we already ran the reset sequence
        cpu.pc = 0xF000; // We've put the program in 0xF000 so we can see the high byte on the stack
        cpu.wait_cycles = 0;

        // Stage 1 checks
        cpu.cycle_to_next_instruction().unwrap(); // LDX #$FF
        cpu.cycle_to_next_instruction().unwrap(); // TXS
        cpu.cycle_to_next_instruction().unwrap(); // LDA #$BB
        assert_eq!(cpu.a, 0xBB);
        assert_eq!(cpu.sp, 0xFF);

        // Stage 2 checks: We expect the stack to contain [0xF0, 0x07]. We expect 0x07 instead of 0x08 because `JSR` pushes
        // the current address _minus 1_ to the stack.
        assert_eq!(cpu.pc, 0xF005);
        cpu.cycle_to_next_instruction().unwrap(); // JSR $0200
        assert_eq!(cpu.pc, 0x0200);
        assert_eq!(cpu.bus.memory[0x01FF], 0xF0, "found {:X} at SP 0xFF, expected {:X}", cpu.bus.memory[0x01FF], 0xF0);
        assert_eq!(cpu.bus.memory[0x01FE], 0x07, "found {:X} at SP 0xFE, expected {:X}", cpu.bus.memory[0x01FE], 0x07);

        // Stage 3 checks: We expect to jump back to `0xF008` because `RTS` adds 1 to the address retrieved from the stack
        println!("{:X}: {:X?}", cpu.pc, Vec::from(&cpu.bus.memory[0x0200..0x0205]));
        cpu.cycle_to_next_instruction().unwrap(); // LDA #$FF
        cpu.cycle_to_next_instruction().unwrap(); // RTS
        assert_eq!(cpu.a, 0xFF);
        assert_eq!(cpu.pc, 0xF008);

        // Stage 4 checks
        cpu.cycle_to_next_instruction().unwrap(); // LDX #$BE
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

        let bus = RamBus16kb::new()
            .with_program(program);
        let mut cpu = MOS6502::new(bus);

        // Cycle the reset instructions
        cpu.cycle_to_next_instruction().unwrap();

        // Stage 1 checks
        cpu.cycle_to_next_instruction().unwrap();
        cpu.cycle_to_next_instruction().unwrap();
        assert_eq!(cpu.sp, 0xFF);

        // Stage 2 checks
        cpu.cycle_to_next_instruction().unwrap();
        cpu.cycle_to_next_instruction().unwrap();
        assert_eq!(cpu.sp, 0xFE);
        assert_eq!(cpu.bus.memory[0x01FF], 0xE0);

        // Stage 3 checks
        cpu.cycle_to_next_instruction().unwrap();
        cpu.cycle_to_next_instruction().unwrap();
        assert_eq!(cpu.sp, 0xFD);
        assert_eq!(cpu.bus.memory[0x01FE], 0xBB);

        // Stage 4 checks
        cpu.cycle_to_next_instruction().unwrap();
        cpu.cycle_to_next_instruction().unwrap();
        assert_eq!(cpu.sp, 0xFC);
        assert_eq!(cpu.bus.memory[0x01FD], 0xFF);

        // Stage 5 checks
        cpu.cycle_to_next_instruction().unwrap();
        cpu.cycle_to_next_instruction().unwrap();
        assert_eq!(cpu.sp, 0xFD);
        assert_eq!(cpu.a, 0xFF);

        // Stage 6 checks
        cpu.cycle_to_next_instruction().unwrap();
        assert_eq!(cpu.sp, 0xFE);
        assert_eq!(cpu.a, 0xBB);

        // Stage 7 checks
        cpu.cycle_to_next_instruction().unwrap();
        assert_eq!(cpu.sp, 0xFF);
        assert_eq!(cpu.a, 0xE0);
    }
}
