#[derive(PartialEq, Debug)]
pub enum MirroringType {
    Horizontal,
    Vertical,
    FourScreen,
}

impl MirroringType {
    pub fn from_ines_byte_6(byte: u8) -> MirroringType {
        let mirror_bit = byte & 0b0000_0001 != 0;
        let four_screen_vram_bit = (byte & 0b0000_1000) >> 3 != 0;

        match (mirror_bit, four_screen_vram_bit) {
            (_    , true) => MirroringType::FourScreen,
            (false, _)    => MirroringType::Horizontal,
            (true , _)    => MirroringType::Vertical,
        }
    }
}
