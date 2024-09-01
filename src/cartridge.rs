use std::error::Error;

const NES_TAG: [u8; 4] = [0x4E, 0x45, 0x53, 0x1A];
const PRG_ROM_BANK_SIZE: usize = 16 * 1024;
const CHR_ROM_BANK_SIZE: usize = 8 * 1024;
const HEADER_SIZE: usize = 16;
const TRAINER_SIZE: usize = 16;
const NES_IDENTIFIER_SIZE: usize = 4;
const NUM_PRG_ROM_BANK_POS: usize = 4;
const NUM_CHR_ROM_BANK_POS: usize = 5;
const CONTROL_BYTE1_POS: usize = 6;
const CONTROL_BYTE2_POS: usize = 7;

pub struct Cartridge {
    chr_rom: Vec<u8>,
    prg_rom: Vec<u8>,
    mapper_type: u8,
    mirror_mode: Mirroring
}

pub enum Mirroring {
    HORIZONTAL,
    VERTICAL,
    FOUR_SCREEN
}

impl Cartridge {
    fn new(rom: &Vec<u8>) -> Result<Self, String> {
        if &rom[0..NES_IDENTIFIER_SIZE] != NES_TAG {
            return Err("Not a valid .NES file!".to_string());
        }
        if (&rom[CONTROL_BYTE2_POS] >> 2) & 0b11  != 0 {
            return Err("Only supports iNES1.0.".to_string());
        }

        let is_vertical = rom[CONTROL_BYTE1_POS] & 1 == 1;
        let is_four_screen = (rom[CONTROL_BYTE1_POS] & 0b1000) == 1;
        let mirror_mode = match (is_vertical, is_four_screen) {
            (true, false) => Mirroring::VERTICAL,
            (false, false) => Mirroring::HORIZONTAL,
            (_, true) => Mirroring::FOUR_SCREEN,
        };

        let prg_rom_size = rom[NUM_PRG_ROM_BANK_POS] as usize * PRG_ROM_BANK_SIZE;
        let chr_rom_size = rom[NUM_CHR_ROM_BANK_POS] as usize * CHR_ROM_BANK_SIZE;
        let mapper_type = rom[CONTROL_BYTE2_POS] & 0b1111_0000 | (rom[CONTROL_BYTE1_POS] >> 4);
        let has_trainer = rom[CONTROL_BYTE1_POS] & 0b100 == 1;
        let prg_rom_start = HEADER_SIZE + if has_trainer { TRAINER_SIZE } else { 0 };
        let chr_rom_start = prg_rom_start + prg_rom_size;
        let prg_rom = rom[prg_rom_start..chr_rom_start].to_vec();
        let chr_rom = rom[chr_rom_start..chr_rom_start + chr_rom_size].to_vec();

        Ok(Cartridge {
            chr_rom,
            prg_rom,
            mapper_type,
            mirror_mode,
        })
    }
}
