use nestalgic_mos6502::mos6502::Bus;

use super::WRAM;
use super::rp2c02::RP2C02;
use super::mapper::Mapper;

pub struct CpuBus<'a> {
    pub wram: &'a mut WRAM,
    pub ppu: &'a mut RP2C02,
    pub mapper: &'a mut dyn Mapper,
}

impl <'a> Bus for CpuBus<'a> {
    fn read_u8(&self, address: u16) -> u8 {
        match address {
            0x4020..=0xFFFF => self.mapper.cpu_read_u8(address),
            0x2000..=0x2007 => self.ppu.read_u8(address),
            0x0000..=0x1FFF => self.wram[(address & 0x0800) as usize],
            _ => 0
        }
    }

    fn write_u8(&mut self, address: u16, data: u8) {
        match address {
            0x4020..=0xFFFF => self.mapper.cpu_write_u8(address, data),
            0x2000..=0x2007 => self.ppu.write_u8(address, data),
            0x0000..=0x1FFF => self.wram[(address & 0x0800) as usize] = data,
            _ => ()
        }
    }
}

