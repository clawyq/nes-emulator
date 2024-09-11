use crate::{cpu::Mem, ppu::PPU, rom::Rom};
const RAM: u16 = 0x0000;
const RAM_MIRRORS_END: u16 = 0x1FFF;
const PPU_REGISTERS: u16 = 0x2000;
const PPU_REGISTERS_MIRRORS_END: u16 = 0x3FFF;
pub const ROM_START: u16 = 0x8000;

pub struct Bus {
    vram: [u8; 2048],
    ppu: PPU,
    prg_rom: Vec<u8>,
}

enum BusDevice {
    CPU,
    PPU,
}

impl BusDevice {
    fn mirror_addr(&self, addr: u16) -> u16 {
        match self {
            BusDevice::CPU => addr & 0x7FF,
            BusDevice::PPU => addr & 0x2007,
        }
    }
}

impl Bus {
    pub fn new(rom: Rom) -> Self {
        let ppu = PPU::new(rom.chr_rom, rom.mirror_mode);
        Bus {
            vram: [0; 2048],
            ppu,
            prg_rom: rom.prg_rom,
        }
    }

    fn prg_read(&self, addr: u16) -> u8 {
        let rom_relative_addr = addr - ROM_START;
        self.prg_rom[(if rom_relative_addr >= 0x4000 && self.prg_rom.len() == 0x4000 {
            rom_relative_addr % 0x4000
        } else {
            rom_relative_addr
        }) as usize]
    }
}

impl Mem for Bus {
    fn mem_read(&mut self, addr: u16) -> u8 {
        match addr {
            RAM..=RAM_MIRRORS_END => self.vram[BusDevice::CPU.mirror_addr(addr) as usize],
            PPU_REGISTERS..=PPU_REGISTERS_MIRRORS_END => {
                self.ppu.mem_read(BusDevice::PPU.mirror_addr(addr))
            },
            ROM_START..=0xFFFF => self.prg_read(addr),
            _ => {
                println!("{}", format!("Out of range: {}", addr));
                0
            }
        }
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        match addr {
            RAM..=RAM_MIRRORS_END => self.vram[BusDevice::CPU.mirror_addr(addr) as usize] = data,
            PPU_REGISTERS..=PPU_REGISTERS_MIRRORS_END => {
                self.ppu.mem_write(BusDevice::PPU.mirror_addr(addr), data)
            }
            ROM_START..=0xFFFF => panic!(
                "{}",
                format!("Invalid request to write to ROM PRG: {}", addr)
            ),
            _ => println!("{}", format!("Out of range: {}", addr)),
        }
    }
}
