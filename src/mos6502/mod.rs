mod addressing_mode;
mod bus;
mod opcode;
mod instruction;
mod error;
mod register;

use std::convert::TryFrom;

use addressing_mode::AddressingMode;
use bus::{Bus, RamBus16kb};
use instruction::Instruction;
use opcode::Opcode;
use error::Error;
use register::Register;

pub type Result<A> = std::result::Result<A, Error>;

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

    /// `p` is the processor status register. Each bit in `p` has a different meaning:
    ///
    /// ```text
    /// +---+---+---+---+---+---+---+---+
    /// | N | V |   | B | D | I | Z | C |
    /// +---+---+---+---+---+---+---+---+
    ///   |   |   |   |   |   |   |   |
    ///   |   |   |   |   |   |   |   \-------- CARRY
    ///   |   |   |   |   |   |   |
    ///   |   |   |   |   |   |   \------------ ZERO RESULT
    ///   |   |   |   |   |   |
    ///   |   |   |   |   |   \---------------- INTERRUPT DISABLE
    ///   |   |   |   |   |
    ///   |   |   |   |   \-------------------- DECIMAL MODE (IGNORED)
    ///   |   |   |   |
    ///   |   |   |   \------------------------ BREAK COMMAND
    ///   |   |   |
    ///   |   |   \---------------------------- EXPANSION
    ///   |   |
    ///   |   \-------------------------------- OVERFLOW
    ///   |
    ///   \------------------------------------ NEGATIVE RESULT
    /// ```
    ///
    /// Flag descriptions:
    ///
    /// - `C` is the carry flag which is modified by specific arithmetic operations. It's used as the "ninth bit" for many arithmetic operations.
    /// - `Z` is automatically set during any movement or calculation hen the 8 bits of the resulting operation are 0.
    /// - `I` is the interrupt disable flag. When set it disables the effect of the interrupt request pin.
    /// - `D` is the decimal mode flag. This flag is ignored on the NES. On the 6502 is makes add/subtract operations work on the decimal representation of numbers
    /// - `B` is only set by the processor and is used to determine if an interrupt was caused by the `BRK` command or a real interrupt.
    /// - ` ` is the expansion bit. It's unused.
    /// - `V` is set when addition/subtraction overflows.
    /// - `N` is set after all data movements or arithmetic. If the resultant value is negative this bit will be set to `1`.
    pub p: u8,

    /// `pc` is the program counter. It points to the current executing address in `memory`.
    ///
    /// `pch` refers to the upper 8 bits of this field while `pcl` refers to the lower 8 bits.
    pub pc: u16,

    /// `sp` is the stack pointer. It points to the top of the 256 byte call stack in memory.
    ///
    /// Pushing to the stack decrements the stack pointer. Pulling causes it to be incremented.
    ///
    /// The stack _must_ be located between `0x0100` and `0x01FF` of the addressable memory.
    pub sp: u8,

    bus: B,
}

impl<B: Bus> MOS6502<B> {
    pub fn new(bus: B) -> MOS6502<B> {
        let mut cpu = MOS6502 {
            a: 0,
            x: 0,
            y: 0,

            p: 0,

            pc: 0,
            sp: 0,

            bus
        };

        cpu.reset();
        cpu
    }

    /// When called: Simulates the `reset` input of the 6502.
    pub fn reset(&mut self) {
        // On reset we set the program counter to whatever address is stored at this location
        let reset_address = 0xFFFC;
        let target_address = self.bus.read_u16(INITIALIZATION_VECTOR_ADDRESS);
        self.pc = target_address;
    }

    /// Execute one clock cycle.
    pub fn cycle(&mut self) -> Result<()> {
        println!("Cycle (pc: {:x})", self.pc);
        let instruction = self.read_instruction()?;
        println!("Running instruction: {:?}", instruction);

        self.execute_instruction(instruction)?;

        Ok(())
    }

    /// Cycle the CPU until we read a BRK (opcode 0).
    ///
    /// This is used for testing
    pub fn cycle_until_brk(&mut self) -> Result<()> {
        loop {
            self.cycle()?;

            let next_instruction = self.peek_instruction();
            if let Ok(Opcode::BRK) = next_instruction.map(|i| i.opcode) {
                return Ok(())
            }
        }
    }

    fn read_instruction(&mut self) -> Result<Instruction> {
        let instruction = self.peek_instruction();
        self.pc += 1;
        instruction
    }

    fn peek_instruction(&self) -> Result<Instruction> {
        let byte = self.bus.read_u8(self.pc);
        Instruction::try_from(byte)
    }

    fn read_next_u8(&mut self) -> u8 {
        let byte = self.bus.read_u8(self.pc);
        self.pc += 1;
        byte
    }

    fn read_next_u16(&mut self) -> u16 {
        let word = self.bus.read_u16(self.pc);
        self.pc += 2;
        word
    }

