mod cpu_bus;
mod rp2c02;
mod mapper;

pub use nestalgic_rom::nesrom::NESROM;
pub use rp2c02::{Texture, Pixel};
use nestalgic_mos6502::mos6502::{MOS6502, DMA};
use rp2c02::RP2C02;
use mapper::Mapper;
use cpu_bus::CpuBus;

use std::time::Duration;

type WRAM = [u8; 2048];

pub struct Nestalgic {
    cpu: MOS6502,
    wram: WRAM,
    ppu: RP2C02,
    mapper: Box<dyn Mapper>,
    // TODO: APU
    // TODO: Input

    master_clock_speed: Duration,
    time_since_last_master_cycle: Duration,
}

impl Nestalgic {
    pub const SCREEN_PIXELS: usize = RP2C02::SCREEN_PIXELS;
    pub const SCREEN_WIDTH: usize = RP2C02::SCREEN_WIDTH;
    pub const SCREEN_HEIGHT: usize = RP2C02::SCREEN_HEIGHT;

    pub const PATTERN_TABLE_PIXELS: usize =
        Nestalgic::PATTERN_TABLE_WIDTH * Nestalgic::PATTERN_TABLE_HEIGHT;
    pub const PATTERN_TABLE_WIDTH: usize = 128;
    pub const PATTERN_TABLE_HEIGHT: usize = 128;

    pub fn new() -> Nestalgic {
        Nestalgic {
            cpu: Nestalgic::nes_cpu(),
            wram: [0; 2048],
            ppu: RP2C02::new(),
            mapper: Box::new(mapper::NullMapper::new()),

            master_clock_speed: Duration::from_secs_f64(1.0 / 21.477272),
            time_since_last_master_cycle: Duration::new(0, 0),
        }
    }

    fn nes_cpu() -> MOS6502 {
        let nes_dma = DMA {
            trigger_address: 0x4014,
            target_address: 0x2004,
            bytes_to_transfer: 256,
        };

        MOS6502::new().with_dma(nes_dma)
    }

    pub fn with_rom(mut self, rom: NESROM) -> Self {
        self.mapper = <dyn Mapper>::from_rom(rom);
        let mut cpu_bus = CpuBus {
            wram: &mut self.wram,
            ppu: &mut self.ppu,
            mapper: &mut *self.mapper
        };
        self.cpu.reset(&mut cpu_bus).expect("Failed to reset CPU");
        self
    }

    /// Simulate the NES forward by `delta` time. Depending on how much time has elapsed this may:
    ///
    /// - Cycle the CPU some number of times
    /// - Cycle the PPU some number of times
    ///
    pub fn tick(&mut self, delta: Duration) {
        self.time_since_last_master_cycle += delta;

        while self.time_since_last_master_cycle > self.master_clock_speed {
            self.time_since_last_master_cycle -= self.master_clock_speed;
            self.cycle();
        }
    }

    pub fn cycle(&mut self) {
        let mut cpu_bus = CpuBus {
            wram: &mut self.wram,
            ppu: &mut self.ppu,
            mapper: &mut *self.mapper
        };
        self.cpu.cycle(&mut cpu_bus).expect("failed to cycle cpu");
        self.ppu.cycle(&mut *self.mapper);
        self.ppu.cycle(&mut *self.mapper);
        self.ppu.cycle(&mut *self.mapper);
    }

    pub fn pixels(&self) -> &[Pixel; Nestalgic::SCREEN_PIXELS] {
        &self.ppu.pixels
    }

    pub fn pattern_table(&self) -> Texture {
        let chr_data = (0..=0x0FFF)
            .map(|a| self.mapper.ppu_read_u8(a as u16))
            .collect::<Vec<u8>>();

        Texture::from_bitplanes(&chr_data, 16, 128, 128)

        // let chr_bytes = Nestalgic::merge_bitplanes(&chr_data, 16);
        // let pixels: Vec<Pixel> = chr_bytes.iter().map(|chr_byte| {
        //     match chr_byte {
        //         0 => Pixel::empty(),
        //         1 => Pixel::new(255, 0, 0, 255),
        //         2 => Pixel::new(0, 255, 0, 255),
        //         3 => Pixel::new(0, 0, 255, 255),
        //         _ => Pixel::new(255, 0, 255, 255)
        //     }
        // }).collect();

        // //let pixels: Vec<Pixel> = [Pixel::empty(); 128 * 128].into();
        // Texture::new(&pixels, 128, 128)

        // for (i, chr) in chr_data.chunks(16).enumerate() {
        //     for y in 0..8 {
        //         let line_byte_1 = chr[y];
        //         let line_byte_2 = chr[8 + y];

        //         for x in 0..8 {
        //             let pixel_bit_1 = (line_byte_1 >> 7 - x) & 1;
        //             let pixel_bit_2 = (line_byte_2 >> 7 - x) & 1;
        //             let pixel_value = pixel_bit_1 + (pixel_bit_2 << 1);

        //             let offset_x = (i * 8) % Nestalgic::PATTERN_TABLE_WIDTH;
        //             let offset_y = (i / 16) * 8;
        //             let pixel_x = offset_x + x;
        //             let pixel_y = offset_y + y;

        //             pixels[(pixel_y * Nestalgic::PATTERN_TABLE_WIDTH) + pixel_x] = match pixel_value {
        //                 0 => Pixel::empty(),
        //                 1 => Pixel::new(255, 0, 0, 255),
        //                 2 => Pixel::new(0, 255, 0, 255),
        //                 3 => Pixel::new(0, 0, 255, 255),
        //                 _ => Pixel::new(255, 0, 255, 255)
        //             };
        //         }
        //     }
        // }

        // pixels
    }
}
