mod pixel;

use crate::mapper::Mapper;
pub use pixel::Pixel;

/// `RP2C02` emulates the NES PPU (a.k.a the `RP2C02`)
pub struct RP2C02 {
    pub pixels: [Pixel; RP2C02::SCREEN_PIXELS],

    // TODO: https://wiki.nesdev.com/w/index.php/PPU_memory_map
    //
    // Position, palette and status of up to 64 sprites
    // object_attribute_memory: [u8; 64],

    // Character ROM, can also be a RAM
    // chr_rom: [u8; 8192],

    // A table of 32x30 bytes that specify which 8x8 pattern to use
    // nametable: [u8; 2048],

    // Specifies which 4-color palette is used for each 16x16 group of tiles
    //attribute_table: []

    // There are 8 different 4-color palettes. The first color is always transparent, and the other 3 choose
    // from 64 different System Colors.
    // palette: [u8; 256],
}

impl RP2C02 {
    pub const SCREEN_WIDTH: usize = 256;
    pub const SCREEN_HEIGHT: usize = 240;
    pub const SCREEN_PIXELS: usize = RP2C02::SCREEN_WIDTH * RP2C02::SCREEN_HEIGHT;

    pub fn new() -> RP2C02 {
        RP2C02 {
            pixels: [Pixel::empty(); RP2C02::SCREEN_PIXELS]
        }
    }

    pub fn cycle(&mut self, _mapper: &mut dyn Mapper) {
        for pixel in &mut self.pixels[0..20] {
            *pixel = Pixel::new(255, 0, 0, 0);
        }
    }

    /// Read from the RP2C02 io mapped registers.
    ///
    /// Valid addresses:
    ///
    /// - `0x2000-0x2007`
    /// - `0x4014`
    ///
    /// Reads from other addresses will cause a panic
    pub fn read_u8(&self, _address: u16) -> u8 {
        0
    }

    /// Write to the RP2C02 io mapped registers.
    ///
    /// Valid addresses:
    ///
    /// - `0x2000-0x2007`
    /// - `0x4014`
    ///
    /// Writes to other addresses will cause a panic
    pub fn write_u8(&mut self, _address: u16, _data: u8) {
    }
}
