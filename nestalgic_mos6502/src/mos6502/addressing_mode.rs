use std::fmt;

use super::{Address, BytesUsed, CyclesTaken, Result};
use super::{MOS6502, Register};
use super::bus::Bus;
use super::error::Error;
use super::status::StatusFlag;

/// `AddressingMode` is combined with `Opcode` to decide _where_ the arguments for an opcode should be sourced from.
///
/// If the `AddressingMode` is `Accumulator`
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum AddressingMode {
    /// An `Opcode` has an `Implied` addressing mode if the target address
    /// is implied by the instruction.
    ///
    /// Example: `INX`
    Implied,

    /// The `Opcode` is targeting the accumulator `A`
    ///
    /// Example: `ROL`
    Accumulator,

    /// The `Opcode` is expecting a value defined inline in assembly.
    ///
    /// Example: `LDA #$AA` which loads `0xAA` into `A`
    Immediate,

    /// `ZeroPage` means we want to load a value referenced by an address stored within the
    /// first 256 bytes of memory (page 0).
    ///
    /// For example, consider the following memory layout:
    ///
    /// ```text
    /// 0x0001: 0xBE
    /// ```
    ///
    /// If I execute `LDA $01` then `A` will contain `0xBE`.
    ZeroPage,

    /// `ZeroPageX` is the same as `ZeroPage` except `X` is added to the zero page address before resolving the value.
    ///
    /// If `$(arg) + X` exceeds `0xFF` the value will wrap-around.
    ///
    /// Example: `LDA $00,X`
    ZeroPageX,

    /// `ZeroPageY` is the same as `ZeroPage` except `Y` is added to the zero page address before resolving the value.
    ///
    /// If `$(arg) + Y` exceeds `0xFF` the value will wrap-around.
    ///
    /// Example: `LDA $00,Y`
    ZeroPageY,

    Relative,  // (s8)
    Indirect,  // u16 -> u16

    /// `IndexedIndirect` means we want to load a value in the Zero Page (first 256 bytes of memory) referenced by
    /// anywhere in memory using an `X` offset
    ///
    /// For example, consider the following memory layout:
    ///
    /// ```text
    /// 0x0000: 0xEF
    /// 0x0001: 0xBE
    /// ...
    /// 0xBEEF: 0xAA
    /// ```
    ///
    /// If I execute LDA
    ///
    /// TODO: Finish this comment
    IndexedIndirect,

    /// This instruction takes `5` cycles (+1 if a page is crossed when adding `y` to the base address)
    IndirectIndexed,

    // 16-bit memory return value
    Absolute,  // u16 -> u8
    AbsoluteX, // (u16, x) -> u8
    AbsoluteY, // (u16, y) -> u8
}


#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Addressing {
    Implied,
    Accumulator,
    Immediate(u8),
    ZeroPage(u8),
    ZeroPageX(u8),
    ZeroPageY(u8),
    Relative(u8),
    IndexedIndirect(u8),
    IndirectIndexed(u8),
    Indirect(Address),
    Absolute(Address),
    AbsoluteX(Address),
    AbsoluteY(Address),
}

/// An `Adderssable` is a fully realized `Addressing` that can be used to read, write and modify registers and memory
/// targeted by an `Addressing`
///
/// An addressing can be used to read, modify and write across all values targetable by `AdressingMode` (Memory, Accumulator, Immediate)
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Addressable {
    Accumulator,
    Immediate(u8),
    Memory(u16),
}

