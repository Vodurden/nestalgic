mod nrom;

pub use nrom::NROM;
use nestalgic_rom::nesrom::NESROM;

/// A mapper is hardware found on the NES cartridge that maps the addresses on the cartridge
/// to the physical hardware.
///
/// All mapper functions accept the entire address space but are only defined
/// within the address `0x4020` - `0xFFFF`. Attempting to read or write outside
/// this address range will result in a panic
pub trait Mapper {
    fn cpu_read_u8(&self, address: u16) -> u8;

    fn cpu_write_u8(&mut self, address: u16, data: u8);

    fn ppu_read_u8(&self, address: u16) -> u8;

    fn ppu_write_u8(&mut self, address: u16, data: u8);
}

impl dyn Mapper {
    pub fn from_rom(rom: NESROM) -> Box<dyn Mapper> {
        match rom.header.mapper_number {
            0 => Box::new(NROM::from_rom(rom)),
            _ => panic!("unsupported mapper number: {}", rom.header.mapper_number)
        }
    }
}

pub struct NullMapper {}

impl NullMapper {
    pub fn new() -> NullMapper {
        NullMapper {}
    }
}

impl Mapper for NullMapper {
    fn cpu_read_u8(&self, _address: u16) -> u8 {
        0
    }
    fn cpu_write_u8(&mut self, _address: u16, _data: u8) {
    }
    fn ppu_read_u8(&self, _address: u16) -> u8 {
        0
    }
    fn ppu_write_u8(&mut self, _address: u16, _data: u8) {
    }
}
