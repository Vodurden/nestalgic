use super::{NMI_VECTOR_ADDRESS, INITIALIZATION_VECTOR_ADDRESS};

pub trait Bus {
    fn write_u8(&mut self, address: u16, data: u8);

    fn read_u8(&self, address: u16) -> u8;

    /// Read a `u16` from the bus from `address`. Assumes the values are in _little endian_ order.
    fn read_u16(&self, address: u16) -> u16 {
        let lo = self.read_u8(address);
        let hi = self.read_u8(address.wrapping_add(1));
        u16::from_le_bytes([lo, hi])
    }

    /// Write a `u16` to the bus starting at `address` in _little endian_ order.
    fn write_u16(&mut self, address: u16, value: u16) {
        let [lo, hi] = value.to_le_bytes();
        self.write_u8(address, lo);
        self.write_u8(address.wrapping_add(1), hi);
    }

    fn read_range(&self, start: u16, end: u16) -> Vec<u8> {
        (start..end)
            .map(|a| self.read_u8(a))
            .collect()
    }
}

/// A Bus used for testing. It stores the program in an expected location
///
/// We use `RamBus16k` for testing.
pub struct RamBus16kb {
    pub memory: [u8; RamBus16kb::SIZE],
}

impl RamBus16kb {
    /// If we have a 16-bit addressing scheme then we can address
    /// _65536_ bytes of memory in total.
    pub const SIZE: usize = 65536;

    pub fn new() -> RamBus16kb {
        RamBus16kb {
            memory: [0; RamBus16kb::SIZE]
        }
    }

    pub fn with_nmi_vector_address(mut self, address: u16) -> RamBus16kb {
        self.write_u16(NMI_VECTOR_ADDRESS, address);
        self
    }

    pub fn with_program(mut self, bytes: Vec<u8>) -> RamBus16kb {
        let program_end = NMI_VECTOR_ADDRESS as usize;
        let program_start = program_end - bytes.len();
        self.memory[program_start..program_end].copy_from_slice(&bytes[..]);

        // Set the initialization vector to point at our program.
        self.write_u16(INITIALIZATION_VECTOR_ADDRESS, program_start as u16);

        self
    }

    /// Writes memory into RAM starting from address `0x0000`
    pub fn with_memory(self, bytes: Vec<u8>) -> RamBus16kb {
        self.with_memory_at(0, bytes)
    }

    pub fn with_memory_at(mut self, start: usize, bytes: Vec<u8>) -> RamBus16kb {
        let program_start = start as usize;
        let program_end = start + bytes.len() as usize;
        self.memory[program_start..program_end].copy_from_slice(&bytes[..]);
        self
    }
}

impl Bus for RamBus16kb {
    fn write_u8(&mut self, address: u16, data: u8) {
        self.memory[address as usize] = data;
    }

    fn read_u8(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }
}

/// Tests for `Bus`
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn read_u16_is_little_endian() {
        let mut bus = RamBus16kb::new();
        bus.memory[0xAAAA] = 0x01;
        bus.memory[0xAAAB] = 0xFF;

        let result = bus.read_u16(0xAAAA);
        assert_eq!(result, 0xFF01);
    }

    #[test]
    pub fn write_u16_is_little_endian() {
        let mut bus = RamBus16kb::new();
        bus.write_u16(0xAAAA, 0xFF01);

        assert_eq!(bus.memory[0xAAAA], 0x01);
        assert_eq!(bus.memory[0xAAAB], 0xFF);
    }

    /// Assuming we have a read/write device connected to the bus we should
    /// expect that `write_u16(m, a)` followed by `read_u16(m)` should be `a`
    #[test]
    pub fn write_u16_read_u16_roundtrip() {
        let mut bus = RamBus16kb::new();
        bus.write_u16(0xBBAA, 0xBEEF);

        let result = bus.read_u16(0xBBAA);
        assert_eq!(result, 0xBEEF);
    }
}

/// Tests for `RamBus16kb`
#[cfg(test)]
mod rambus_tests {
    use super::*;

    /// Generally program rom is placed at the end of addressable memory. But we need to leave
    /// some space for the interrupt vectors used by the 6502.
    ///
    /// We want to make sure `with_program` puts the program data as far towards the end as possible
    /// within these constraints.
    #[test]
    pub fn with_program_inserts_at_end() {
        let bus = RamBus16kb::new()
            .with_program(vec![0xAA, 0xBB, 0xCC]);

        assert_eq!(bus.memory[0xFFF7], 0xAA);
        assert_eq!(bus.memory[0xFFF8], 0xBB);
        assert_eq!(bus.memory[0xFFF9], 0xCC);
    }

    /// `with_program` writes as closely as possible to `INITIALIZATION_VECTOR_ADDRESS`. We want to make sure
    /// it doesn't write _too_ close and clobber the data.
    #[test]
    pub fn with_program_doesnt_override_nmi_vector_address() {
        let bus = RamBus16kb::new()
            .with_nmi_vector_address(0xBEEF)
            .with_program(vec![0x01, 0x02, 0x03]);

        let nmi_address = NMI_VECTOR_ADDRESS as usize;
        assert_eq!(bus.memory[nmi_address    ], 0xEF);
        assert_eq!(bus.memory[nmi_address + 1], 0xBE);
    }

    /// `with_program` should write the first instruction to `INITIALIZATION_VECTOR_ADDRESS` (0xFFFC) since that's
    /// what the 6502 expects.
    #[test]
    pub fn with_program_writes_instruction_to_initialization_vector() {
        let bus = RamBus16kb::new()
            .with_program(vec![0xAA, 0xBB, 0xCC]);

        // Remember: addresses are in little-endian so if we expect the address `0xFFF7` then
        // we check for the byte `0xED` _followed by_ `0xFF`.
        let iv_address = INITIALIZATION_VECTOR_ADDRESS as usize;
        assert_eq!(bus.memory[iv_address    ], 0xF7);
        assert_eq!(bus.memory[iv_address + 1], 0xFF);
    }
}
