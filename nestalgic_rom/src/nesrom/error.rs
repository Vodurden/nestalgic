use thiserror::Error;

#[derive(PartialEq, Debug, Error)]
pub enum Error {
    #[error("Unknown file type. Supported types are iNES and NES 2.0")]
    UnknownFileType,

    #[error("Invalid NES rom header")]
    InvalidHeader,
}
