mod pixel;
mod texture;
mod ppuctrl;
mod ppumask;
mod ppustatus;

use nestalgic_mos6502::Bus;
pub use ppuctrl::PPUCtrl;
pub use ppumask::PPUMask;
pub use ppustatus::PPUStatus;
pub use pixel::Pixel;
pub use texture::Texture;


/// `RP2C02` emulates the NES PPU (a.k.a the `RP2C02`)
pub struct RP2C02 {
    pub pixels: [Pixel; RP2C02::SCREEN_PIXELS],

    pub ppuctrl: PPUCtrl,

    pub ppumask: PPUMask,

    /// Backing field for `ppustatus`. Use `read_ppustatus()` to access.
    ppustatus: PPUStatus,

    pub oam_addr: u8,
    pub oam_data: [u8; 256],

    pub addr: u16,

    /// Determines if we are writing to the high 8 bits of `addr` or the low 8 bits.
    ///
    /// If false: Write to the high 8 bits
    /// If true: Write to the low 8 bits
    ///
    /// Toggled on each write to `addr` (shared by PPUADDR and PPUSCROLL)
    /// Set to false when reading `ppustatus`
    pub addr_latch: bool,

    pub horizontal_scroll: u8,

    pub vertical_scroll:u8,

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
            ppumask: PPUMask::default(),
            ppustatus: PPUStatus::default(),
            addr: 0,
            addr_latch: false,
            oam_addr: 0,
            oam_data: [0; 256],
            horizontal_scroll: 0,
            vertical_scroll: 0,
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


    /// This function is only defined for addresses `0x2000-0x3FFF`, attempting to
    /// read outside this range will result in a panic.
    pub fn cpu_mapped_read_u8(&mut self, ppu_bus: &mut impl Bus, address: u16) -> u8 {
        match address {
            0x2000 => panic!("0x2000 is not readable"),
            0x2001 => panic!("0x2001 is not readable"),
            0x2002 => self.read_ppustatus().into(), // PPU Status
            0x2003 => panic!("0x2003 is not readable"),
            0x2004 => self.oam_data[self.oam_addr as usize],
            0x2005 => panic!("0x2005 is not readable"),
            0x2006 => panic!("0x2006 is not readable"),
            0x2007 => self.read_ppudata(ppu_bus),

            // Memory is mirrored everey 8 bytes up to 0x3FFF
            0x2008..=0x3FFF => self.cpu_mapped_read_u8(ppu_bus, address & 0x2007),

            _ => panic!("cpu_mapped_read_u8 expects address in range 0x2000-0x3FFF, was {}", address)
        }
    }

    /// This function is only defined for addresses `0x2000-0x3FFF`, attempting to
    /// write outside this range will result in a panic.
    pub fn cpu_mapped_write_u8(&mut self, ppu_bus: &mut impl Bus, address: u16, data: u8) {
        match address {
            0x2000 => self.ppuctrl.0 = data,
            0x2001 => self.ppumask = PPUMask::from(data),
            0x2002 => panic!("0x2002 is not writable"),
            0x2003 => self.oam_addr = data,
            0x2004 => self.write_oamdata(data),
            0x2005 => self.write_ppuscroll(data),
            0x2006 => self.write_ppuaddr(data),
            0x2007 => self.write_ppudata(ppu_bus, data),

            // Memory is mirrored everey 8 bytes up to 0x3FFF
            0x2008..=0x3FFF => self.cpu_mapped_write_u8(ppu_bus, address & 0x2007, data),

            _ => panic!("cpu_mapped_write_u8 expects address in range 0x2000-0x3FFF, was {} = {}", address, data)
        }
    }

    pub fn write_ppuaddr(&mut self, data: u8) {
        let [addr_lo, addr_hi] = self.addr.to_le_bytes();
        let [addr_lo, addr_hi] = if self.addr_latch {
            [addr_lo, data]
        } else {
            [data, addr_hi]
        };

        self.addr = u16::from_le_bytes([addr_lo, addr_hi]);
        self.addr_latch = !self.addr_latch;
    }

    pub fn write_ppuscroll(&mut self, data: u8) {
        if !self.addr_latch {
            self.horizontal_scroll = data;
        } else {
            self.vertical_scroll = data;
        }

        self.addr_latch = !self.addr_latch;
    }

    pub fn read_ppustatus(&mut self) -> PPUStatus {
        self.addr_latch = false;

        let old_ppustatus = self.ppustatus;

        // in_vblank is cleared after reading PPUStatus
        self.ppustatus.in_vblank = false;

        old_ppustatus
    }

    pub fn read_ppudata(&mut self, bus: &mut impl Bus) -> u8 {
        // TODO: Mirror values above 0x3FFF
        let value = bus.read_u8(self.addr);
        self.addr += self.ppuctrl.vram_address_increment() as u16;
        value
    }

    pub fn write_ppudata(&mut self, bus: &mut impl Bus, data: u8) {
        bus.write_u8(self.addr, data);
        self.addr += self.ppuctrl.vram_address_increment() as u16;
    }

    pub fn write_oamdata(&mut self, data: u8) {
        self.oam_data[self.oam_addr as usize] = data;
        self.oam_addr += 1; // TODO: Does this wrap?
    }
}
