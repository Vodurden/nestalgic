use nestalgic_mos6502::MOS6502;
pub(crate) use nestalgic_mos6502::mos6502::Bus;

use crate::cartridge::Cartridge;
use crate::rp2c02::PPUMask;

use super::WRAM;
use super::rp2c02::RP2C02;


// pub struct NesBus<'a> {
//     pub wram: &'a mut WRAM,
//     pub ppu: &'a mut RP2C02,
//     pub cartridge: &'a mut Cartridge,
// }

// pub struct PpuBus<'a> {
//     pub cartridge: &'a mut Cartridge
// }

// impl <'a> Bus for NesBus<'a> {
//     fn read_u8(&mut self, address: u16) -> u8 {
//         match address {
//             0x4020..=0xFFFF  => self.cartridge.mapper.cpu_read_u8(address),
//             0x2000 => self.ppu.ppuctrl.0,
//             0x2007 => {
//                 self.ppu.read_ppudata(self)
//             },
//             // TODO 0x2001..=2007
//             0x0000..=0x1FFF  => self.wram[(address & 0x0800) as usize],
//             _ => 0
//         }
//     }

//     fn write_u8(&mut self, address: u16, data: u8) {
//         todo!()
//     }
// }

// pub fn new_nes_bus<'a>(
//     wram: &'a mut WRAM,
//     ppu: &'a mut RP2C02,
//     cartridge: &'a mut Cartridge
// ) -> NesBus<'a> {
//     NesBus {
//         wram,
//         ppu,
//         cartridge
//     }
// }

// impl <'a, T> NesBus<'a, T> {
//     pub fn as_cpu_bus<'b>(&'b mut self) -> CpuBus<'b> {
//         CpuBus {
//             bus_type: PhantomData,
//             wram: self.wram,
//             ppu: self.ppu,
//             cartridge: self.cartridge
//         }
//     }

//     pub fn as_ppu_bus<'b>(&'b mut self) -> PpuBus<'b> {
//         PpuBus {
//             bus_type: PhantomData,
//             wram: self.wram,
//             ppu: self.ppu,
//             cartridge: self.cartridge
//         }
//         // self.bus_type = PhantomData as PhantomData<PpuBusType>;
//         // let NesBus { bus_type: _ } = self
//         // self as PpuBus
//     }
// }

// struct CpuBusType;
// pub type CpuBus<'a> = NesBus<'a, CpuBusType>;

// struct PpuBusType;
// pub type PpuBus<'a> = NesBus<'a, PpuBusType>;

// impl Bus for CpuBus<'_> {
//     fn read_u8(&self, address: u16) -> u8 {
//         match address {
//             0x4020..=0xFFFF  => self.cartridge.mapper.cpu_read_u8(address),
//             0x2000 => self.ppu.ppuctrl.0,
//             0x2007 => {
//                 let ppu_bus = self.as_ppu_bus();
//                 let ppu = self.ppu;
//                 ppu.read_ppudata(&mut ppu_bus)
//             },
//             // TODO 0x2001..=2007
//             0x0000..=0x1FFF  => self.wram[(address & 0x0800) as usize],
//             _ => 0
//         }
//     }

//     fn write_u8(&mut self, address: u16, data: u8) {
//         let mut ppu_bus = self.as_ppu_bus();
//         let ppu = self.ppu;
//         let val = ppu.read_ppudata(&mut ppu_bus);

//         todo!()
//     }
// }

// impl <'a> Bus for PpuBus<'a> {
//     fn read_u8(&self, address: u16) -> u8 {
//         todo!()
//     }

//     fn write_u8(&mut self, address: u16, data: u8) {
//         todo!()
//     }
// }

pub struct CpuBus<'a> {
    pub wram: &'a mut WRAM,
    pub ppu: &'a mut RP2C02,
    pub cartridge: &'a mut Cartridge,
}

impl <'a> Bus for CpuBus<'a> {
    fn read_u8(&mut self, address: u16) -> u8 {
        match address {
            0x4020..=0xFFFF => self.cartridge.mapper.cpu_read_u8(address),
            0x2000..=0x3FFF => {
                let mut ppu_bus = PpuBus { cartridge: self.cartridge };
                let value = self.ppu.cpu_mapped_read_u8(&mut ppu_bus, address);
                value
            },
            0x0000..=0x1FFF  => self.wram[(address & 0x07FF) as usize],
            _ => 0
        }
    }

    fn write_u8(&mut self, address: u16, data: u8) {
        match address {
            0x4020..=0xFFFF => self.cartridge.mapper.cpu_write_u8(address, data),
            0x2000..=0x3FFF => {
                let mut ppu_bus = PpuBus { cartridge: self.cartridge };
                self.ppu.cpu_mapped_write_u8(&mut ppu_bus, address, data)
            },
            0x0000..=0x1FFF => self.wram[(address & 0x07FF) as usize] = data,
            _ => ()
        }
    }
}

pub struct PpuBus<'a> {
    pub cartridge: &'a mut Cartridge
}

impl <'a> Bus for PpuBus<'a> {
    fn read_u8(&mut self, address: u16) -> u8 {
        // TODO
        0
    }

    fn write_u8(&mut self, address: u16, data: u8) {
        // TODO
    }
}
