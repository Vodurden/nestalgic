mod pixel;
mod texture;
mod ppuctrl;
mod ppumask;

use crate::mapper::Mapper;

pub use ppuctrl::PPUCtrl;
pub use ppumask::PPUMask;
pub use pixel::Pixel;
pub use texture::Texture;

/// `RP2C02` emulates the NES PPU (a.k.a the `RP2C02`)
pub struct RP2C02 {
    pub pixels: [Pixel; RP2C02::SCREEN_PIXELS],

    pub ppuctrl: PPUCtrl,

    pub oam_addr: u16,

    pub oam_data: [u8; 256],

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
            pixels: [Pixel::empty(); RP2C02::SCREEN_PIXELS],
            ppuctrl: PPUCtrl::default(),
            oam_addr: 0,
            oam_data: [0; 256],
        }
    }

    pub fn cycle(&mut self, mapper: &mut dyn Mapper) {
        // Render first tile in pattern table 0 (0x0000-0x0FFF)
        //
        // Each tile is 8x8
        //
        // TODO: Render the last line of the pattern table without crashing
        let chr_data = (0..7 * 1024)
            .map(|a| mapper.ppu_read_u8(a as u16))
            .collect::<Vec<u8>>();

        for (i, chr) in chr_data.chunks(16).enumerate() {
            for y in 0..8 {
                let line_byte_1 = chr[y];
                let line_byte_2 = chr[8 + y];

                for x in 0..8 {
                    let pixel_bit_1 = (line_byte_1 >> 7 - x) & 1;
                    let pixel_bit_2 = (line_byte_2 >> 7 - x) & 1;
                    let pixel_value = pixel_bit_1 + (pixel_bit_2 << 1);

                    let offset_x = (i * 8) % RP2C02::SCREEN_WIDTH;
                    let offset_y = (i / 16) * 8;
                    let pixel_x = offset_x + x;
                    let pixel_y = offset_y + y;

                    self.pixels[(pixel_y * RP2C02::SCREEN_WIDTH) + pixel_x] = match pixel_value {
                        0 => Pixel::empty(),
                        1 => Pixel::new(255, 0, 0, 255),
                        2 => Pixel::new(0, 255, 0, 255),
                        3 => Pixel::new(0, 0, 255, 255),
                        _ => Pixel::new(255, 0, 255, 255)
                    };
                }
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
    pub fn read_u8(&self, address: u16) -> u8 {
        match address {
            0x2000 => self.ppuctrl.0,
            _ => 0
        }
    }

    /// Write to the RP2C02 io mapped registers.
    ///
    /// Valid addresses:
    ///
    /// - `0x2000-0x2007`
    /// - `0x4014`
    ///
    /// Writes to other addresses will cause a panic
    pub fn write_u8(&mut self, address: u16, data: u8) {
        println!("ppu_write: {:X} = {:X}", address, data);
    }
}
