mod pixel;
mod texture;
mod ppuctrl;
mod ppumask;

use nestalgic_mos6502::Bus;
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

    pub addr: u16,

    /// Determines if we are writing to the high 8 bits of `addr` or the low 8 bits.
    ///
    /// If false: Write to the high 8 bits
    /// If true: Write to the low 8 bits
    ///
    /// Toggled on each write to `addr` (shared by PPUADDR and PPUSCROLL)
    pub addr_latch: bool,

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
            addr: 0,
            addr_latch: false,
            oam_addr: 0,
            oam_data: [0; 256],
        }
    }

    pub fn cycle(&mut self, bus: &mut impl Bus) {
        // Render first tile in pattern table 0 (0x0000-0x0FFF)
        //
        // Each tile is 8x8
        //
        // TODO: Render the last line of the pattern table without crashing
        let chr_data = (0..7 * 1024)
            .map(|a| bus.read_u8(a as u16))
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

    pub fn write_ppuaddr(&mut self, value: u8) {
        let [addr_lo, addr_hi] = self.addr.to_le_bytes();
        let [addr_lo, addr_hi] = if self.addr_latch {
            [addr_lo, value]
        } else {
            [value, addr_hi]
        };

        self.addr = u16::from_le_bytes([addr_lo, addr_hi]);
        self.addr_latch = !self.addr_latch;
    }

    pub fn read_ppudata(&mut self, bus: &impl Bus) -> u8 {
        // TODO: Mirror values above 0x3FFF
        let value = bus.read_u8(self.addr);
        self.addr += self.ppuctrl.vram_address_increment() as u16;
        value
    }

    pub fn write_ppudata(&mut self, bus: &mut impl Bus, value: u8) {

    }
}
