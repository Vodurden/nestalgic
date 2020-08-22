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

    pub fn cycle(&mut self, mapper: &mut dyn Mapper) {
        // Render first tile in pattern table 0 (0x0000-0x0FFF)
        //
        // Each tile is 8x8
        let plane_1 = (0..8).map(|a| mapper.ppu_read_u8(a)).collect::<Vec<u8>>();
        let plane_2 = (8..16).map(|a| mapper.ppu_read_u8(a)).collect::<Vec<u8>>();

        for y in 0..8 {
            let line_byte_1 = plane_1[y];
            let line_byte_2 = plane_2[y];

            for x in 0..8 {
                let pixel_bit_1 = (line_byte_1 >> 7 - x) & 1;
                let pixel_bit_2 = (line_byte_2 >> 7 - x) & 1;
                let pixel_value = pixel_bit_1 + (pixel_bit_2 << 1);

                self.pixels[(y * RP2C02::SCREEN_WIDTH) + x] = match pixel_value {
                    0 => Pixel::empty(),
                    1 => Pixel::new(255, 0, 0, 255),
                    2 => Pixel::new(0, 255, 0, 255),
                    3 => Pixel::new(0, 0, 255, 255),
                    _ => Pixel::new(255, 0, 255, 255)
                };
            }
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
