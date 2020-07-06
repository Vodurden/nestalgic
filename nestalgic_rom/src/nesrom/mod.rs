mod header;
mod error;
mod file_type;
mod mirroring_type;

pub use header::Header;
pub use file_type::FileType;
pub use mirroring_type::MirroringType;

pub type Result<A> = std::result::Result<A, error::Error>;

#[derive(PartialEq, Debug)]
pub struct NESROM {
    pub header: Header,

    /// If `header.has_trainer` is true this will contain 512 bytes of data that should
    /// be mapped from `0x7000` to `0x71FF`.
    pub trainer: Option<Vec<u8>>,

    /// The program rom data.
    pub prg_rom: Vec<u8>,

    // The character rom data.
    pub chr_rom: Vec<u8>,
}

impl NESROM {
    pub fn from_bytes(bytes: Vec<u8>) -> Result<NESROM> {
        let mut bytes = bytes.into_iter();

        let header_bytes: Vec<u8> = bytes.by_ref().take(16).collect();
        let header = Header::from_bytes(&header_bytes)?;

        let trainer = if header.has_trainer {
            let trainer: Vec<u8> = bytes.by_ref().take(512).collect();
            Some(trainer)
        } else {
            None
        };

        let prg_rom: Vec<u8> = bytes.by_ref().take(header.prg_rom_bytes as usize).collect();
        let chr_rom: Vec<u8> = bytes.by_ref().take(header.chr_rom_bytes as usize).collect();

        let rom = NESROM {
            header,
            trainer,
            prg_rom,
            chr_rom
        };

        Ok(rom)
    }
}
