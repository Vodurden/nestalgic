mod cpu_bus;
mod rp2c02;
mod mapper;

pub use nestalgic_rom::nesrom::NESROM;
use nestalgic_mos6502::mos6502::MOS6502;
use rp2c02::{RP2C02, Pixel};
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

    pub fn new() -> Nestalgic {
        Nestalgic {
            cpu: MOS6502::new(),
            wram: [0; 2048],
            ppu: RP2C02::new(),
            mapper: Box::new(mapper::NullMapper::new()),

            master_clock_speed: Duration::from_secs_f64(1.0 / 21.477272),
            time_since_last_master_cycle: Duration::new(0, 0),
        }
    }

    pub fn with_rom(mut self, rom: NESROM) -> Self {
        self.mapper = Mapper::from_rom(rom);
        let cpu_bus = CpuBus {
            wram: &mut self.wram,
            ppu: &mut self.ppu,
            mapper: &mut *self.mapper
        };
        self.cpu.reset(&cpu_bus);
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
}
