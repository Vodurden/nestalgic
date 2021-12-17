mod nrom;
mod mapper;

use mapper::Mapper;
pub use nrom::NROM;
use nestalgic_rom::nesrom::NESROM;

pub struct Cartridge {
    pub rom: NESROM,
    pub mapper: Box<dyn Mapper>
}

impl Cartridge {
    pub fn from_rom(rom: NESROM) -> Cartridge {
        let mapper = <dyn Mapper>::for_rom(&rom);
        Cartridge {
            rom,
            mapper
        }
    }
}
