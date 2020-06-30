use nestalgic_rom::nesrom::{self, NESROM};

#[test]
fn load_nestest_with_expected_header() {
    let rom_file = include_bytes!("./fixtures/nestest.nes").to_vec();
    let rom = NESROM::from_bytes(rom_file);
    let header = rom.map(|r| r.header);

    let expected_header = nesrom::Header {
        file_type: nesrom::FileType::INES,
        prg_rom_bytes: 16384,
        chr_rom_bytes: 8192,
        mirroring_type: nesrom::MirroringType::Horizontal,
        has_persistent_memory: false,
        has_trainer: false,
        mapper_number: 0,
    };

    assert_eq!(header, Ok(expected_header));
}
