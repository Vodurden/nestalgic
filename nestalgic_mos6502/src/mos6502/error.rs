use thiserror::Error;

use super::instruction::Instruction;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Invalid instruction: {0:x}")]
    InvalidInstruction(u8),

    #[error("Invalid attempt to read instruction value with instruction: {0:?}")]
    ImpliedReadValue(Instruction),

    #[error("Invalid attempt to read address with instruction: {0:?}")]
    InvalidReadAddress(Instruction),
}
