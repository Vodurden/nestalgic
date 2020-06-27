use std::fmt;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum AddressingMode {
    // Consume 0, Return 0
    Implied, // None

    // Consume 0, Return 8-bit
    Accumulator, // u8

    // Consume 8bit, return 8bit
    Immediate, // u8 -> u8
    ZeroPage,  // u8 -> u8
    ZeroPageX, // (u8, x) -> u8
    ZeroPageY, // (u8, y) -> u8
    Relative,  // (s8)

    // 16-bit memory return value
    Absolute,  // u16 -> u8
    AbsoluteX, // (u16, x) -> u8
    AbsoluteY, // (u16, y) -> u8
    Indirect,  // u16 -> u16
    IndirectX, // (u16, x) -> u16. Should this be IndexedIndirectX?
    IndirectY, // (u16, y) -> u16. Should this be IndirectIndexedY?
}

impl fmt::Display for AddressingMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
