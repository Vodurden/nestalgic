use super::{Address, CyclesTaken, Result, MOS6502, Register};
use super::bus::Bus;
use super::addressing_mode::Addressing;
use super::error::Error;
use super::status::StatusFlag;

#[derive(PartialEq, Eq, Debug)]
pub struct Addressable {
    pub addressing: Addressing,

    /// The value, memory or register targeted by this addressable.
    pub target: AddressableTarget,

    /// True if the addressing crossed a page boundary when calculating this addressable.
    pub page_boundary_crossed: bool,
}

/// An `Adderssable` is a fully realized `Addressing` that can be used to read, write and modify registers and memory
/// targeted by an `Addressing`
///
/// An addressing can be used to read, modify and write across all values targetable by `AdressingMode` (Memory, Accumulator, Immediate)
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum AddressableTarget {
    Accumulator,
    Immediate(u8),
    Memory(u16),
}

impl Addressable {
    pub fn address(&self) -> Result<Address> {
        let address = match self.target {
            AddressableTarget::Accumulator => Err(Error::InvalidAddressAttempt(self.target)),
            AddressableTarget::Immediate(_) => Err(Error::InvalidAddressAttempt(self.target)),
            AddressableTarget::Memory(address) => Ok(address),
        }?;

        Ok(address)
    }

    pub fn read<B: Bus>(&self, cpu: &MOS6502<B>) -> (u8, CyclesTaken) {
        match self.target {
            AddressableTarget::Accumulator => (cpu.a, 0),
            AddressableTarget::Immediate(value) => (value, 0),
            AddressableTarget::Memory(address) => {
                let value = cpu.bus.read_u8(address);
                let mut cycles_taken = 1; // read bus: +1 cycle

                // If the page boundary was crossed the 6502 re-reads the memory location after
                // swapping the page. This costs a cycle
                if self.page_boundary_crossed {
                    cycles_taken += 1;
                }

                (value, cycles_taken)
            }
        }
    }

    pub fn try_write<B: Bus>(&self, cpu: &mut MOS6502<B>, value: u8) -> Result<CyclesTaken> {
        match self.target {
            AddressableTarget::Immediate(_) => Err(Error::InvalidAddressableWrite(self.target, value)),
            AddressableTarget::Accumulator => {
                cpu.write_register(Register::A, value);
                Ok(0)
            }
            AddressableTarget::Memory(address) => {
                cpu.bus.write_u8(address, value);

                let indirection_cycles = match self.addressing {
                    Addressing::AbsoluteX(_) => 1,
                    Addressing::AbsoluteY(_) => 1,
                    Addressing::IndirectIndexed(_) => 1,
                    _ => 0
                };

                Ok(indirection_cycles + 1)
            }
        }
    }

    pub fn try_modify<B: Bus>(&self, cpu: &mut MOS6502<B>, f: impl FnOnce(u8) -> u8) -> Result<(u8, u8, CyclesTaken)> {
        // At first glance you might think we can write this function in terms of `read` and `try_write`.
        //
        // Think again!
        //
        // The cycle behavior of the 6502 has subtle differences when reading/writing vs. modifying. I.e:
        //
        // - When reading we need to add 1 cycle if we cross a page boundary. `modify` should always do this
        // - When writing some addressing modes do an extra `read`. The same addressing modes do not do the extra
        //   read when modifying
        //
        // Let's save ourselves the pain and just implement modify directly.
        let (input, output, modify_cycles) = match self.target {
            AddressableTarget::Immediate(_) => Err(Error::InvalidAddressableModify(self.target)),

            AddressableTarget::Accumulator => {
                let input = cpu.a;
                let output = f(cpu.a);
                cpu.write_register(Register::A, output);

                Ok((input, output, 0))
            },

            AddressableTarget::Memory(address) => {
                let input = cpu.bus.read_u8(address);
                let mut read_cycles = 1; // read bus: +1 cycle

                // For `AbsoluteX`, `AbsoluteY` and `IndirectIndexed` the 6502 reads the value twice before
                // performing the operations, incurring an extra cycle cost
                match self.addressing {
                    Addressing::AbsoluteX(_) => read_cycles += 1,
                    Addressing::AbsoluteY(_) => read_cycles += 1,
                    Addressing::IndirectIndexed(_) => read_cycles += 1,
                    _ => {},
                };

                let output = f(input);

                cpu.bus.write_u8(address, output);
                let mut write_cycles = 1; // write bus: +1 cycle

                // The 6502 actually writes the result twice when modifying, incurring an extra cycle
                cpu.bus.write_u8(address, output);
                write_cycles += 1; // write bus: +1 cycle

                Ok((input, output, read_cycles + write_cycles))
            },
        }?;

        // When doing a `modify` we affect `Zero` and `Negative` even when
        // writing to memory
        cpu.p.set(StatusFlag::Zero, output == 0);
        cpu.p.set(StatusFlag::Negative, output & 0b1000_0000 > 0);

        Ok((input, output, modify_cycles))
    }
}
