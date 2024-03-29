use thiserror::Error;

use super::instruction::Instruction;
use super::addressing_mode::Addressing;
use super::addressable::AddressableTarget;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Invalid instruction: {0:X}")]
    InvalidInstruction(u8),

    #[error("Invalid attempt to target address with addressing: {0:?}")]
    InvalidTargetAddressAttempt(Addressing),

    #[error("Invalid attempt to access address of {0:?}")]
    InvalidAddressAttempt(AddressableTarget),

    #[error("Invalid attempt to write to unwritable addressable: {0:?} = {1:?}")]
    InvalidAddressableWrite(AddressableTarget, u8),

    #[error("Invalid attempt to modify to unmodifiable addressable: {0:?}")]
    InvalidAddressableModify(AddressableTarget),

    #[error("Invalid attempt to read using invalid addressing mode with instruction: {0:?}")]
    InvalidReadValue(Instruction),

    #[error("Invalid attempt to write using invalid addessing mode with instruction: {0:?}")]
    InvalidWriteValue(Instruction),

    #[error("Invalid attempt to read address with instruction: {0:?}")]
    InvalidReadAddress(Instruction),
}
