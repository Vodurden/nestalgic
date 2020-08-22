use std::fmt;

use super::{Address, BytesUsed, CyclesTaken, Result};
use super::MOS6502;
use super::addressable::{Addressable, AddressableTarget};
use super::bus::Bus;
use super::error::Error;

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
    pub fn read_addressable(self, cpu: &MOS6502, bus: &impl Bus) -> Result<(Addressable, CyclesTaken)> {
        match self {
            Addressing::Implied => Err(Error::InvalidTargetAddressAttempt(self)),
            Addressing::Accumulator => self.target_accumulator(),
            Addressing::Immediate(value) => self.target_immediate(value),
            Addressing::ZeroPage(address) => self.target_zero_page(address),
            Addressing::ZeroPageX(address) => self.target_zero_page_indexed(bus, address, cpu.x),
            Addressing::ZeroPageY(address) => self.target_zero_page_indexed(bus, address, cpu.y),
            Addressing::Relative(offset) => self.target_relative(cpu, offset),
            Addressing::IndexedIndirect(indexed_address) => self.target_indexed_indirect(cpu, bus, indexed_address),
            Addressing::IndirectIndexed(indexed_address) => self.target_indirect_indexed(cpu, bus, indexed_address),
            Addressing::Indirect(target_address) => self.target_indirect(bus, target_address),
            Addressing::Absolute(address) => self.target_absolute(address),
            Addressing::AbsoluteX(base_address) => self.target_absolute_indexed(base_address, cpu.x),
            Addressing::AbsoluteY(base_address) => self.target_absolute_indexed(base_address, cpu.y),
        }
    }

    fn target_accumulator(self) -> Result<(Addressable, CyclesTaken)> {
        let addressable = Addressable {
            addressing: self,
            target: AddressableTarget::Accumulator,
            page_boundary_crossed: false,
        };

        Ok((addressable, 0))
    }

    fn target_immediate(self, value: u8) -> Result<(Addressable, CyclesTaken)> {
        let addressable = Addressable {
            addressing: self,
            target: AddressableTarget::Immediate(value),
            page_boundary_crossed: false,
        };

        Ok((addressable, 0))
    }

    fn target_zero_page(self, zero_page_address: u8) -> Result<(Addressable, CyclesTaken)> {
        let addressable = Addressable {
            addressing: self,
            target: AddressableTarget::Memory(zero_page_address as u16),
            page_boundary_crossed: false,
        };

        Ok((addressable, 0))
    }

    fn target_zero_page_indexed(
        self,
        bus: &impl Bus,
        address: u8,
        register: u8
    ) -> Result<(Addressable, CyclesTaken)> {
        // The 6502 does a dummy read on zero page indexed that it throws away. +1 Cycle
        let _ = bus.read_u8(address as u16);
        let address = address.wrapping_add(register);
        let cycles_taken = 1;

        let addressable = Addressable {
            addressing: self,
            target: AddressableTarget::Memory(address as u16),
            page_boundary_crossed: false,
        };

        Ok((addressable, cycles_taken))
    }

    fn target_relative(self, cpu: &MOS6502, offset: u8) -> Result<(Addressable, CyclesTaken)> {
        let signed_offset = offset as i8;
        let target = cpu.pc.wrapping_add(signed_offset as u16);

        let [pc_lo, pc_hi] = cpu.pc.to_le_bytes();
        let pc_lo = pc_lo.wrapping_add(offset);
        let target_fixed_lo = u16::from_le_bytes([pc_lo, pc_hi]);

        let page_boundary_crossed = target != target_fixed_lo;

        let addressable = Addressable {
            addressing: self,
            target: AddressableTarget::Memory(target),
            page_boundary_crossed,
        };

        Ok((addressable, 0))
    }

    fn target_indexed_indirect(
        self,
        cpu: &MOS6502,
        bus: &impl Bus,
        indexed_address: u8
    ) -> Result<(Addressable, CyclesTaken)> {
        // Adding `x` to the address costs 1 cycle on the 6502.
        let target_address_lo = indexed_address.wrapping_add(cpu.x);
        let mut cycles_taken = 1;
        let target_lo = bus.read_u8(target_address_lo as u16);
        cycles_taken += 1;

        // Incrementing `target_address_lo` by one is done as part of the read cycle so it
        // doesn't cost an extra cycle
        let target_address_hi = target_address_lo.wrapping_add(1);
        let target_hi = bus.read_u8(target_address_hi as u16);
        cycles_taken += 1;

        // We don't use `cpu.read_u16` here because we need each part of
        // the 8-bit address to wrap around on the zero page, rather then
        // the whole address space
        let target_address = u16::from_le_bytes([target_lo, target_hi]);

        let addressable = Addressable {
            addressing: self,
            target: AddressableTarget::Memory(target_address),
            page_boundary_crossed: false,
        };

        Ok((addressable, cycles_taken))
    }

    fn target_indirect_indexed(
        self,
        cpu: &MOS6502,
        bus: &impl Bus,
        indexed_address: u8
    ) -> Result<(Addressable, CyclesTaken)> {
        let target_address_lo = indexed_address;
        let target_lo = bus.read_u8(target_address_lo as u16);
        let mut cycles_taken = 1;

        let target_address_hi = indexed_address.wrapping_add(1);
        let target_hi = bus.read_u8(target_address_hi as u16);
        cycles_taken += 1;

        // We don't use `cpu.read_u16` here because we need each part of
        // the 8-bit address to wrap around on the zero page, rather then
        // the whole address space
        let target_address = u16::from_le_bytes([target_lo, target_hi]);

        // +1 cycle if page boundary is crossed or if we are forcing a page
        // boundary check (which usually occurs as part of a write)
        // TODO: Move to Addressable
        let (_, page_boundary_crossed) = target_lo.overflowing_add(cpu.y);

        let adjusted_address = target_address.wrapping_add(cpu.y as u16);

        let addressable = Addressable {
            addressing: self,
            target: AddressableTarget::Memory(adjusted_address),
            page_boundary_crossed,
        };

        Ok((addressable, cycles_taken))
    }

    fn target_indirect(
        self,
        bus: &impl Bus,
        target_address: Address
    ) -> Result<(Addressable, CyclesTaken)> {
        let address_lo = bus.read_u8(target_address);
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
        let address_hi = bus.read_u8(target_address_plus_one_with_bug);
        cycles_taken += 1;

        let address = u16::from_le_bytes([address_lo, address_hi]);

        let addressable = Addressable {
            addressing: self,
            target: AddressableTarget::Memory(address),
            page_boundary_crossed: false,
        };

        Ok((addressable, cycles_taken))
    }

    fn target_absolute(self, address: u16) -> Result<(Addressable, CyclesTaken)> {
        let addressable = Addressable {
            addressing: self,
            target: AddressableTarget::Memory(address),
            page_boundary_crossed: false,
        };

        Ok((addressable, 0))
    }

    fn target_absolute_indexed(self, base_address: u16, index_register: u8) -> Result<(Addressable, CyclesTaken)> {
        // +1 cycle if page boundary crossed or if we are forcing a page boundary check
        // (which usually occurs as part of a write)
        let (_, page_boundary_crossed) = (base_address as u8).overflowing_add(index_register);
        let address = base_address.wrapping_add(index_register as u16);

        let addressable = Addressable {
            addressing: self,
            target: AddressableTarget::Memory(address),
            page_boundary_crossed,
        };

        Ok((addressable, 0))
    }
}
