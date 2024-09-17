mod registers;

use crate::{cpu::Mem, rom::Mirroring};
use registers::{
    address::AddressRegister, control::ControlRegister, mask::MaskRegister, oam::Oam,
    scroll::ScrollRegister, status::StatusRegister,
};

// KEY ADDRESSES
pub const NAME_TABLE_SIZE: u16 = 0x400;
pub const CHR_ROM_END_ADDR: u16 = 0x1FFF;
pub const NAME_TABLE_START_ADDR: u16 = 0x2000;
pub const NAME_TABLE_END_ADDR: u16 = 0x2FFF;
pub const PALETTE_START_ADDR: u16 = 0x3f00;
pub const BEFORE_MIRROR_RANGE: u16 = 0x3FFF;

// CLOCK
const SCAN_LINES_PER_FRAME: u16 = 262;
const CLOCK_CYCLES_PER_SCAN_LINE: usize  = 341;
const SCAN_LINE_INTERRUPT: u16  = 241;

pub struct PPU {
    pub chr_rom: Vec<u8>,
    pub mirror_mode: Mirroring,
    pub palette_table: [u8; 32],
    pub vram: [u8; 2048],
    control: ControlRegister,
    addr: AddressRegister,
    scroll: ScrollRegister,
    status: StatusRegister,
    mask: MaskRegister,
    oam: Oam,
    data_buffer: u8,
    scan_line: u16,
    cycles: usize,
    nmi: Option<bool>
}

impl Mem for PPU {
    fn mem_read(&mut self, addr: u16) -> u8 {
        match addr {
            0x2000 | 0x2001 | 0x2003 | 0x2005 | 0x2006 | 0x4014 => {
                panic!("Attempt to read from write-only PPU address {:x}", addr);
                // 0
            }
            0x2002 => self.read_status(),
            0x2004 => self.oam.read_data(),
            0x2007 => self.read_ppu_data(),
            _ => {
                println!("{}", format!("PPU read attempt out of range: {}", addr));
                0
            }
        }
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        match addr {
            0x2000 => {
                self.nmi = self.control.update(data, self.status.is_in_vblank());
            },
            0x2001 => self.mask.update(data),
            0x2002 => panic!("Attempted to write to PPU status register >:("),
            0x2003 => self.oam.write_addr(data),
            0x2004 => self.oam.write_data(data),
            0x2005 => self.scroll.write(data),
            0x2006 => self.addr.write(data),
            0x2007 => self.write_ppu_data(data),
            0x4014 => todo!("oam dma"),
            _ => panic!("dafk bro"),
        }
    }
}

impl PPU {
    pub fn new(chr_rom: Vec<u8>, mirror_mode: Mirroring) -> Self {
        PPU {
            chr_rom,
            mirror_mode,
            palette_table: [0; 32],
            vram: [0; 2048],
            control: ControlRegister::new(),
            addr: AddressRegister::new(),
            scroll: ScrollRegister::new(),
            status: StatusRegister::new(),
            mask: MaskRegister::new(),
            oam: Oam::new(),
            data_buffer: 0,
            scan_line: 0,
            cycles: 0,
            nmi: None
        }
    }

    pub fn tick(&mut self, cycles: u8) -> bool {
        self.cycles += cycles as usize;
        if self.cycles >= CLOCK_CYCLES_PER_SCAN_LINE {
            self.cycles = self.cycles - CLOCK_CYCLES_PER_SCAN_LINE;
            self.scan_line += 1;

            if self.scan_line == SCAN_LINE_INTERRUPT {
                self.status.set_vblank(true);
                self.status.set_sprite_zero_hit(false);
                if self.control.generate_nmi() {
                    self.nmi = Some(true);
                }
            }

            if self.scan_line >= SCAN_LINES_PER_FRAME {
                self.scan_line = 0;
                self.nmi = None;
                self.status.set_sprite_zero_hit(false);
                self.status.reset_vblank();
                return true;
            }
        }
        return false;
    }
    
    pub fn poll_nmi(&mut self) -> Option<bool> {
        self.nmi.take()
    }

    fn read_status(&mut self) -> u8 {
        let status = self.status.bits();
        self.status.reset_vblank();
        self.addr.reset_latch();
        self.scroll.reset_latch();
        status
    }

