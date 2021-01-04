/// `PPUStatus` represents the PPU status register mapped to `0x2002`
///
/// Each bit in `PPUStatus` has a different meaning:
///
/// ```text
/// +---+---+---+---+---+---+---+---+
/// | V | S | O | . | . | . | . | . |
/// +---+---+---+---+---+---+---+---+
///   |   |   |   |   |   |   |   |
///   |   |   |   \------------------------ Least significant bits previously written into a PPU register.
///   |   |   |
///   |   |   \---------------------------- Sprite overflow
///   |   |
///   |   \-------------------------------- Sprite 0 hit
///   |
///   \------------------------------------ Vertical Blank status
/// ```
///
/// See also: https://wiki.nesdev.com/w/index.php/PPU_registers
#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub struct PPUMask {
    pub lsb_of_previous_ppu_register: u8,

    pub sprite_overflow: bool,

    /// `sprite_0_hit` is set to true if a non-zero pixel of sprite 0 overlaps with a non-zero background pixel.
    ///
    /// `sprite_0_hit` is reset to false at dot 1 of the pre-render line.
    pub sprite_0_hit: bool,

    /// True if the PPU is within Vertical Blank, false otherwise.
    ///
    /// This value is set to true at dot 1 of line 241 (the line after the post-render line)
    ///
    /// This value is set to false after reading 0x2002 and at dot 1 of the pre-render line.
    pub in_vblank: bool,
}
