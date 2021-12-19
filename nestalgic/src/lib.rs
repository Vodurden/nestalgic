mod nes_bus;
mod rp2c02;
mod cartridge;

use cartridge::Cartridge;
use nes_bus::{CpuBus, PpuBus};
pub use nestalgic_rom::nesrom::NESROM;
pub use rp2c02::{Texture, Pixel};
use nestalgic_mos6502::mos6502::{MOS6502, DMA};
use rp2c02::RP2C02;

use std::time::Duration;

type WRAM = [u8; 2048];

pub struct Nestalgic {
    pub cpu: MOS6502,
    pub ppu: RP2C02,

    wram: WRAM,
    cartridge: Cartridge,
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

    pub fn new(rom: NESROM) -> Nestalgic {
        let mut nestalgic = Nestalgic {
            cpu: Nestalgic::nes_cpu(),
            wram: [0; 2048],
            ppu: RP2C02::new(),
            cartridge: Cartridge::from_rom(rom),

            master_clock_speed: Duration::from_nanos(559),
            time_since_last_master_cycle: Duration::new(0, 0),
        };
        nestalgic.reset();
        nestalgic
    }

    fn nes_cpu() -> MOS6502 {
        let nes_dma = DMA {
            trigger_address: 0x4014,
            target_address: 0x2004,
            bytes_to_transfer: 256,
        };

        MOS6502::new().with_dma(nes_dma)
    }

    pub fn reset(&mut self) {
        let mut cpu_bus = CpuBus {
            wram: &mut self.wram,
            ppu: &mut self.ppu,
            cartridge: &mut self.cartridge
        };
        self.cpu.reset(&mut cpu_bus).expect("Failed to reset CPU");
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
            cartridge: &mut self.cartridge
        };
        self.cpu.cycle(&mut cpu_bus).expect("failed to cycle cpu");

        let mut ppu_bus = PpuBus {
            cartridge: &mut self.cartridge
        };
        self.ppu.cycle(&mut self.cpu, &mut ppu_bus);
        self.ppu.cycle(&mut self.cpu, &mut ppu_bus);
        self.ppu.cycle(&mut self.cpu, &mut ppu_bus);
    }

    pub fn pixels(&self) -> &[Pixel; Nestalgic::SCREEN_PIXELS] {
        &self.ppu.pixels
    }

    pub fn pattern_table_left(&self) -> Texture {
        let chr_data = (0..=0x0FFF)
            .map(|a| self.cartridge.mapper.ppu_read_u8(a as u16))
            .collect::<Vec<u8>>();

        Texture::from_bitplanes(&chr_data, 16, 128, 128)
    }

    pub fn pattern_table_right(&self) -> Texture {
        let chr_data = (0x1000..=0x1FFF)
            .map(|a| self.cartridge.mapper.ppu_read_u8(a as u16))
            .collect::<Vec<u8>>();

        Texture::from_bitplanes(&chr_data, 16, 128, 128)
    }
}
