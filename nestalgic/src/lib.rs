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

    pub fn pattern_table_left(&self) -> Texture {
        let chr_data = (0..=0x0FFF)
            .map(|a| self.mapper.ppu_read_u8(a as u16))
            .collect::<Vec<u8>>();

        Texture::from_bitplanes(&chr_data, 16, 128, 128)
    }

    pub fn pattern_table_right(&self) -> Texture {
        let chr_data = (0x1000..=0x1FFF)
            .map(|a| self.mapper.ppu_read_u8(a as u16))
            .collect::<Vec<u8>>();

        Texture::from_bitplanes(&chr_data, 16, 128, 128)
    }
}
