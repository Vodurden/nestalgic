use std::fmt;

use super::Address;
use super::bus::Bus;

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

impl fmt::Display for AddressingMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl AddressingMode {
    /// Read an address from `bus` starting at `start` using the current addressing mode.
    ///
    /// Returns the address and the number of bytes read from the bus which is always `1` or `2`.
    pub fn read_address(&self, start: Address, bus: &impl Bus) -> (Address, u16) {
        match self {
            // 8-bit addressing modes
            AddressingMode::Implied => self.read_address_u8(start, bus),
            AddressingMode::Accumulator => self.read_address_u8(start, bus),
            AddressingMode::Immediate => self.read_address_u8(start, bus),
            AddressingMode::ZeroPage => self.read_address_u8(start, bus),
            AddressingMode::ZeroPageX => self.read_address_u8(start, bus),
            AddressingMode::ZeroPageY => self.read_address_u8(start, bus),
            AddressingMode::Relative => self.read_address_u8(start, bus),
            AddressingMode::IndexedIndirect => self.read_address_u8(start, bus),
            AddressingMode::IndirectIndexed => self.read_address_u8(start, bus),

            // 16-bit addressing modes
            AddressingMode::Absolute => self.read_address_u16(start, bus),
            AddressingMode::AbsoluteX => self.read_address_u16(start, bus),
            AddressingMode::AbsoluteY => self.read_address_u16(start, bus),
            AddressingMode::Indirect => self.read_address_u16(start, bus),
        }
    }

    fn read_address_u8(&self, start: Address, bus: &impl Bus) -> (Address, u16) {
        let address = bus.read_u8(start);
        (address as u16, 1)
    }

    fn read_address_u16(&self, start: Address, bus: &impl Bus) -> (Address, u16) {
        let address = bus.read_u16(start);
        (address, 2)
    }
}
