use std::fmt;

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
    /// 0x0000: 0xEF
    /// 0x0001: 0xBE
    /// ...
    /// 0xBEEF: 0xAA
    /// ```
    ///
    /// If I execute `LDA $00` then `A` will contain `0xAA` since address `0x000` and `0x0001` reference address `0xBEEF`
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

    // 16-bit memory return value
    Absolute,  // u16 -> u8
    AbsoluteX, // (u16, x) -> u8
    AbsoluteY, // (u16, y) -> u8

    Relative,  // (s8)
    Indirect,  // u16 -> u16
    IndexedIndirect, // (u16, x) -> u16. Should this be IndexedIndirectX?
    IndirectIndexed, // (u16, y) -> u16. Should this be IndirectIndexedY?
}

impl fmt::Display for AddressingMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
