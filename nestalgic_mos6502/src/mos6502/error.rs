use thiserror::Error;

use super::instruction::Instruction;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Invalid instruction: {0:x}")]
    InvalidInstruction(u8),

    #[error("Invalid attempt to read using invalid addressing mode with instruction: {0:?}")]
    InvalidReadValue(Instruction),

    #[error("Invalid attempt to write using invalid addessing mode with instruction: {0:?}")]
    InvalidWriteValue(Instruction),

    #[error("Invalid attempt to read address with instruction: {0:?}")]
    InvalidReadAddress(Instruction),
}