impl fmt::Display for AddressingMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl fmt::Display for Addressing {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl fmt::Display for Addressable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl AddressingMode {
    /// Retrieve an `Addressing` from memory for the given `AddressingMode`.
    ///
    /// If successful, returns the `Addressing`, the number of cycles taken and the number of bytes used
    /// in the construction of the `Addressing`.
    pub fn read_addressing(&self, start: Address, bus: &impl Bus) -> (Addressing, CyclesTaken, BytesUsed) {
        match self {
            AddressingMode::Implied => {
                // The 6502 always reads from the bus even if the `AddressingMode` doesn't actually use the value.
                let _ = bus.read_u8(start);
                (Addressing::Implied, 1, 0)
            }

            AddressingMode::Accumulator => {
                // The 6502 always reads from the bus even if the `AddressingMode` doesn't actually use the value.
                let _ = bus.read_u8(start);
                (Addressing::Accumulator, 1, 0)
            }

            AddressingMode::Immediate => {
                let value = bus.read_u8(start);
                (Addressing::Immediate(value), 1, 1)
            }

            AddressingMode::ZeroPage => {
                let address = bus.read_u8(start);
                (Addressing::ZeroPage(address), 1, 1)
            }

            AddressingMode::ZeroPageX => {
                let address = bus.read_u8(start);
                (Addressing::ZeroPageX(address), 1, 1)
            }

            AddressingMode::ZeroPageY => {
                let address = bus.read_u8(start);
                (Addressing::ZeroPageY(address), 1, 1)
            }

            AddressingMode::Relative => {
                let address = bus.read_u8(start);
                (Addressing::Relative(address), 1, 1)
            }

            AddressingMode::IndexedIndirect => {
                let address = bus.read_u8(start);
                (Addressing::IndexedIndirect(address), 1, 1)
            }

            AddressingMode::IndirectIndexed => {
                let address = bus.read_u8(start);
                (Addressing::IndirectIndexed(address), 1, 1)
            }

            AddressingMode::Indirect => {
                let address = bus.read_u16(start);
                (Addressing::Indirect(address), 2, 2)
            }

            AddressingMode::Absolute => {
                let address = bus.read_u16(start);
                (Addressing::Absolute(address), 2, 2)
            }

            AddressingMode::AbsoluteX => {
                let address = bus.read_u16(start);
                (Addressing::AbsoluteX(address), 2, 2)
            }

            AddressingMode::AbsoluteY => {
                let address = bus.read_u16(start);
                (Addressing::AbsoluteY(address), 2, 2)
            }
        }
    }
}

impl Addressing {
    pub fn target<B: Bus>(&self, cpu: &MOS6502<B>, force_page_boundary_check: bool) -> Result<(Addressable, CyclesTaken)> {
        match self {
            Addressing::Implied => Err(Error::InvalidTargetAddressAttempt(*self)),
            Addressing::Accumulator => Addressing::target_accumulator(),
            Addressing::Immediate(value) => Addressing::target_immediate(*value),
            Addressing::ZeroPage(address) => Addressing::target_zero_page(*address),
            Addressing::ZeroPageX(address) => Addressing::target_zero_page_indexed(cpu, *address, cpu.x),
            Addressing::ZeroPageY(address) => Addressing::target_zero_page_indexed(cpu, *address, cpu.y),
            Addressing::Relative(offset) => Addressing::target_relative(cpu, *offset),
            Addressing::IndexedIndirect(indexed_address) => Addressing::target_indexed_indirect(cpu, *indexed_address),
            Addressing::IndirectIndexed(indexed_address) => Addressing::target_indirect_indexed(cpu, *indexed_address, force_page_boundary_check),
            Addressing::Indirect(target_address) => Addressing::target_indirect(cpu, *target_address),
            Addressing::Absolute(address) => Addressing::target_absolute(*address),
            Addressing::AbsoluteX(base_address) => Addressing::target_absolute_indexed(*base_address, cpu.x, force_page_boundary_check),
            Addressing::AbsoluteY(base_address) => Addressing::target_absolute_indexed(*base_address, cpu.y, force_page_boundary_check),
        }
    }

    fn target_accumulator() -> Result<(Addressable, CyclesTaken)> {
        Ok((Addressable::Accumulator, 0))
    }

    fn target_immediate(value: u8) -> Result<(Addressable, CyclesTaken)> {
        Ok((Addressable::Immediate(value), 0))
    }

    fn target_zero_page(zero_page_address: u8) -> Result<(Addressable, CyclesTaken)> {
        Ok((Addressable::Memory(zero_page_address as u16), 0))
    }

    fn target_zero_page_indexed<B: Bus>(cpu: &MOS6502<B>, address: u8, register: u8) -> Result<(Addressable, CyclesTaken)> {
        // The 6502 does a dummy read on zero page indexed that it throws away. +1 Cycle
        let _ = cpu.bus.read_u8(address as u16);
        let address = address.wrapping_add(register);

        Ok((Addressable::Memory(address as u16), 1))
    }

    fn target_relative<B>(cpu: &MOS6502<B>, offset: u8) -> Result<(Addressable, CyclesTaken)> {
        // TODO: +2 cycles if page boundary crossed
        let target = cpu.pc.wrapping_add(offset as u16);
        Ok((Addressable::Memory(target), 0))
    }

    fn target_indexed_indirect<B: Bus>(cpu: &MOS6502<B>, indexed_address: u8) -> Result<(Addressable, CyclesTaken)> {
        // Adding `x` to the address costs 1 cycle on the 6502.
        let target_address_lo = indexed_address.wrapping_add(cpu.x);
        let mut cycles_taken = 1;
        let target_lo = cpu.bus.read_u8(target_address_lo as u16);
        cycles_taken += 1;

        // Incrementing `target_address_lo` by one is done as part of the read cycle so it
        // doesn't cost an extra cycle
        let target_address_hi = target_address_lo.wrapping_add(1);
        let target_hi = cpu.bus.read_u8(target_address_hi as u16);
        cycles_taken += 1;

        // We don't use `cpu.read_u16` here because we need each part of
        // the 8-bit address to wrap around on the zero page, rather then
        // the whole address space
        let target_address = u16::from_le_bytes([target_lo, target_hi]);

        Ok((Addressable::Memory(target_address), cycles_taken))
    }