    fn read_ppu_data(&mut self) -> u8 {
        let ppu_addr = self.addr.get();
        self.increment_vram_ptr();
        match ppu_addr {
            0..=CHR_ROM_END_ADDR => {
                let data = self.data_buffer;
                self.data_buffer = self.chr_rom[ppu_addr as usize];
                data
            }
            NAME_TABLE_START_ADDR..=NAME_TABLE_END_ADDR => {
                let data = self.data_buffer;
                self.data_buffer = self.vram[self.mirror_vram(ppu_addr) as usize];
                data
            }
            0x3000..=0x3eff => panic!(
                "ppu_addr space 0x3000 - 0x3eff is not expected to be used, requested = {} ",
                ppu_addr
            ),
            0x3f10 | 0x3f14 | 0x3f18 | 0x3f1c => {
                let addr_mirror = ppu_addr - 0x10;
                self.palette_table[(addr_mirror - PALETTE_START_ADDR) as usize]
            }
            PALETTE_START_ADDR..=BEFORE_MIRROR_RANGE => {
                self.palette_table[(ppu_addr - PALETTE_START_ADDR) as usize]
            }
            _ => panic!("unexpected access to mirrored space {}", ppu_addr),
        }
    }

    fn write_ppu_data(&mut self, data: u8) {
        let ppu_addr = self.addr.get();
        self.increment_vram_ptr();

        match ppu_addr {
            0..=0x1fff => println!("Attempt to write to chr rom: {}", ppu_addr),
            0x2000..=0x2fff => {
                self.vram[self.mirror_vram(ppu_addr) as usize] = data;
            }
            0x3000..=0x3eff => unimplemented!(
                "ppu_ppu_addr space 0x3000 - 0x3eff is not expected to be used, requested = {} ",
                ppu_addr
            ),

            //ppu_addresses $3F10/$3F14/$3F18/$3F1C are mirrors of $3F00/$3F04/$3F08/$3F0C
            0x3f10 | 0x3f14 | 0x3f18 | 0x3f1c => {
                let addr_mirror = ppu_addr - 0x10;
                self.palette_table[(addr_mirror - PALETTE_START_ADDR) as usize] = data;
            }
            0x3f00..=0x3fff => {
                self.palette_table[(ppu_addr - PALETTE_START_ADDR) as usize] = data;
            }
            _ => panic!("unexpected access to mirrored space {}", ppu_addr),
        }
    }

    fn increment_vram_ptr(&mut self) {
        self.addr.increment(self.control.get_vram_jump_dist());
    }

    /**
     * Horizontal
     * 1st screen: 2000..2400 <- 2400..2800     (0, 1)
     * 2nd screen: 2800..2C00 <- 2C00..0x3F00   (2, 3)
     *
     * Vertical
     * 1st screen | 2nd screen
     * 2000..2400 | 2400..2800      (0, 1)
     * 2800..2C00 | 2C00..0x3F00    (2, 3)
     *
     * While it is trivial to see why (1, H), (2, V) & (3, V) have the following offsets,
     * the other cases rely on the fact that these addresses map onto contiguous blocks
     * of memory on the PPU VRAM reserved for nametables.
     * Hence, for (2, H), we need to subtract another 0x400 to offset the address to the next 1KiB on the vram for the 2nd nametable/screen.
     * Since (3, H) maps onto (2, H), it needs to have another 0x400 subtracted.
     */
    fn mirror_vram(&self, addr: u16) -> u16 {
        let addr_mirror = addr & NAME_TABLE_END_ADDR; // mirrors addresses surpassing the end of the name tables
        let vram_addr = addr_mirror - NAME_TABLE_START_ADDR;
        let name_table_index = vram_addr / NAME_TABLE_SIZE;
        match (name_table_index, &self.mirror_mode) {
            (1, Mirroring::HORIZONTAL) | (2, Mirroring::HORIZONTAL) => vram_addr - NAME_TABLE_SIZE,
            (3, Mirroring::HORIZONTAL) | (2, Mirroring::VERTICAL) | (3, Mirroring::VERTICAL) => {
                vram_addr - (2 * NAME_TABLE_SIZE)
            }
            _ => vram_addr,
        }
    }
}
