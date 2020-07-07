mod addressing_mode;
mod bus;
mod opcode;
mod instruction;
mod error;
mod register;
mod status;

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

    fn write_u16(&mut self, address: Address, value: u16) {
        self.bus.write_u16(address, value);
        self.wait_cycles += 2;
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

            // Jumps & Calls
            Opcode::JMP => self.op_jump(instruction),
            Opcode::JSR => self.op_jump_subroutine(instruction),

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

            _ => todo!(),
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

        self.p.set(StatusFlag::Zero, value == 0);
        self.p.set(StatusFlag::Negative, value & 0b1000_0000 > 0);
    }

    fn push_stack_u8(&mut self, value: u8) {
        self.write_u8(self.sp as u16, value);
        self.sp -= 1;
    }

    fn pull_stack_u8(&mut self) -> u8 {
        let value = self.read_u8(self.sp as u16);
        self.sp += 1;
        value
    }

    fn push_stack_u16(&mut self, value: u16) {
        self.write_u16(self.sp as u16, value);
        self.sp -= 2;
    }

    fn pull_stack_u16(&mut self) -> u16 {
        let value = self.read_u16(self.sp as u16);
        self.sp += 2;
        value
    }

    fn try_read_instruction_target_address(&mut self, instruction: Instruction) -> Result<u16> {
        match instruction.argument {
            InstructionArgument::ZeroPage(address) => Ok(address as u16),
            InstructionArgument::ZeroPageX(address) => Ok(address.wrapping_add(self.x) as u16),
            InstructionArgument::ZeroPageY(address) => Ok(address.wrapping_add(self.y) as u16),

            InstructionArgument::Relative(offset) => {
                // TODO: +2 cycles if page boundary crossed

                Ok(self.pc.wrapping_add(offset as u16))
            }

            InstructionArgument::IndexedIndirect(indexed_address) => {
                let indexed_address = indexed_address.wrapping_add(self.x);
                let address = self.read_u16(indexed_address as u16);
                Ok(address)
            },

            InstructionArgument::IndirectIndexed(indexed_address) => {
                let address = self.read_u16(indexed_address as u16);
                let address = address.wrapping_add(self.y as u16);
                // TODO: Add Carry Bit

                // +1 cycle if page boundary is crossed
                if (indexed_address as u16 & 0xFF) + (self.y as u16) > 0xFF {
                    self.wait_cycles += 1;
                }

                Ok(address)
            },

            InstructionArgument::Indirect(address_address) => Ok(self.read_u16(address_address)),
            InstructionArgument::Absolute(address) => Ok(address),

            InstructionArgument::AbsoluteX(address) => {
                // +1 cycle if page boundary crossed
                if (address & 0xFF) + (self.y as u16) > 0xFF {
                    self.wait_cycles += 1;
                }

                Ok(address.wrapping_add(self.x as u16))
            },

            InstructionArgument::AbsoluteY(address) => {
                // +1 cycle if page boundary crossed
                if (address & 0xFF) + (self.y as u16) > 0xFF {
                    self.wait_cycles += 1;
                }

                Ok(address.wrapping_add(self.y as u16))
            },

            _ => Err(Error::InvalidReadAddress(instruction)),
        }
    }

    /// Attempt to read the u8 value targeted by this instruction.
    ///
    /// If the `InstructionArgument` of the `Instruction` is `Implied` this function will fail.
    fn try_read_instruction_value(&mut self, instruction: Instruction) -> Result<u8> {
        // `Implied` instructions don't have an argument so this method should never be called
        // with an implied instruction.
        match instruction.argument {
            InstructionArgument::Implied => Err(Error::ImpliedReadValue(instruction)),
            InstructionArgument::Accumulator => Ok(self.a),
            InstructionArgument::Immediate(value) => Ok(value),

            _ => {
                let address = self.try_read_instruction_target_address(instruction)?;
                let value = self.read_u8(address);
                Ok(value)
            }
        }
    }

    fn op_load(&mut self, register: Register, instruction: Instruction) -> Result<()> {
        let value = self.try_read_instruction_value(instruction)?;
        self.write_register(register, value);
        Ok(())
    }

    fn op_store(&mut self, register: Register, instruction: Instruction) -> Result<()> {
        let value = self.read_register(register);
        let address = self.try_read_instruction_target_address(instruction)?;
        self.write_u8(address, value);
        Ok(())
    }

    /// Copy the contents of `source` into `target`
    fn op_transfer(&mut self, source: Register, target: Register) -> Result<()> {
        let value = self.read_register(source);
        self.write_register(target, value);
        Ok(())
    }

    fn op_push_stack(&mut self, source: Register) -> Result<()> {
        let value = self.read_register(source);
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

    fn op_branch_if(&mut self, instruction: Instruction, condition: bool) -> Result<()> {
        let address = self.try_read_instruction_target_address(instruction)?;
        if condition {
            self.pc = address;
            self.wait_cycles += 1;
        }
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
}