    fn target_indirect_indexed<B: Bus>(cpu: &MOS6502<B>, indexed_address: u8, force_page_boundary_check: bool) -> Result<(Addressable, CyclesTaken)> {
        let target_address_lo = indexed_address;
        let target_lo = cpu.bus.read_u8(target_address_lo as u16);
        let mut cycles_taken = 1;

        let target_address_hi = indexed_address.wrapping_add(1);
        let target_hi = cpu.bus.read_u8(target_address_hi as u16);
        cycles_taken += 1;

        // We don't use `cpu.read_u16` here because we need each part of
        // the 8-bit address to wrap around on the zero page, rather then
        // the whole address space
        let target_address = u16::from_le_bytes([target_lo, target_hi]);

        // +1 cycle if page boundary is crossed or if we are forcing a page
        // boundary check (which usually occurs as part of a write)
        let (_, page_boundary_crossed) = target_lo.overflowing_add(cpu.y);
        if page_boundary_crossed || force_page_boundary_check {
            cycles_taken += 1;
        }

        let adjusted_address = target_address.wrapping_add(cpu.y as u16);

        Ok((Addressable::Memory(adjusted_address), cycles_taken))
    }

    fn target_indirect<B: Bus>(cpu: &MOS6502<B>, target_address: Address) -> Result<(Addressable, CyclesTaken)> {
        let address_lo = cpu.bus.read_u8(target_address);
        let mut cycles_taken = 1;

        // This is a bug in the original 6502 that we need to emulate: If our address
        // spans two pages then the least signifiant byte (the "hi" byte) wraps around
        // and is fetched from the same page. It's known as the "JMP $xxFF" bug.
        //
        // For example: `JMP $02FF` will fetch byte `$02FF` as the low byte and `$0200` as
        // the high byte, instead of `$02FF` and `$0300` as we would normally expect.
        let [target_address_lo, target_address_hi] = target_address.to_le_bytes();
        let target_address_lo = target_address_lo.wrapping_add(1);
        let target_address_plus_one_with_bug = u16::from_le_bytes([target_address_lo, target_address_hi]);
        let address_hi = cpu.bus.read_u8(target_address_plus_one_with_bug);
        cycles_taken += 1;

        let address = u16::from_le_bytes([address_lo, address_hi]);

        Ok((Addressable::Memory(address), cycles_taken))
    }

    fn target_absolute(address: u16) -> Result<(Addressable, CyclesTaken)> {
        Ok((Addressable::Memory(address), 0))
    }

    fn target_absolute_indexed(base_address: u16, index_register: u8, force_page_boundary_check: bool) -> Result<(Addressable, CyclesTaken)> {
        let mut cycles_taken = 0;

        // +1 cycle if page boundary crossed or if we are forcing a page boundary check
        // (which usually occurs as part of a write)
        let (_, page_boundary_crossed) = (base_address as u8).overflowing_add(index_register);
        if page_boundary_crossed || force_page_boundary_check {
            cycles_taken += 1;
        }

        let address = base_address.wrapping_add(index_register as u16);

        Ok((Addressable::Memory(address), cycles_taken))
    }
}

impl Addressable {
    pub fn address(&self) -> Result<Address> {
        match self {
            Addressable::Accumulator => Err(Error::InvalidAddressAttempt(*self)),
            Addressable::Immediate(_) => Err(Error::InvalidAddressAttempt(*self)),
            Addressable::Memory(address) => Ok(*address),
        }
    }

    pub fn read<B: Bus>(&self, cpu: &MOS6502<B>) -> (u8, CyclesTaken) {
        match self {
            Addressable::Accumulator => (cpu.a, 0),
            Addressable::Immediate(value) => (*value, 0),
            Addressable::Memory(address) => {
                let value = cpu.bus.read_u8(*address);
                (value, 1)
            }
        }
    }

    pub fn try_write<B: Bus>(&self, cpu: &mut MOS6502<B>, value: u8) -> Result<CyclesTaken> {
        match *self {
            Addressable::Immediate(_) => Err(Error::InvalidAddressableWrite(*self, value)),
            Addressable::Accumulator => {
                cpu.write_register(Register::A, value);
                Ok(0)
            }
            Addressable::Memory(address) => {
                cpu.bus.write_u8(address, value);
                Ok(1)
            }
        }
    }

    pub fn try_modify<B: Bus>(&self, cpu: &mut MOS6502<B>, f: impl FnOnce(u8) -> u8) -> Result<(u8, u8, CyclesTaken)>
    {
        let (value, read_cycles) = self.read(cpu);
        let result = f(value);
        let mut write_cycles = self.try_write(cpu, result)?;

        // For non-accumulator modify instructions the 6502 writes the result twice, incurring an extra cycle cost
        if *self != Addressable::Accumulator {
            write_cycles += 1;
        }

        // When doing a `modify` we affect `Zero` and `Negative` even when
        // writing to memory
        cpu.p.set(StatusFlag::Zero, result == 0);
        cpu.p.set(StatusFlag::Negative, result & 0b1000_0000 > 0);

        Ok((value, result, read_cycles + write_cycles))
    }
}
