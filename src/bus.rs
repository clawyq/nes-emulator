use crate::{cpu::Mem, rom::Rom};

const RAM: u16 = 0x0000;
const RAM_MIRRORS_END: u16 = 0x1FFF;
const PPU_REGISTERS: u16 = 0x2000;
const PPU_REGISTERS_MIRRORS_END: u16 = 0x3FFF;
pub const ROM_START: u16 = 0x8000;

pub struct Bus {
    vram: [u8; 2048],
    rom: Rom,
}

enum BusDevice {
    CPU,
    PPU,
}

impl BusDevice {
    fn mirror_addr(&self, addr: u16) -> usize {
        match self {
            BusDevice::CPU => (addr & 0x7FF) as usize,
            BusDevice::PPU => (addr & 0x2007) as usize,
        }
    }
}

impl Bus {
    pub fn new(rom: Rom) -> Self {
        Bus {
            vram: [0; 2048],
            rom,
        }
    }
}

impl Mem for Bus {
    fn mem_read(&self, addr: u16) -> u8 {
        match addr {
            RAM..=RAM_MIRRORS_END => self.vram[BusDevice::CPU.mirror_addr(addr)],
            PPU_REGISTERS..=PPU_REGISTERS_MIRRORS_END => 0, //todo!("BusDevice::CPU.mirror_addr(addr)"),
            ROM_START..=0xFFFF => self.rom.mem_read(addr),
            _ => {
                println!("{}", format!("Out of range: {}", addr));
                0
            }
        }
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        match addr {
            RAM..=RAM_MIRRORS_END => self.vram[BusDevice::CPU.mirror_addr(addr)] = data,
            PPU_REGISTERS..=PPU_REGISTERS_MIRRORS_END => todo!("BusDevice::CPU.mirror_addr(addr)"),
            ROM_START..=0xFFFF => panic!(
                "{}",
                format!("Invalid request to write to ROM PRG: {}", addr)
            ),
            _ => println!("{}", format!("Out of range: {}", addr)),
        }
    }
}
