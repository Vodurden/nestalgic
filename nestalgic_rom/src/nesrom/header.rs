use super::Result;
use super::error::Error;
use super::file_type::FileType;
use super::mirroring_type::MirroringType;

use std::convert::TryInto;

#[derive(PartialEq, Debug)]
pub struct Header {
    pub file_type: FileType,

    /// The size of the program rom section in 16kb increments. For example: A value of `2` means the
    /// program rom has `32768` bytes (`16384 * prg_rom_size_16kb`).
    ///
    /// TODO: NES 2.0 doesn't always consider things in 16kb increments. Should we change this type?

    /// The number of bytes containing the program rom data.
    pub prg_rom_bytes: u32,

    /// The size of the character rom in 8kb increments. For example: A value of `1` means the
    /// character rom has `8192` bytes (`8192 * chr_rom_size_8kb`)

    /// The number of bytes containing the character rom data.
    pub chr_rom_bytes: u32,

    /// The type of Nametable Mirroring used by this ROM.
    ///
    /// TODO: Figure out what this is and write a better comment
    pub mirroring_type: MirroringType,

    /// If true the cartrige contained battery-backed persistent memory mapped between `0x6000` to `0x7FFF`.
    ///
    /// Usually this is used for save games and other persistent storage.
    pub has_persistent_memory: bool,

    /// If true the rom has a 512-byte trainer mapped into `0x7000` to `0x71FF`.
    ///
    /// Not used by unmodified dumps of real cartridges.
    pub has_trainer: bool,

    pub mapper_number: u16,
}

impl Header {
    pub fn from_bytes(rom_bytes: &[u8]) -> Result<Header> {
        if rom_bytes.len() < 16 {
            return Err(Error::InvalidHeader);
        }

        let rom_bytes: [u8; 16] = rom_bytes[0..16]
            .try_into()
            .map_err(|_| Error::InvalidHeader)?;

        let file_type = FileType::from_bytes(rom_bytes)?;
        match file_type {
            FileType::INES => Header::from_bytes_ines(rom_bytes),
            FileType::NES2 => Header::from_bytes_nes2(rom_bytes),
        }
    }

    /// Load a header in the ines format.
    fn from_bytes_ines(rom_bytes: [u8; 16]) -> Result<Header> {
        // For iNES byte 4 gives us the number of program rom bytes in increments of 16kb. This means
        // we need to multiply the value to get the actual number of bytes.
        let prg_rom_bytes = (rom_bytes[4] as u32) * 16384;

        // For iNES byte 5 gives us the number of character rom bytes in increments of 8kb.
        // As above we need to multiply the value to get the real number.
        let chr_rom_bytes = (rom_bytes[5] as u32) * 8192;

        let mirroring_type = MirroringType::from_ines_byte_6(rom_bytes[6]);
        let has_persistent_memory = (rom_bytes[6] & 0b0000_0010 >> 1) != 0;
        let has_trainer = (rom_bytes[6] & 0b0000_0100 >> 2) != 0;

        let mapper_lower_nibble = rom_bytes[6] & 0b1111_0000 >> 4;
        let mapper_upper_nibble = rom_bytes[7] & 0b1111_0000; // No shift since we're going to merge them
        let mapper_number = (mapper_upper_nibble | mapper_lower_nibble) as u16;

        let header = Header {
            file_type: FileType::INES,
            prg_rom_bytes,
            chr_rom_bytes,
            mirroring_type,
            has_persistent_memory,
            has_trainer,
            mapper_number,
        };

        Ok(header)
    }

    /// Load a header from the "NES 2.0" file format.
    ///
    /// At the moment we don't actually use any NES 2.0 file format features
    /// and the format is backwards compatible with INES so we just parse it
    /// with `from_bytes_ines` and change the file type.
    fn from_bytes_nes2(rom_bytes: [u8; 16]) -> Result<Header> {
        let mut ines_header = Header::from_bytes_ines(rom_bytes)?;
        ines_header.file_type = FileType::NES2;

        Ok(ines_header)
    }
}
