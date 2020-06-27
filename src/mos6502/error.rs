use thiserror::Error;

use super::addressing_mode::AddressingMode;
use std::fmt;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Invalid instruction: {0:x}")]
    InvalidInstruction(u8),

    #[error("Invalid attempt to read target address in addressing mode {0} ")]
    InvalidAddressRead(AddressingMode),
}

// impl fmt::Display for Error {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         match self {
//             Error::InvalidInstruction(byte) => write!(f, "Invalid instruction: {:x}", byte),
//             Error::InvalidAddressRead(mode) => write!(f, "Invalid attempt to read address in mode {}", mode),
//         }
//     }
// }
