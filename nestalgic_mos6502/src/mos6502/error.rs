use thiserror::Error;

use super::addressing_mode::AddressingMode;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Invalid instruction: {0:x}")]
    InvalidInstruction(u8),

    #[error("Invalid attempt to read target address in addressing mode {0} ")]
    InvalidAddressRead(AddressingMode),
}
