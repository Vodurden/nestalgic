#[derive(PartialEq, Eq)]
pub enum Interrupt {
    NMI,
    RESET,
    IRQ,
    BRK,
}

pub const NMI_VECTOR_ADDRESS: u16 = 0xFFFA;
pub const IRQ_VECTOR_ADDRESS: u16 = 0xFFFE;
pub const RESET_VECTOR_ADDRESS: u16 = 0xFFFC;

impl Interrupt {
    pub fn maskable(&self) -> bool {
        // It's unclear whether `RESET` and `BRK` are maskable so we
        // just assume IRQ is
        return *self == Interrupt::IRQ
    }

    pub fn vector_address(&self) -> u16 {
        match self {
            Interrupt::NMI   => NMI_VECTOR_ADDRESS,
            Interrupt::RESET => RESET_VECTOR_ADDRESS,
            Interrupt::IRQ   => IRQ_VECTOR_ADDRESS,
            Interrupt::BRK   => IRQ_VECTOR_ADDRESS,
        }
    }
}
