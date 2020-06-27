use std::convert::TryFrom;

use super::error::Error;
use super::opcode::Opcode;
use super::addressing_mode::AddressingMode;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Instruction {
    pub opcode: Opcode,
    pub addressing_mode: AddressingMode,
}

impl TryFrom<u8> for Instruction {
    type Error = Error;

    fn try_from(byte: u8) -> Result<Self, Self::Error> {
        INSTRUCTIONS[byte as usize]
            .ok_or_else(|| Error::InvalidInstruction(byte))
    }
}

impl Instruction {
    pub const fn new(opcode: Opcode, addressing_mode: AddressingMode) -> Instruction {
        Instruction { opcode, addressing_mode }
    }
}

pub static INSTRUCTIONS: [Option<Instruction>; 256] = [
    /*0x00*/ Some(Instruction::new(Opcode::BRK, AddressingMode::Implied)),
    /*0x01*/ Some(Instruction::new(Opcode::ORA, AddressingMode::IndirectX)),
    /*0x02*/ None,
    /*0x03*/ None,
    /*0x04*/ None,
    /*0x05*/ Some(Instruction::new(Opcode::ORA, AddressingMode::ZeroPage)),
    /*0x06*/ Some(Instruction::new(Opcode::ASL, AddressingMode::ZeroPage)),
    /*0x07*/ None,
    /*0x08*/ Some(Instruction::new(Opcode::PHP, AddressingMode::Implied)),
    /*0x09*/ Some(Instruction::new(Opcode::ORA, AddressingMode::Immediate)),
    /*0x0A*/ Some(Instruction::new(Opcode::ASL, AddressingMode::Accumulator)),
    /*0x0B*/ None,
    /*0x0C*/ None,
    /*0x0D*/ Some(Instruction::new(Opcode::ORA, AddressingMode::Absolute)),
    /*0x0E*/ Some(Instruction::new(Opcode::ASL, AddressingMode::Absolute)),
    /*0x0F*/ None,
    /*0x10*/ Some(Instruction::new(Opcode::BPL, AddressingMode::Relative)),
    /*0x11*/ Some(Instruction::new(Opcode::ORA, AddressingMode::IndirectY)),
    /*0x12*/ None,
    /*0x13*/ None,
    /*0x14*/ None,
    /*0x15*/ Some(Instruction::new(Opcode::ORA, AddressingMode::ZeroPageX)),
    /*0x16*/ Some(Instruction::new(Opcode::ASL, AddressingMode::ZeroPageX)),
    /*0x17*/ None,
    /*0x18*/ Some(Instruction::new(Opcode::CLC, AddressingMode::Implied)),
    /*0x19*/ Some(Instruction::new(Opcode::ORA, AddressingMode::AbsoluteY)),
    /*0x1A*/ None,
    /*0x1B*/ None,
    /*0x1C*/ None,
    /*0x1D*/ Some(Instruction::new(Opcode::ORA, AddressingMode::AbsoluteX)),
    /*0x1E*/ Some(Instruction::new(Opcode::ASL, AddressingMode::AbsoluteX)),
    /*0x1F*/ None,
    /*0x20*/ Some(Instruction::new(Opcode::JSR, AddressingMode::Absolute)),
    /*0x21*/ Some(Instruction::new(Opcode::AND, AddressingMode::IndirectX)),
    /*0x22*/ None,
    /*0x23*/ None,
    /*0x24*/ Some(Instruction::new(Opcode::BIT, AddressingMode::ZeroPage)),
    /*0x25*/ Some(Instruction::new(Opcode::AND, AddressingMode::ZeroPage)),
    /*0x26*/ Some(Instruction::new(Opcode::ROL, AddressingMode::ZeroPage)),
    /*0x27*/ None,
    /*0x28*/ Some(Instruction::new(Opcode::PLP, AddressingMode::Implied)),
    /*0x29*/ Some(Instruction::new(Opcode::AND, AddressingMode::Immediate)),
    /*0x2A*/ Some(Instruction::new(Opcode::ROL, AddressingMode::Accumulator)),
    /*0x2B*/ None,
    /*0x2C*/ Some(Instruction::new(Opcode::BIT, AddressingMode::Absolute)),
    /*0x2D*/ Some(Instruction::new(Opcode::AND, AddressingMode::Absolute)),
    /*0x2E*/ Some(Instruction::new(Opcode::ROL, AddressingMode::Absolute)),
    /*0x2F*/ None,
    /*0x30*/ Some(Instruction::new(Opcode::BMI, AddressingMode::Relative)),
    /*0x31*/ Some(Instruction::new(Opcode::AND, AddressingMode::IndirectY)),
    /*0x32*/ None,
    /*0x33*/ None,
    /*0x34*/ None,
    /*0x35*/ Some(Instruction::new(Opcode::AND, AddressingMode::ZeroPageX)),
    /*0x36*/ Some(Instruction::new(Opcode::ROL, AddressingMode::ZeroPageX)),
    /*0x37*/ None,
    /*0x38*/ Some(Instruction::new(Opcode::SEC, AddressingMode::Implied)),
    /*0x39*/ Some(Instruction::new(Opcode::AND, AddressingMode::AbsoluteY)),
    /*0x3A*/ None,
    /*0x3B*/ None,
    /*0x3C*/ None,
    /*0x3D*/ Some(Instruction::new(Opcode::AND, AddressingMode::AbsoluteX)),
    /*0x3E*/ Some(Instruction::new(Opcode::ROL, AddressingMode::AbsoluteX)),
    /*0x3F*/ None,
    /*0x40*/ Some(Instruction::new(Opcode::RTI, AddressingMode::Implied)),
    /*0x41*/ Some(Instruction::new(Opcode::EOR, AddressingMode::IndirectX)),
    /*0x42*/ None,
    /*0x43*/ None,
    /*0x44*/ None,
    /*0x45*/ Some(Instruction::new(Opcode::EOR, AddressingMode::ZeroPage)),
    /*0x46*/ Some(Instruction::new(Opcode::LSR, AddressingMode::ZeroPage)),
    /*0x47*/ None,
    /*0x48*/ Some(Instruction::new(Opcode::PHA, AddressingMode::Implied)),
    /*0x49*/ Some(Instruction::new(Opcode::EOR, AddressingMode::Immediate)),
    /*0x4A*/ Some(Instruction::new(Opcode::LSR, AddressingMode::Accumulator)),
    /*0x4B*/ None,
    /*0x4C*/ Some(Instruction::new(Opcode::JMP, AddressingMode::Absolute)),
    /*0x4D*/ Some(Instruction::new(Opcode::EOR, AddressingMode::Absolute)),
    /*0x4E*/ Some(Instruction::new(Opcode::LSR, AddressingMode::Absolute)),
    /*0x4F*/ None,
    /*0x50*/ Some(Instruction::new(Opcode::BVC, AddressingMode::Relative)),
    /*0x51*/ Some(Instruction::new(Opcode::EOR, AddressingMode::IndirectY)),
    /*0x52*/ None,
    /*0x53*/ None,
    /*0x54*/ None,
    /*0x55*/ Some(Instruction::new(Opcode::EOR, AddressingMode::ZeroPageX)),
    /*0x56*/ Some(Instruction::new(Opcode::LSR, AddressingMode::ZeroPageX)),
    /*0x57*/ None,
    /*0x58*/ Some(Instruction::new(Opcode::CLI, AddressingMode::Implied)),
    /*0x59*/ Some(Instruction::new(Opcode::EOR, AddressingMode::AbsoluteY)),
    /*0x5A*/ None,
    /*0x5B*/ None,
    /*0x5C*/ None,
    /*0x5D*/ Some(Instruction::new(Opcode::EOR, AddressingMode::AbsoluteX)),
    /*0x5E*/ Some(Instruction::new(Opcode::LSR, AddressingMode::AbsoluteX)),
    /*0x5F*/ None,
    /*0x60*/ Some(Instruction::new(Opcode::RTS, AddressingMode::Implied)),
    /*0x61*/ Some(Instruction::new(Opcode::ADC, AddressingMode::IndirectX)),
    /*0x62*/ None,
    /*0x63*/ None,
    /*0x64*/ None,
    /*0x65*/ Some(Instruction::new(Opcode::ADC, AddressingMode::ZeroPage)),
    /*0x66*/ Some(Instruction::new(Opcode::ROR, AddressingMode::ZeroPage)),
    /*0x67*/ None,
    /*0x68*/ Some(Instruction::new(Opcode::PLA, AddressingMode::Implied)),
    /*0x69*/ Some(Instruction::new(Opcode::ADC, AddressingMode::Immediate)),
    /*0x6A*/ Some(Instruction::new(Opcode::ROR, AddressingMode::Accumulator)),
    /*0x6B*/ None,
    /*0x6C*/ Some(Instruction::new(Opcode::JMP, AddressingMode::Indirect)),
    /*0x6D*/ Some(Instruction::new(Opcode::ADC, AddressingMode::Absolute)),
    /*0x6E*/ Some(Instruction::new(Opcode::ROR, AddressingMode::Absolute)),
    /*0x6F*/ None,
    /*0x70*/ Some(Instruction::new(Opcode::BVS, AddressingMode::Relative)),
    /*0x71*/ Some(Instruction::new(Opcode::ADC, AddressingMode::IndirectY)),
    /*0x72*/ None,
    /*0x73*/ None,
    /*0x74*/ None,
    /*0x75*/ Some(Instruction::new(Opcode::ADC, AddressingMode::ZeroPageX)),
    /*0x76*/ Some(Instruction::new(Opcode::ROR, AddressingMode::ZeroPageX)),
    /*0x77*/ None,
    /*0x78*/ Some(Instruction::new(Opcode::SEI, AddressingMode::Implied)),
    /*0x79*/ Some(Instruction::new(Opcode::ADC, AddressingMode::AbsoluteY)),
    /*0x7A*/ None,
    /*0x7B*/ None,
    /*0x7C*/ None,
    /*0x7D*/ Some(Instruction::new(Opcode::ADC, AddressingMode::AbsoluteX)),
    /*0x7E*/ Some(Instruction::new(Opcode::ROR, AddressingMode::AbsoluteX)),
    /*0x7F*/ None,
    /*0x80*/ None,
    /*0x81*/ Some(Instruction::new(Opcode::STA, AddressingMode::IndirectX)),
    /*0x82*/ None,
    /*0x83*/ None,
    /*0x84*/ Some(Instruction::new(Opcode::STY, AddressingMode::ZeroPage)),
    /*0x85*/ Some(Instruction::new(Opcode::STA, AddressingMode::ZeroPage)),
    /*0x86*/ Some(Instruction::new(Opcode::STX, AddressingMode::ZeroPage)),
    /*0x87*/ None,
    /*0x88*/ Some(Instruction::new(Opcode::DEY, AddressingMode::Implied)),
    /*0x89*/ None,
    /*0x8A*/ Some(Instruction::new(Opcode::TXA, AddressingMode::Implied)),
    /*0x8B*/ None,
    /*0x8C*/ Some(Instruction::new(Opcode::STY, AddressingMode::Absolute)),
    /*0x8D*/ Some(Instruction::new(Opcode::STA, AddressingMode::Absolute)),
    /*0x8E*/ Some(Instruction::new(Opcode::STX, AddressingMode::Absolute)),
    /*0x8F*/ None,
    /*0x90*/ Some(Instruction::new(Opcode::BCC, AddressingMode::Relative)),
    /*0x91*/ Some(Instruction::new(Opcode::STA, AddressingMode::IndirectY)),
    /*0x92*/ None,
    /*0x93*/ None,
    /*0x94*/ Some(Instruction::new(Opcode::STY, AddressingMode::ZeroPageX)),
    /*0x95*/ Some(Instruction::new(Opcode::STA, AddressingMode::ZeroPageX)),
    /*0x96*/ Some(Instruction::new(Opcode::STX, AddressingMode::ZeroPageY)),
    /*0x97*/ None,
    /*0x98*/ Some(Instruction::new(Opcode::TYA, AddressingMode::Implied)),
    /*0x99*/ Some(Instruction::new(Opcode::STA, AddressingMode::AbsoluteY)),
    /*0x9A*/ Some(Instruction::new(Opcode::TXS, AddressingMode::Implied)),
    /*0x9B*/ None,
    /*0x9C*/ None,
    /*0x9D*/ Some(Instruction::new(Opcode::STA, AddressingMode::AbsoluteX)),
    /*0x9E*/ None,
    /*0x9F*/ None,
    /*0xA0*/ Some(Instruction::new(Opcode::LDY, AddressingMode::Immediate)),
    /*0xA1*/ Some(Instruction::new(Opcode::LDA, AddressingMode::IndirectX)),
    /*0xA2*/ Some(Instruction::new(Opcode::LDX, AddressingMode::Immediate)),
    /*0xA3*/ None,
    /*0xA4*/ Some(Instruction::new(Opcode::LDY, AddressingMode::ZeroPage)),
    /*0xA5*/ Some(Instruction::new(Opcode::LDA, AddressingMode::ZeroPage)),
    /*0xA6*/ Some(Instruction::new(Opcode::LDX, AddressingMode::ZeroPage)),
    /*0xA7*/ None,
    /*0xA8*/ Some(Instruction::new(Opcode::TAY, AddressingMode::Implied)),
    /*0xA9*/ Some(Instruction::new(Opcode::LDA, AddressingMode::Immediate)),
    /*0xAA*/ Some(Instruction::new(Opcode::TAX, AddressingMode::Implied)),
    /*0xAB*/ None,
    /*0xAC*/ Some(Instruction::new(Opcode::LDY, AddressingMode::Absolute)),
    /*0xAD*/ Some(Instruction::new(Opcode::LDA, AddressingMode::Absolute)),
    /*0xAE*/ Some(Instruction::new(Opcode::LDX, AddressingMode::Absolute)),
    /*0xAF*/ None,
    /*0xB0*/ Some(Instruction::new(Opcode::BCS, AddressingMode::Relative)),
    /*0xB1*/ Some(Instruction::new(Opcode::LDA, AddressingMode::IndirectY)),
    /*0xB2*/ None,
    /*0xB3*/ None,
    /*0xB4*/ Some(Instruction::new(Opcode::LDY, AddressingMode::ZeroPageX)),
    /*0xB5*/ Some(Instruction::new(Opcode::LDA, AddressingMode::ZeroPageX)),
    /*0xB6*/ Some(Instruction::new(Opcode::LDX, AddressingMode::ZeroPageY)),
    /*0xB7*/ None,
    /*0xB8*/ Some(Instruction::new(Opcode::CLV, AddressingMode::Implied)),
    /*0xB9*/ Some(Instruction::new(Opcode::LDA, AddressingMode::AbsoluteY)),
    /*0xBA*/ Some(Instruction::new(Opcode::TSX, AddressingMode::Implied)),
    /*0xBB*/ None,
    /*0xBC*/ Some(Instruction::new(Opcode::LDY, AddressingMode::AbsoluteX)),
    /*0xBD*/ Some(Instruction::new(Opcode::LDA, AddressingMode::AbsoluteX)),
    /*0xBE*/ Some(Instruction::new(Opcode::LDX, AddressingMode::AbsoluteY)),
    /*0xBF*/ None,
    /*0xC0*/ Some(Instruction::new(Opcode::CPY, AddressingMode::Immediate)),
    /*0xC1*/ Some(Instruction::new(Opcode::CMP, AddressingMode::IndirectX)),
    /*0xC2*/ None,
    /*0xC3*/ None,
    /*0xC4*/ Some(Instruction::new(Opcode::CPY, AddressingMode::ZeroPage)),
    /*0xC5*/ Some(Instruction::new(Opcode::CMP, AddressingMode::ZeroPage)),
    /*0xC6*/ Some(Instruction::new(Opcode::DEC, AddressingMode::ZeroPage)),
    /*0xC7*/ None,
    /*0xC8*/ Some(Instruction::new(Opcode::INY, AddressingMode::Implied)),
    /*0xC9*/ Some(Instruction::new(Opcode::CMP, AddressingMode::Immediate)),
    /*0xCA*/ Some(Instruction::new(Opcode::DEX, AddressingMode::Implied)),
    /*0xCB*/ None,
    /*0xCC*/ Some(Instruction::new(Opcode::CPY, AddressingMode::Absolute)),
    /*0xCD*/ Some(Instruction::new(Opcode::CMP, AddressingMode::Absolute)),
    /*0xCE*/ Some(Instruction::new(Opcode::DEC, AddressingMode::Absolute)),
    /*0xCF*/ None,
    /*0xD0*/ Some(Instruction::new(Opcode::BNE, AddressingMode::Relative)),
    /*0xD1*/ Some(Instruction::new(Opcode::CMP, AddressingMode::IndirectY)),
    /*0xD2*/ None,
    /*0xD3*/ None,
    /*0xD4*/ None,
    /*0xD5*/ Some(Instruction::new(Opcode::CMP, AddressingMode::ZeroPageX)),
    /*0xD6*/ Some(Instruction::new(Opcode::DEC, AddressingMode::ZeroPageX)),
    /*0xD7*/ None,
    /*0xD8*/ Some(Instruction::new(Opcode::CLD, AddressingMode::Implied)),
    /*0xD9*/ Some(Instruction::new(Opcode::CMP, AddressingMode::AbsoluteY)),
    /*0xDA*/ None,
    /*0xDB*/ None,
    /*0xDC*/ None,
    /*0xDD*/ Some(Instruction::new(Opcode::CMP, AddressingMode::AbsoluteX)),
    /*0xDE*/ Some(Instruction::new(Opcode::DEC, AddressingMode::AbsoluteX)),
    /*0xDF*/ None,
    /*0xE0*/ Some(Instruction::new(Opcode::CPX, AddressingMode::Immediate)),
    /*0xE1*/ Some(Instruction::new(Opcode::SBC, AddressingMode::IndirectX)),
    /*0xE2*/ None,
    /*0xE3*/ None,
    /*0xE4*/ Some(Instruction::new(Opcode::CPX, AddressingMode::ZeroPage)),
    /*0xE5*/ Some(Instruction::new(Opcode::SBC, AddressingMode::ZeroPage)),
    /*0xE6*/ Some(Instruction::new(Opcode::INC, AddressingMode::ZeroPage)),
    /*0xE7*/ None,
    /*0xE8*/ Some(Instruction::new(Opcode::INX, AddressingMode::Implied)),
    /*0xE9*/ Some(Instruction::new(Opcode::SBC, AddressingMode::Immediate)),
    /*0xEA*/ Some(Instruction::new(Opcode::NOP, AddressingMode::Implied)),
    /*0xEB*/ None,
    /*0xEC*/ Some(Instruction::new(Opcode::CPX, AddressingMode::Absolute)),
    /*0xED*/ Some(Instruction::new(Opcode::SBC, AddressingMode::Absolute)),
    /*0xEE*/ Some(Instruction::new(Opcode::INC, AddressingMode::Absolute)),
    /*0xEF*/ None,
    /*0xF0*/ Some(Instruction::new(Opcode::BEQ, AddressingMode::Relative)),
    /*0xF1*/ Some(Instruction::new(Opcode::SBC, AddressingMode::IndirectY)),
    /*0xF2*/ None,
    /*0xF3*/ None,
    /*0xF4*/ None,
    /*0xF5*/ Some(Instruction::new(Opcode::SBC, AddressingMode::ZeroPageX)),
    /*0xF6*/ Some(Instruction::new(Opcode::INC, AddressingMode::ZeroPageX)),
    /*0xF7*/ None,
    /*0xF8*/ Some(Instruction::new(Opcode::SED, AddressingMode::Implied)),
    /*0xF9*/ Some(Instruction::new(Opcode::SBC, AddressingMode::AbsoluteY)),
    /*0xFA*/ None,
    /*0xFB*/ None,
    /*0xFC*/ None,
    /*0xFD*/ Some(Instruction::new(Opcode::SBC, AddressingMode::AbsoluteX)),
    /*0xFE*/ Some(Instruction::new(Opcode::INC, AddressingMode::AbsoluteX)),
    /*0xFF*/ None,
];