    fn execute_instruction(&mut self, instruction: Instruction) -> Result<()> {
        match instruction.opcode {
            // Register Operations
            Opcode::LDA => self.op_load(Register::A, instruction.addressing_mode),
            Opcode::LDX => self.op_load(Register::X, instruction.addressing_mode),
            Opcode::LDY => self.op_load(Register::Y, instruction.addressing_mode),
            Opcode::STA => self.op_store(Register::A, instruction.addressing_mode),
            Opcode::STX => self.op_store(Register::X, instruction.addressing_mode),
            Opcode::STY => self.op_store(Register::Y, instruction.addressing_mode),
            Opcode::TAX => self.op_transfer(Register::A, Register::X),
            Opcode::TAY => self.op_transfer(Register::A, Register::Y),
            Opcode::TXA => self.op_transfer(Register::X, Register::A),
            Opcode::TYA => self.op_transfer(Register::Y, Register::A),

            // Stack Operations
            Opcode::TSX => self.op_transfer(Register::SP, Register::X),
            Opcode::TXS => self.op_transfer(Register::X, Register::SP),

            _ => Ok(())
        }
    }

    fn read_register(&self, register: Register) -> u8 {
        match register {
            Register::A => self.a,
            Register::X => self.x,
            Register::Y => self.y,
            Register::P => self.p,
            Register::SP => self.sp,
        }
    }

    /// Write a value to a register and update any status flags if necessary
    fn write_register(&mut self, register: Register, value: u8) {
        let register_ref = match register {
            Register::A => &mut self.a,
            Register::X => &mut self.x,
            Register::Y => &mut self.y,
            Register::P => &mut self.p,
            Register::SP => &mut self.sp,
        };
        // TODO: Update status flags
        *register_ref = value;
    }

    /// Read the address at `self.pc` based on the given addressing mode.
    fn read_instruction_target_address(&mut self, addressing_mode: AddressingMode) -> Result<u16> {
        match addressing_mode {
            AddressingMode::ZeroPage => {
                let address = self.read_next_u8();
                Ok(address as u16)
            },

            AddressingMode::ZeroPageX => {
                let address = self.read_next_u8();
                let address = address.wrapping_add(self.x);
                Ok(address as u16)
            },

            AddressingMode::ZeroPageY => {
                let address = self.read_next_u8();
                let address = address.wrapping_add(self.y);
                Ok(address as u16)
            },

            AddressingMode::Absolute => {
                let address = self.read_next_u16();
                Ok(address)
            },

            AddressingMode::AbsoluteX => {
                let address = self.read_next_u16();
                let address = address.wrapping_add(self.x as u16);
                Ok(address)
            },

            AddressingMode::AbsoluteY => {
                let address = self.read_next_u16();
                let address = address.wrapping_add(self.y as u16);
                Ok(address)
            },

            AddressingMode::Relative => {
                // TODO: Check that we are actually applying the right offsets here
                let address = self.read_next_u8() as i8;
                let address = self.pc.wrapping_add(address as u16);
                Ok(address)
            },

            AddressingMode::Indirect => {
                let address_address = self.read_next_u16();
                let address = self.bus.read_u16(address_address);
                Ok(address)
            },

            AddressingMode::IndirectX => {
                todo!()
            },

            AddressingMode::IndirectY => {
                todo!()
            },

            invalid_mode => Err(Error::InvalidAddressRead(invalid_mode))
        }
    }

    fn read_instruction_argument(&mut self, addressing_mode: AddressingMode) -> Result<u8> {
        match addressing_mode {
            AddressingMode::Accumulator => Ok(self.a),
            AddressingMode::Immediate => Ok(self.read_next_u8()),

            memory_addressing_mode => {
                let address = self.read_instruction_target_address(memory_addressing_mode)?;
                let value = self.bus.read_u8(address);
                Ok(value)
            }
        }
    }

    fn op_load(&mut self, register: Register, addressing_mode: AddressingMode) -> Result<()> {
        let value = self.read_instruction_argument(addressing_mode)?;
        self.write_register(register, value);
        Ok(())
    }

    fn op_store(&mut self, register: Register, addressing_mode: AddressingMode) -> Result<()> {
        let value = self.read_register(register);
        let address = self.read_instruction_target_address(addressing_mode)?;
        self.bus.write_u8(address, value);
        Ok(())
    }

    /// Copy the contents of `source` into `target`
    fn op_transfer(&mut self, source: Register, target: Register) -> Result<()> {
        let value = self.read_register(source);
        self.write_register(target, value);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// When the `MOS6502` initializes it should start the program counter
    /// at the address stored in 0xFFFC
    #[test]
    pub fn program_counter_is_initialized_correctly() {
        let mut bus = RamBus16kb::new();
        bus.write_u16(0xFFFC, 0xFF00);

        let mut cpu = MOS6502::new(bus);

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
