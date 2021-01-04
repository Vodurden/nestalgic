/// `PPUCtrl` represents the PPU control register mapped to `0x2000`.
///
/// Each bit in `PPUCtrl` has a different meaning:
///
/// ```text
/// +---+---+---+---+---+---+---+---+
/// | V | P | H | B | S | I | N | N |
/// +---+---+---+---+---+---+---+---+
///   |   |   |   |   |   |   |   |
///   |   |   |   |   |   |   \---\-------- Base nametable address
///   |   |   |   |   |   |                 (0 = 0x2000, 1 = 0x2400, 2 = 0x2800, 3 = 0x2C00)
///   |   |   |   |   |   |
///   |   |   |   |   |   \---------------- VRAM address increment per CPU read/write of PPUDATA
///   |   |   |   |   |                     (0: add 1, going across. 1: add 32, going down)
///   |   |   |   |   |
///   |   |   |   |   \-------------------- Sprite pattern table address for 8x8 sprites
///   |   |   |   |                         (0: 0x0000, 1: 0x1000. Ignored in 8x16 mode)
///   |   |   |   |
///   |   |   |   \------------------------ Background pattern table address
///   |   |   |                             (0: 0x0000, 1: 0x1000)
///   |   |   |
///   |   |   \---------------------------- Sprite size
///   |   |                                 (0: 8x8 pixels. 1: 8x16 pixels)
///   |   |
///   |   \-------------------------------- PPU select (unused)
///   |                                     (0: read backdrop from EXT pints. 1: output colour on EXT pins)
///   |
///   \------------------------------------ Generate a NMI at the start of VBLANK
///                                         (0: off. 1: on)
/// ```
///
/// See also: https://wiki.nesdev.com/w/index.php/PPU_registers
#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub struct PPUCtrl(pub u8);

impl PPUCtrl {
    pub fn get(&self, flag: PPUCtrlFlag) -> bool {
        let mask = flag as u8;
        (self.0 & mask) != 0
    }

    pub fn set(&mut self, flag: PPUCtrlFlag, value: bool) {
        let mask = flag as u8;
        if value {
            self.0 |= mask;
        } else {
            self.0 &= !mask;
        }
    }

    pub fn base_nametable_address(&self) -> u16 {
        match self.0 & 0b0000_0011 {
            0b00 => 0x2000,
            0b01 => 0x2400,
            0b10 => 0x2800,
            0b11 => 0x2C00,
            _ => unreachable!()
        }
    }

    pub fn vram_address_increment(&self) -> u8 {
        match self.get(PPUCtrlFlag::VramAddressIncrement) {
            false => 1,
            true => 32
        }
    }

    pub fn sprite_pattern_table_address(&self) -> u16 {
        match self.get(PPUCtrlFlag::SpritePatternTable) {
            false => 0x0000,
            true => 0x1000
        }
    }

    pub fn background_pattern_table_address(&self) -> u16 {
        match self.get(PPUCtrlFlag::BackgroundPatternTable) {
            false => 0x0000,
            true => 0x1000
        }
    }
}

impl Default for PPUCtrl {
    fn default() -> Self {
        PPUCtrl(0)
    }

}

pub enum PPUCtrlFlag {
    NametableLo             = 0b0000_0001,
    NametableHi             = 0b0000_0010,
    VramAddressIncrement    = 0b0000_0100,
    SpritePatternTable      = 0b0000_1000,
    BackgroundPatternTable  = 0b0001_0000,
    SpriteSize              = 0b0010_0000,
    PpuSelect               = 0b0100_0000,
    GenerateNmiOnVblank     = 0b1000_0000,
}
