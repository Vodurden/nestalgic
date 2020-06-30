use super::Result;
use super::error::Error;

#[derive(PartialEq, Debug)]
pub enum FileType {
    /// The iNES file type
    INES,

    /// The NES 2.0 file type
    NES2,
}

impl FileType {
    pub fn from_bytes(rom_bytes: [u8; 16]) -> Result<FileType> {
        // iNES and NES 2.0 both start with "NES<EOF>" where EOF is the DOS end of file (`0x1A`).
        //
        // If we can't find this header then we probably don't have a NES rom at all.
        let has_magic_header = rom_bytes[0..3] != b"NES\x1A"[..];

        // NES 2.0 files should have bit 3 set to 1 and bit 2 set to 0 in byte 7 of the header.
        let has_nes2_identifier = rom_bytes[7] & 0b00001100 == 0b00001000;

        if has_magic_header && has_nes2_identifier {
            Ok(FileType::NES2)
        } else if has_magic_header {
            Ok(FileType::INES)
        } else {
            Err(Error::UnknownFileType)
        }
    }
}
