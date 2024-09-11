use crate::ppu::BEFORE_MIRROR_RANGE;
pub struct AddressRegister {
    ppu_address: (u8, u8), //ignores endianness
    address_latch: bool,
}

impl AddressRegister {
    pub fn new() -> Self {
        AddressRegister {
            ppu_address: (0, 0),
            address_latch: true,
        }
    }

    pub fn get(&self) -> u16 {
        (self.ppu_address.0 as u16) << 8 | (self.ppu_address.1 as u16)
    }

    fn set(&mut self, addr: u16) {
        self.ppu_address.0 = (addr >> 8) as u8;
        self.ppu_address.1 = (addr & 0xff) as u8;
    }

    pub fn reset_latch(&mut self) {
        self.address_latch = false;
    }

    pub fn write(&mut self, addr: u8) {
        if self.address_latch {
            self.ppu_address.0 = addr;
        } else {
            self.ppu_address.1 = addr;
        }
        self.mirror();
        self.address_latch = !self.address_latch;
    }

    pub fn increment(&mut self, increment: u8) {
        let old_val = self.ppu_address.1;
        self.ppu_address.1 = old_val.wrapping_add(increment);
        if old_val > self.ppu_address.1 {
            self.ppu_address.0 = self.ppu_address.0.wrapping_add(1);
        }
        self.mirror();
    }

    pub fn mirror(&mut self) {
        let full_address = self.get();
        if full_address <= BEFORE_MIRROR_RANGE {
            return;
        }
        self.set(full_address & BEFORE_MIRROR_RANGE);
    }
}
