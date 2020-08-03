use std::convert::TryFrom;

use super::{Address, BytesUsed, CyclesTaken, Result};
use super::bus::Bus;
use super::error::Error;
use super::opcode::Opcode;
use super::addressing_mode::{AddressingMode, Addressing};

/// An instruction is a fully realized 6502 instruction including the `Opcode` (`LDA`, `STX`, etc...), the
/// `AddressingMode` of the instruction and the target `Address` of the operation.
///
/// Example: `LDA $#100`
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Instruction {
    pub opcode: Opcode,

    /// The value of `addressing` depends on the `AddressingMode` of the
    ///
    /// For `Implied` and `Acummulator` addressing modes a read is performed into `argument` but the value
    /// should never be used as the 6502 always discards this value.
    ///
    /// For the `Immediate` addressing mode `argument` is a raw 8-bit value that should be used directly.
    ///
    /// For any other addressing mode `argument` is an address that points to the address of the true value.
    pub addressing: Addressing,
}

impl Instruction {
    /// Attempt to read an instruction from `bus` starting from `start_address`.
    ///
    /// Returns either a failure or:
    ///
    /// - The `Instruction`
    /// - The number of bytes read from the bus
    /// - The number of bytes used to construct the instruction
    ///
    /// For most operations bytes_read and bytes_used will be the same. The exceptions are
    /// `AddressingMode::Implied` and `AddressingMode::Accumulator` where the 6502 reads
    /// 1 byte but uses 0
    pub fn try_from_bus(start: Address, bus: &impl Bus) -> Result<(Instruction, CyclesTaken, BytesUsed)> {
        let (signature, signature_cycles_taken, signature_bytes_used) = InstructionSignature::try_from_bus(start, bus)?;
        let (addressing, addressing_cycles_taken, addressing_bytes_used) = signature.addressing_mode.read_addressing(
            start + signature_bytes_used,
            bus
        );

        let instruction = Instruction {
            opcode: signature.opcode,
            addressing,
        };

        let cycles_taken = signature_cycles_taken + addressing_cycles_taken;
        let bytes_used = signature_bytes_used + addressing_bytes_used;

        Ok((instruction, cycles_taken, bytes_used))
    }
}

/// The signature of an instruction is it's `Opcode` + `AddressingMode` pair.
///
/// This tells us what kinds of arguments we should expect and what operation we should
/// perform.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct InstructionSignature {
    pub opcode: Opcode,
    pub addressing_mode: AddressingMode,
}

impl TryFrom<u8> for InstructionSignature {
    type Error = Error;

    fn try_from(byte: u8) -> Result<Self> {
        INSTRUCTION_SIGNATURES[byte as usize]
            .ok_or_else(|| Error::InvalidInstruction(byte))
    }
}

impl InstructionSignature {
    pub const fn new(opcode: Opcode, addressing_mode: AddressingMode) -> InstructionSignature {
        InstructionSignature { opcode, addressing_mode }
    }

    /// Attempt to read an `InstructionSignature` from `bus` at `address`.
    ///
    /// Returns either a failure or the `InstructionSignature` and the number of bytes read from the bus.
    pub fn try_from_bus(address: Address, bus: &impl Bus) -> Result<(InstructionSignature, CyclesTaken, BytesUsed)> {
        let byte = bus.read_u8(address);
        let instruction_signature = InstructionSignature::try_from(byte)?;

        Ok((instruction_signature, 1, 1))
    }
}

