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
pub struct PPUStatus {
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

impl Default for PPUStatus {
    fn default() -> Self {
        Self {
            lsb_of_previous_ppu_register: 0,
            sprite_overflow: false,
            sprite_0_hit: false,
            in_vblank: false
        }
    }
}

impl From<PPUStatus> for u8 {
    fn from(status: PPUStatus) -> Self {
        let lsb_bits = status.lsb_of_previous_ppu_register & 0b0001_1111;
        let sprite_overflow = (status.sprite_overflow as u8) << 5;
        let sprite_0_hit = (status.sprite_0_hit as u8) << 6;
        let in_vblank = (status.in_vblank as u8) << 7;

        in_vblank | sprite_0_hit | sprite_overflow | lsb_bits
    }
}


/// Tests for `Bus`
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn ppustatus_to_mixed_u8() {
        let status = PPUStatus {
            lsb_of_previous_ppu_register: 0b0001_0101,
            sprite_overflow: true,
            sprite_0_hit: true,
            in_vblank: true
        };

        let status_u8: u8 = status.into();

        assert_eq!(status_u8, 0b1111_0101, "status was {:#08b}, expected {:#08b}", status_u8, 0b1111_0101);
    }

    #[test]
    pub fn ppustatus_to_empty_u8() {
        let status = PPUStatus {
            lsb_of_previous_ppu_register: 0b0000_0000,
            sprite_overflow: false,
            sprite_0_hit: false,
            in_vblank: false
        };

        let status_u8: u8 = status.into();

        assert_eq!(status_u8, 0b0000_0000, "status was {:#08b}, expected {:#08b}", status_u8, 0b0000_0000);
    }
}
