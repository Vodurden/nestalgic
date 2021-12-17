use nestalgic_rom::nesrom::NESROM;
use super::Mapper;

pub struct NROM {
    /// In NROM-256 the `prg_rom` is 32kb, for NROM-128 the `prg_rom` is only 16kb and will be
    /// repeated to fill the remaining 16kb.
    ///
    /// Address space: `0x8000`-`0xBFFF` (First 16kb)
    pub prg_rom_bank_1: [u8; 16 * 1024],

    /// Address Space: `0xC000`-`0xFFFF` (Last 16kb or mirror of first 16kb)
    pub prg_rom_bank_2: [u8; 16 * 1024],

    /// 2kb mirrored 4 times
    ///
    /// Address space:
    ///
    /// - `0x6000`-`0x7FFF`
    ///
    pub prg_ram: [u8; 2048],

    pub chr_ram: [u8; 8 * 1024],

    pub nametable_1: [u8; 1024],
    pub nametable_2: [u8; 1024],
}

impl NROM {
    pub fn empty() -> NROM {
        NROM {
            prg_rom_bank_1: [0; 16 * 1024],
            prg_rom_bank_2: [0; 16 * 1024],
            prg_ram: [0; 2048],
            chr_ram: [0; 8 * 1024],
            nametable_1: [0; 1024],
            nametable_2: [0; 1024]
        }
    }

    pub fn from_rom(rom: &NESROM) -> NROM {
        let mut nrom = NROM::empty();

        if rom.prg_rom.len() <= 16 * 1024 {
            nrom.prg_rom_bank_1[0..rom.prg_rom.len()].copy_from_slice(&rom.prg_rom[..]);
            nrom.prg_rom_bank_2[0..rom.prg_rom.len()].copy_from_slice(&rom.prg_rom[..]);
        } else {
            nrom.prg_rom_bank_1[0..16 * 1024].copy_from_slice(&rom.prg_rom[0..16 * 1024]);
            nrom.prg_rom_bank_2[0..16 * 1024].copy_from_slice(&rom.prg_rom[16 * 1024..rom.prg_rom.len()]);
        };

        // TODO: Support bigger chr_ram
        nrom.chr_ram.copy_from_slice(&rom.chr_rom[0..8 * 1024]);

        nrom
    }
}

impl Mapper for NROM {
    fn cpu_read_u8(&self, address: u16) -> u8 {
        match address {
            0x8000..=0xBFFF => self.prg_rom_bank_1[address as usize - 0x8000],
            0xC000..=0xFFFF => self.prg_rom_bank_2[address as usize - 0xC000],
            0x6000..=0x7FFF => self.prg_ram[address as usize - 0x6000],
            _ => panic!("attempt to cpu_read from unmapped address {:04X}", address)
        }
    }

    fn cpu_write_u8(&mut self, address: u16, data: u8) {
        match address {
            0x6000..=0x7FFF => self.prg_ram[address as usize - 0x6000] = data,
            0x8000..=0xFFFF => {},
            _ => panic!("attempt to cpu_write to unmapped address {:04X}", address)
        }
    }

    fn ppu_read_u8(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x1FFF => self.chr_ram[address as usize],
            0x2000..=0x23FF => self.nametable_1[address as usize],
            0x2400..=0x27FF => self.nametable_2[address as usize],
            0x2800..=0x2BFF => self.nametable_1[address as usize],
            0x2C00..=0x2FFF => self.nametable_2[address as usize],
            _ => panic!("attempt to ppu_read from unmapped address {:04X}", address)
        }
    }

    fn ppu_write_u8(&mut self, address: u16, data: u8) {
        match address {
            0x0000..=0x1FFF => self.chr_ram[address as usize] = data,
            0x2000..=0x23FF => self.nametable_1[address as usize] = data,
            0x2400..=0x27FF => self.nametable_2[address as usize] = data,
            0x2800..=0x2BFF => self.nametable_1[address as usize] = data,
            0x2C00..=0x2FFF => self.nametable_2[address as usize] = data,
            _ => panic!("attempt to ppu_read from unmapped address {:04X}", address)
        }
    }
}
