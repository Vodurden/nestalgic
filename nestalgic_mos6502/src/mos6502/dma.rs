/// The MOS6502 doesn't directly support DMA, but it's common for systems using a 6502
/// to need DMA capability.
#[derive(Debug)]
pub struct DMA {
    /// Trigger this DMA when this address is written to on the CPU bus.
    pub trigger_address: u16,

    /// Write each byte to this address on the CPU bus.
    pub target_address: u16,

    /// The number of bytes to write to `target_address`.
    pub bytes_to_transfer: u16,
}

#[derive(Clone, Debug)]
pub struct ActiveDMA {
    pub start_address: u16,

    pub target_address: u16,

    pub bytes_to_transfer: u16,

    pub bytes_transferred: u16,
}

impl ActiveDMA {
    pub fn from_dma(dma: &DMA, start_address: u16) -> ActiveDMA {
        ActiveDMA {
            start_address,
            target_address: dma.target_address,
            bytes_to_transfer: dma.bytes_to_transfer,
            bytes_transferred: 0,
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum DMAStatus {
    Active,
    Inactive
}
