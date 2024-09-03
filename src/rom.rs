use std::{
    fs::File,
    io::{Cursor, Write},
};

use crate::{bus::ROM_START, cpu::Mem};

const NES_TAG: [u8; 4] = [0x4E, 0x45, 0x53, 0x1A];
pub const PRG_ROM_BANK_SIZE: usize = 16 * 1024;
pub const CHR_ROM_BANK_SIZE: usize = 8 * 1024;
const HEADER_SIZE: usize = 16;
const TRAINER_SIZE: usize = 16;
const NES_IDENTIFIER_SIZE: usize = 4;
const NUM_PRG_ROM_BANK_POS: usize = 4;
const NUM_CHR_ROM_BANK_POS: usize = 5;
const CONTROL_BYTE1_POS: usize = 6;
const CONTROL_BYTE2_POS: usize = 7;

pub struct Rom {
    chr_rom: Vec<u8>,
    prg_rom: Vec<u8>,
    mapper_type: u8,
    mirror_mode: Mirroring,
}

impl Mem for Rom {
    fn mem_read(&self, addr: u16) -> u8 {
        let rom_relative_addr = addr - ROM_START;
        self.prg_rom[(if rom_relative_addr >= 0x4000 && self.prg_rom.len() == 0x4000 {
            rom_relative_addr % 0x4000
        } else {
            rom_relative_addr
        }) as usize]
    }
}

pub enum Mirroring {
    HORIZONTAL,
    VERTICAL,
    FOUR_SCREEN,
}

impl Rom {
    pub fn new(rom: &Vec<u8>) -> Result<Self, String> {
        if &rom[0..NES_IDENTIFIER_SIZE] != NES_TAG {
            return Err("Not a valid .NES file!".to_string());
        }
        if (&rom[CONTROL_BYTE2_POS] >> 2) & 0b11 != 0 {
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

        Ok(Rom {
            chr_rom,
            prg_rom,
            mapper_type,
            mirror_mode,
        })
    }
}

pub fn insert_new_cartridge(path_to_game: &str) -> Result<Vec<u8>, String> {
    match std::fs::read(format!("{path_to_game}.nes")) {
        Ok(game_bytes) => return Ok(game_bytes),
        Err(_) => return create_cartridge(path_to_game),
    }
}

fn create_cartridge(path_to_game: &str) -> Result<Vec<u8>, String> {
    println!("{path_to_game}.nes not found, creating ROM...");
    let mut buffer = Cursor::new(Vec::new());

    let header = vec![
        0x4E, 0x45, 0x53, 0x1A, 0x02, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00,
    ];
    let pre = [0; 0x600];
    buffer
        .write_all(&header)
        .map_err(|e| format!("Cannot write header: {e}"))?;
    buffer
        .write_all(&pre)
        .map_err(|e| format!("Cannot write pre-padding: {e}"))?;
    let raw_bytes = std::fs::read(path_to_game)
        .map_err(|e| format!("Raw file of {path_to_game} not found: {e}"))?;
    let hex_string = String::from_utf8_lossy(&raw_bytes);
    let cleaned_hexu8s = hex_string
        .trim()
        .replace("\r\n", "");
    let bytes: Vec<u8> = cleaned_hexu8s
        .split(',')
        .filter_map(|s| {
            let trimmed = s.trim(); // Remove whitespace
            if !trimmed.is_empty() {
                u8::from_str_radix(trimmed.trim_start_matches("0x"), 16).ok()
            } else {
                None
            }
        })
        .collect();

    buffer
        .write_all(&bytes)
        .map_err(|e| format!("Cannot write raw bytes: {e}"))?;

    // Calculate the current position and fill the remaining space with zeros
    let pos = 0x600 + bytes.len();
    let filler_size = ((0xFFFC - ROM_START) as usize).saturating_sub(pos);
    buffer
        .write_all(&vec![0; filler_size.into()])
        .map_err(|e| format!("Cannot write filler: {e}"))?;

    // Write the final bytes
    buffer
        .write_all(&[0x0, 0x86, 0, 0])
        .map_err(|e| format!("Cannot write final bytes: {e}"))?;

    let bytes = buffer.into_inner();
    let mut file = File::create(format!("{path_to_game}.nes"))
        .map_err(|e| format!("Cannot create ROM file: {e}"))?;
    file.write_all(&bytes)
        .map_err(|e| format!("Cannot write to ROM file: {e}"))?;
    file.flush()
        .map_err(|e| format!("Error flushing ROM file: {e}"))?;
    println!("{path_to_game}.nes created!");
    Ok(bytes)
}

pub mod test {

    use super::*;

    struct TestRom {
        header: Vec<u8>,
        trainer: Option<Vec<u8>>,
        pgp_rom: Vec<u8>,
        chr_rom: Vec<u8>,
    }

    fn create_rom(rom: TestRom) -> Vec<u8> {
        let mut result = Vec::with_capacity(
            rom.header.len()
                + rom.trainer.as_ref().map_or(0, |t| t.len())
                + rom.pgp_rom.len()
                + rom.chr_rom.len(),
        );

        result.extend(&rom.header);
        if let Some(t) = rom.trainer {
            result.extend(t);
        }
        result.extend(&rom.pgp_rom);
        result.extend(&rom.chr_rom);

        result
    }

    pub fn test_rom() -> Rom {
        let test_rom = create_rom(TestRom {
            header: vec![
                0x4E, 0x45, 0x53, 0x1A, 0x02, 0x01, 0x31, 00, 00, 00, 00, 00, 00, 00, 00, 00,
            ],
            trainer: None,
            pgp_rom: vec![1; 2 * PRG_ROM_BANK_SIZE],
            chr_rom: vec![2; 1 * CHR_ROM_BANK_SIZE],
        });

        Rom::new(&test_rom).unwrap()
    }
}