/// Instruction signatures for all official 6502 opcodes
///
/// Also includes enough unofficial opcodes to get `nestest` to pass.
static INSTRUCTION_SIGNATURES: [Option<InstructionSignature>; 256] = [
    /*0x00*/ Some(InstructionSignature::new(Opcode::BRK, AddressingMode::Implied)),
    /*0x01*/ Some(InstructionSignature::new(Opcode::ORA, AddressingMode::IndexedIndirect)),
    /*0x02*/ None,
    /*0x03*/ None,
    /*0x04*/ Some(InstructionSignature::new(Opcode::NOP, AddressingMode::ZeroPage)), // Unofficial
    /*0x05*/ Some(InstructionSignature::new(Opcode::ORA, AddressingMode::ZeroPage)),
    /*0x06*/ Some(InstructionSignature::new(Opcode::ASL, AddressingMode::ZeroPage)),
    /*0x07*/ None,
    /*0x08*/ Some(InstructionSignature::new(Opcode::PHP, AddressingMode::Implied)),
    /*0x09*/ Some(InstructionSignature::new(Opcode::ORA, AddressingMode::Immediate)),
    /*0x0A*/ Some(InstructionSignature::new(Opcode::ASL, AddressingMode::Accumulator)),
    /*0x0B*/ None,
    /*0x0C*/ Some(InstructionSignature::new(Opcode::NOP, AddressingMode::Absolute)), // Unofficial
    /*0x0D*/ Some(InstructionSignature::new(Opcode::ORA, AddressingMode::Absolute)),
    /*0x0E*/ Some(InstructionSignature::new(Opcode::ASL, AddressingMode::Absolute)),
    /*0x0F*/ None,
    /*0x10*/ Some(InstructionSignature::new(Opcode::BPL, AddressingMode::Relative)),
    /*0x11*/ Some(InstructionSignature::new(Opcode::ORA, AddressingMode::IndirectIndexed)),
    /*0x12*/ None,
    /*0x13*/ None,
    /*0x14*/ Some(InstructionSignature::new(Opcode::NOP, AddressingMode::ZeroPageX)), // Unofficial
    /*0x15*/ Some(InstructionSignature::new(Opcode::ORA, AddressingMode::ZeroPageX)),
    /*0x16*/ Some(InstructionSignature::new(Opcode::ASL, AddressingMode::ZeroPageX)),
    /*0x17*/ None,
    /*0x18*/ Some(InstructionSignature::new(Opcode::CLC, AddressingMode::Implied)),
    /*0x19*/ Some(InstructionSignature::new(Opcode::ORA, AddressingMode::AbsoluteY)),
    /*0x1A*/ Some(InstructionSignature::new(Opcode::NOP, AddressingMode::Implied)), // Unofficial
    /*0x1B*/ None,
    /*0x1C*/ Some(InstructionSignature::new(Opcode::NOP, AddressingMode::AbsoluteX)), // Unofficial
    /*0x1D*/ Some(InstructionSignature::new(Opcode::ORA, AddressingMode::AbsoluteX)),
    /*0x1E*/ Some(InstructionSignature::new(Opcode::ASL, AddressingMode::AbsoluteX)),
    /*0x1F*/ None,
    /*0x20*/ Some(InstructionSignature::new(Opcode::JSR, AddressingMode::Absolute)),
    /*0x21*/ Some(InstructionSignature::new(Opcode::AND, AddressingMode::IndexedIndirect)),
    /*0x22*/ None,
    /*0x23*/ None,
    /*0x24*/ Some(InstructionSignature::new(Opcode::BIT, AddressingMode::ZeroPage)),
    /*0x25*/ Some(InstructionSignature::new(Opcode::AND, AddressingMode::ZeroPage)),
    /*0x26*/ Some(InstructionSignature::new(Opcode::ROL, AddressingMode::ZeroPage)),
    /*0x27*/ None,
    /*0x28*/ Some(InstructionSignature::new(Opcode::PLP, AddressingMode::Implied)),
    /*0x29*/ Some(InstructionSignature::new(Opcode::AND, AddressingMode::Immediate)),
    /*0x2A*/ Some(InstructionSignature::new(Opcode::ROL, AddressingMode::Accumulator)),
    /*0x2B*/ None,
    /*0x2C*/ Some(InstructionSignature::new(Opcode::BIT, AddressingMode::Absolute)),
    /*0x2D*/ Some(InstructionSignature::new(Opcode::AND, AddressingMode::Absolute)),
    /*0x2E*/ Some(InstructionSignature::new(Opcode::ROL, AddressingMode::Absolute)),
    /*0x2F*/ None,
    /*0x30*/ Some(InstructionSignature::new(Opcode::BMI, AddressingMode::Relative)),
    /*0x31*/ Some(InstructionSignature::new(Opcode::AND, AddressingMode::IndirectIndexed)),
    /*0x32*/ None,
    /*0x33*/ None,
    /*0x34*/ Some(InstructionSignature::new(Opcode::NOP, AddressingMode::ZeroPageX)), // Unofficial
    /*0x35*/ Some(InstructionSignature::new(Opcode::AND, AddressingMode::ZeroPageX)),
    /*0x36*/ Some(InstructionSignature::new(Opcode::ROL, AddressingMode::ZeroPageX)),
    /*0x37*/ None,
    /*0x38*/ Some(InstructionSignature::new(Opcode::SEC, AddressingMode::Implied)),
    /*0x39*/ Some(InstructionSignature::new(Opcode::AND, AddressingMode::AbsoluteY)),
    /*0x3A*/ Some(InstructionSignature::new(Opcode::NOP, AddressingMode::Implied)), // Unofficial
    /*0x3B*/ None,
    /*0x3C*/ Some(InstructionSignature::new(Opcode::NOP, AddressingMode::AbsoluteX)), // Unofficial
    /*0x3D*/ Some(InstructionSignature::new(Opcode::AND, AddressingMode::AbsoluteX)),
    /*0x3E*/ Some(InstructionSignature::new(Opcode::ROL, AddressingMode::AbsoluteX)),
    /*0x3F*/ None,
    /*0x40*/ Some(InstructionSignature::new(Opcode::RTI, AddressingMode::Implied)),
    /*0x41*/ Some(InstructionSignature::new(Opcode::EOR, AddressingMode::IndexedIndirect)),
    /*0x42*/ None,
    /*0x43*/ None,
    /*0x44*/ Some(InstructionSignature::new(Opcode::NOP, AddressingMode::ZeroPage)), // Unofficial
    /*0x45*/ Some(InstructionSignature::new(Opcode::EOR, AddressingMode::ZeroPage)),
    /*0x46*/ Some(InstructionSignature::new(Opcode::LSR, AddressingMode::ZeroPage)),
    /*0x47*/ None,
    /*0x48*/ Some(InstructionSignature::new(Opcode::PHA, AddressingMode::Implied)),
    /*0x49*/ Some(InstructionSignature::new(Opcode::EOR, AddressingMode::Immediate)),
    /*0x4A*/ Some(InstructionSignature::new(Opcode::LSR, AddressingMode::Accumulator)),
    /*0x4B*/ None,
    /*0x4C*/ Some(InstructionSignature::new(Opcode::JMP, AddressingMode::Absolute)),
    /*0x4D*/ Some(InstructionSignature::new(Opcode::EOR, AddressingMode::Absolute)),
    /*0x4E*/ Some(InstructionSignature::new(Opcode::LSR, AddressingMode::Absolute)),
    /*0x4F*/ None,
    /*0x50*/ Some(InstructionSignature::new(Opcode::BVC, AddressingMode::Relative)),
    /*0x51*/ Some(InstructionSignature::new(Opcode::EOR, AddressingMode::IndirectIndexed)),
    /*0x52*/ None,
    /*0x53*/ None,
    /*0x54*/ Some(InstructionSignature::new(Opcode::NOP, AddressingMode::ZeroPageX)), // Unofficial
    /*0x55*/ Some(InstructionSignature::new(Opcode::EOR, AddressingMode::ZeroPageX)),
    /*0x56*/ Some(InstructionSignature::new(Opcode::LSR, AddressingMode::ZeroPageX)),
    /*0x57*/ None,
    /*0x58*/ Some(InstructionSignature::new(Opcode::CLI, AddressingMode::Implied)),
    /*0x59*/ Some(InstructionSignature::new(Opcode::EOR, AddressingMode::AbsoluteY)),
    /*0x5A*/ Some(InstructionSignature::new(Opcode::NOP, AddressingMode::Implied)), // Unofficial
    /*0x5B*/ None,
    /*0x5C*/ Some(InstructionSignature::new(Opcode::NOP, AddressingMode::AbsoluteX)), // Unofficial
    /*0x5D*/ Some(InstructionSignature::new(Opcode::EOR, AddressingMode::AbsoluteX)),
    /*0x5E*/ Some(InstructionSignature::new(Opcode::LSR, AddressingMode::AbsoluteX)),
    /*0x5F*/ None,
    /*0x60*/ Some(InstructionSignature::new(Opcode::RTS, AddressingMode::Implied)),
    /*0x61*/ Some(InstructionSignature::new(Opcode::ADC, AddressingMode::IndexedIndirect)),
    /*0x62*/ None,
    /*0x63*/ None,
    /*0x64*/ Some(InstructionSignature::new(Opcode::NOP, AddressingMode::ZeroPage)), // Unofficial
    /*0x65*/ Some(InstructionSignature::new(Opcode::ADC, AddressingMode::ZeroPage)),
    /*0x66*/ Some(InstructionSignature::new(Opcode::ROR, AddressingMode::ZeroPage)),
    /*0x67*/ None,
    /*0x68*/ Some(InstructionSignature::new(Opcode::PLA, AddressingMode::Implied)),
    /*0x69*/ Some(InstructionSignature::new(Opcode::ADC, AddressingMode::Immediate)),
    /*0x6A*/ Some(InstructionSignature::new(Opcode::ROR, AddressingMode::Accumulator)),
    /*0x6B*/ None,
    /*0x6C*/ Some(InstructionSignature::new(Opcode::JMP, AddressingMode::Indirect)),
    /*0x6D*/ Some(InstructionSignature::new(Opcode::ADC, AddressingMode::Absolute)),
    /*0x6E*/ Some(InstructionSignature::new(Opcode::ROR, AddressingMode::Absolute)),
    /*0x6F*/ None,
    /*0x70*/ Some(InstructionSignature::new(Opcode::BVS, AddressingMode::Relative)),
    /*0x71*/ Some(InstructionSignature::new(Opcode::ADC, AddressingMode::IndirectIndexed)),
    /*0x72*/ None,
    /*0x73*/ None,
    /*0x74*/ Some(InstructionSignature::new(Opcode::NOP, AddressingMode::ZeroPageX)), // Unofficial
    /*0x75*/ Some(InstructionSignature::new(Opcode::ADC, AddressingMode::ZeroPageX)),
    /*0x76*/ Some(InstructionSignature::new(Opcode::ROR, AddressingMode::ZeroPageX)),
    /*0x77*/ None,
    /*0x78*/ Some(InstructionSignature::new(Opcode::SEI, AddressingMode::Implied)),
    /*0x79*/ Some(InstructionSignature::new(Opcode::ADC, AddressingMode::AbsoluteY)),
    /*0x7A*/ Some(InstructionSignature::new(Opcode::NOP, AddressingMode::Implied)), // Unofficial
    /*0x7B*/ None,
    /*0x7C*/ Some(InstructionSignature::new(Opcode::NOP, AddressingMode::AbsoluteX)), // Unofficial
    /*0x7D*/ Some(InstructionSignature::new(Opcode::ADC, AddressingMode::AbsoluteX)),
    /*0x7E*/ Some(InstructionSignature::new(Opcode::ROR, AddressingMode::AbsoluteX)),
    /*0x7F*/ None,
    /*0x80*/ Some(InstructionSignature::new(Opcode::NOP, AddressingMode::Immediate)), // Unofficial
    /*0x81*/ Some(InstructionSignature::new(Opcode::STA, AddressingMode::IndexedIndirect)),
    /*0x82*/ None,
    /*0x83*/ Some(InstructionSignature::new(Opcode::SAX, AddressingMode::IndexedIndirect)), // Unofficial
    /*0x84*/ Some(InstructionSignature::new(Opcode::STY, AddressingMode::ZeroPage)),
    /*0x85*/ Some(InstructionSignature::new(Opcode::STA, AddressingMode::ZeroPage)),
    /*0x86*/ Some(InstructionSignature::new(Opcode::STX, AddressingMode::ZeroPage)),
    /*0x87*/ Some(InstructionSignature::new(Opcode::SAX, AddressingMode::ZeroPage)),
    /*0x88*/ Some(InstructionSignature::new(Opcode::DEY, AddressingMode::Implied)),
    /*0x89*/ None,
    /*0x8A*/ Some(InstructionSignature::new(Opcode::TXA, AddressingMode::Implied)),
    /*0x8B*/ None,
    /*0x8C*/ Some(InstructionSignature::new(Opcode::STY, AddressingMode::Absolute)),
    /*0x8D*/ Some(InstructionSignature::new(Opcode::STA, AddressingMode::Absolute)),
    /*0x8E*/ Some(InstructionSignature::new(Opcode::STX, AddressingMode::Absolute)),
    /*0x8F*/ Some(InstructionSignature::new(Opcode::SAX, AddressingMode::Absolute)), // Unofficial
    /*0x90*/ Some(InstructionSignature::new(Opcode::BCC, AddressingMode::Relative)),
    /*0x91*/ Some(InstructionSignature::new(Opcode::STA, AddressingMode::IndirectIndexed)),
    /*0x92*/ None,
    /*0x93*/ None,
    /*0x94*/ Some(InstructionSignature::new(Opcode::STY, AddressingMode::ZeroPageX)),
    /*0x95*/ Some(InstructionSignature::new(Opcode::STA, AddressingMode::ZeroPageX)),
    /*0x96*/ Some(InstructionSignature::new(Opcode::STX, AddressingMode::ZeroPageY)),
    /*0x97*/ Some(InstructionSignature::new(Opcode::SAX, AddressingMode::ZeroPageY)),
    /*0x98*/ Some(InstructionSignature::new(Opcode::TYA, AddressingMode::Implied)),
    /*0x99*/ Some(InstructionSignature::new(Opcode::STA, AddressingMode::AbsoluteY)),
    /*0x9A*/ Some(InstructionSignature::new(Opcode::TXS, AddressingMode::Implied)),
    /*0x9B*/ None,
    /*0x9C*/ None,
    /*0x9D*/ Some(InstructionSignature::new(Opcode::STA, AddressingMode::AbsoluteX)),
    /*0x9E*/ None,
    /*0x9F*/ None,
    /*0xA0*/ Some(InstructionSignature::new(Opcode::LDY, AddressingMode::Immediate)),
    /*0xA1*/ Some(InstructionSignature::new(Opcode::LDA, AddressingMode::IndexedIndirect)),
    /*0xA2*/ Some(InstructionSignature::new(Opcode::LDX, AddressingMode::Immediate)),
    /*0xA3*/ Some(InstructionSignature::new(Opcode::LAX, AddressingMode::IndexedIndirect)), // Unofficial
    /*0xA4*/ Some(InstructionSignature::new(Opcode::LDY, AddressingMode::ZeroPage)),
    /*0xA5*/ Some(InstructionSignature::new(Opcode::LDA, AddressingMode::ZeroPage)),
    /*0xA6*/ Some(InstructionSignature::new(Opcode::LDX, AddressingMode::ZeroPage)),
    /*0xA7*/ Some(InstructionSignature::new(Opcode::LAX, AddressingMode::ZeroPage)), // Unofficial
    /*0xA8*/ Some(InstructionSignature::new(Opcode::TAY, AddressingMode::Implied)),
    /*0xA9*/ Some(InstructionSignature::new(Opcode::LDA, AddressingMode::Immediate)),
    /*0xAA*/ Some(InstructionSignature::new(Opcode::TAX, AddressingMode::Implied)),
    /*0xAB*/ None,
    /*0xAC*/ Some(InstructionSignature::new(Opcode::LDY, AddressingMode::Absolute)),
    /*0xAD*/ Some(InstructionSignature::new(Opcode::LDA, AddressingMode::Absolute)),
    /*0xAE*/ Some(InstructionSignature::new(Opcode::LDX, AddressingMode::Absolute)),
    /*0xAF*/ Some(InstructionSignature::new(Opcode::LAX, AddressingMode::Absolute)), // Unofficial
    /*0xB0*/ Some(InstructionSignature::new(Opcode::BCS, AddressingMode::Relative)),
    /*0xB1*/ Some(InstructionSignature::new(Opcode::LDA, AddressingMode::IndirectIndexed)),
    /*0xB2*/ None,
    /*0xB3*/ Some(InstructionSignature::new(Opcode::LAX, AddressingMode::IndirectIndexed)), // Unofficial
    /*0xB4*/ Some(InstructionSignature::new(Opcode::LDY, AddressingMode::ZeroPageX)),
    /*0xB5*/ Some(InstructionSignature::new(Opcode::LDA, AddressingMode::ZeroPageX)),
    /*0xB6*/ Some(InstructionSignature::new(Opcode::LDX, AddressingMode::ZeroPageY)),
    /*0xB7*/ Some(InstructionSignature::new(Opcode::LAX, AddressingMode::ZeroPageY)), // Unofficial
    /*0xB8*/ Some(InstructionSignature::new(Opcode::CLV, AddressingMode::Implied)),
    /*0xB9*/ Some(InstructionSignature::new(Opcode::LDA, AddressingMode::AbsoluteY)),
    /*0xBA*/ Some(InstructionSignature::new(Opcode::TSX, AddressingMode::Implied)),
    /*0xBB*/ None,
    /*0xBC*/ Some(InstructionSignature::new(Opcode::LDY, AddressingMode::AbsoluteX)),
    /*0xBD*/ Some(InstructionSignature::new(Opcode::LDA, AddressingMode::AbsoluteX)),
    /*0xBE*/ Some(InstructionSignature::new(Opcode::LDX, AddressingMode::AbsoluteY)),
    /*0xBF*/ Some(InstructionSignature::new(Opcode::LAX, AddressingMode::AbsoluteY)), // Unofficial
    /*0xC0*/ Some(InstructionSignature::new(Opcode::CPY, AddressingMode::Immediate)),
    /*0xC1*/ Some(InstructionSignature::new(Opcode::CMP, AddressingMode::IndexedIndirect)),
    /*0xC2*/ None,
    /*0xC3*/ Some(InstructionSignature::new(Opcode::DCP, AddressingMode::IndexedIndirect)), // Unofficial
    /*0xC4*/ Some(InstructionSignature::new(Opcode::CPY, AddressingMode::ZeroPage)),
    /*0xC5*/ Some(InstructionSignature::new(Opcode::CMP, AddressingMode::ZeroPage)),
    /*0xC6*/ Some(InstructionSignature::new(Opcode::DEC, AddressingMode::ZeroPage)),
    /*0xC7*/ Some(InstructionSignature::new(Opcode::DCP, AddressingMode::ZeroPage)), // Unofficial
    /*0xC8*/ Some(InstructionSignature::new(Opcode::INY, AddressingMode::Implied)),
    /*0xC9*/ Some(InstructionSignature::new(Opcode::CMP, AddressingMode::Immediate)),
    /*0xCA*/ Some(InstructionSignature::new(Opcode::DEX, AddressingMode::Implied)),
    /*0xCB*/ None,
    /*0xCC*/ Some(InstructionSignature::new(Opcode::CPY, AddressingMode::Absolute)),
    /*0xCD*/ Some(InstructionSignature::new(Opcode::CMP, AddressingMode::Absolute)),
    /*0xCE*/ Some(InstructionSignature::new(Opcode::DEC, AddressingMode::Absolute)),
    /*0xCF*/ Some(InstructionSignature::new(Opcode::DCP, AddressingMode::Absolute)), // Unofficial
    /*0xD0*/ Some(InstructionSignature::new(Opcode::BNE, AddressingMode::Relative)),
    /*0xD1*/ Some(InstructionSignature::new(Opcode::CMP, AddressingMode::IndirectIndexed)),
    /*0xD2*/ None,
    /*0xD3*/ Some(InstructionSignature::new(Opcode::DCP, AddressingMode::IndirectIndexed)), // Unofficial
    /*0xD4*/ Some(InstructionSignature::new(Opcode::NOP, AddressingMode::ZeroPageX)), // Unofficial
    /*0xD5*/ Some(InstructionSignature::new(Opcode::CMP, AddressingMode::ZeroPageX)),
    /*0xD6*/ Some(InstructionSignature::new(Opcode::DEC, AddressingMode::ZeroPageX)),
    /*0xD7*/ Some(InstructionSignature::new(Opcode::DCP, AddressingMode::ZeroPageX)), // Unofficial
    /*0xD8*/ Some(InstructionSignature::new(Opcode::CLD, AddressingMode::Implied)),
    /*0xD9*/ Some(InstructionSignature::new(Opcode::CMP, AddressingMode::AbsoluteY)),
    /*0xDA*/ Some(InstructionSignature::new(Opcode::NOP, AddressingMode::Implied)), // Unofficial
    /*0xDB*/ Some(InstructionSignature::new(Opcode::DCP, AddressingMode::AbsoluteX)), // Unofficial
    /*0xDC*/ Some(InstructionSignature::new(Opcode::NOP, AddressingMode::AbsoluteX)), // Unofficial
    /*0xDD*/ Some(InstructionSignature::new(Opcode::CMP, AddressingMode::AbsoluteX)),
    /*0xDE*/ Some(InstructionSignature::new(Opcode::DEC, AddressingMode::AbsoluteX)),
    /*0xDF*/ Some(InstructionSignature::new(Opcode::DCP, AddressingMode::AbsoluteX)), // Unofficial
    /*0xE0*/ Some(InstructionSignature::new(Opcode::CPX, AddressingMode::Immediate)),
    /*0xE1*/ Some(InstructionSignature::new(Opcode::SBC, AddressingMode::IndexedIndirect)),
    /*0xE2*/ None,
    /*0xE3*/ Some(InstructionSignature::new(Opcode::ISC, AddressingMode::IndexedIndirect)), // Unofficial
    /*0xE4*/ Some(InstructionSignature::new(Opcode::CPX, AddressingMode::ZeroPage)),
    /*0xE5*/ Some(InstructionSignature::new(Opcode::SBC, AddressingMode::ZeroPage)),
    /*0xE6*/ Some(InstructionSignature::new(Opcode::INC, AddressingMode::ZeroPage)),
    /*0xE7*/ Some(InstructionSignature::new(Opcode::ISC, AddressingMode::ZeroPage)), // Unofficial
    /*0xE8*/ Some(InstructionSignature::new(Opcode::INX, AddressingMode::Implied)),
    /*0xE9*/ Some(InstructionSignature::new(Opcode::SBC, AddressingMode::Immediate)),
    /*0xEA*/ Some(InstructionSignature::new(Opcode::NOP, AddressingMode::Implied)),
    /*0xEB*/ Some(InstructionSignature::new(Opcode::SBC, AddressingMode::Immediate)), // Unofficial
    /*0xEC*/ Some(InstructionSignature::new(Opcode::CPX, AddressingMode::Absolute)),
    /*0xED*/ Some(InstructionSignature::new(Opcode::SBC, AddressingMode::Absolute)),
    /*0xEE*/ Some(InstructionSignature::new(Opcode::INC, AddressingMode::Absolute)),
    /*0xEF*/ Some(InstructionSignature::new(Opcode::ISC, AddressingMode::Absolute)), // Unofficial
    /*0xF0*/ Some(InstructionSignature::new(Opcode::BEQ, AddressingMode::Relative)),
    /*0xF1*/ Some(InstructionSignature::new(Opcode::SBC, AddressingMode::IndirectIndexed)),
    /*0xF2*/ None,
    /*0xF3*/ Some(InstructionSignature::new(Opcode::ISC, AddressingMode::IndirectIndexed)), // Unofficial
    /*0xF4*/ Some(InstructionSignature::new(Opcode::NOP, AddressingMode::ZeroPageX)), // Unofficial
    /*0xF5*/ Some(InstructionSignature::new(Opcode::SBC, AddressingMode::ZeroPageX)),
    /*0xF6*/ Some(InstructionSignature::new(Opcode::INC, AddressingMode::ZeroPageX)),
    /*0xF7*/ Some(InstructionSignature::new(Opcode::ISC, AddressingMode::ZeroPageX)), // Unofficial
    /*0xF8*/ Some(InstructionSignature::new(Opcode::SED, AddressingMode::Implied)),
    /*0xF9*/ Some(InstructionSignature::new(Opcode::SBC, AddressingMode::AbsoluteY)),
    /*0xFA*/ Some(InstructionSignature::new(Opcode::NOP, AddressingMode::Implied)), // Unofficial
    /*0xFB*/ Some(InstructionSignature::new(Opcode::ISC, AddressingMode::AbsoluteY)), // Unofficial
    /*0xFC*/ Some(InstructionSignature::new(Opcode::NOP, AddressingMode::AbsoluteX)), // Unofficial
    /*0xFD*/ Some(InstructionSignature::new(Opcode::SBC, AddressingMode::AbsoluteX)),
    /*0xFE*/ Some(InstructionSignature::new(Opcode::INC, AddressingMode::AbsoluteX)),
    /*0xFF*/ Some(InstructionSignature::new(Opcode::ISC, AddressingMode::AbsoluteX)), // Unofficial
];
