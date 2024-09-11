use bitflags::bitflags;

bitflags! {
    pub struct StatusRegister: u8 {
        const UNUSED1         = 0b0000_0001;
        const UNUSED2         = 0b0000_0010;
        const UNUSED3         = 0b0000_0100;
        const UNUSED4         = 0b0000_1000;
        const UNUSED5         = 0b0001_0000;
        const SPRITE_OVERFLOW = 0b0010_0000;
        const SPRITE_0_HIT    = 0b0100_0000;
        const IN_VBLANK       = 0b1000_0000;
    }

}

impl StatusRegister {
    pub fn new() -> Self {
        StatusRegister::from_bits_truncate(0b00000000)
    }

    pub fn set_sprite_overflow(&mut self, status: bool) {
        self.set(StatusRegister::SPRITE_OVERFLOW, status);
    }

    pub fn set_sprite_zero_hit(&mut self, status: bool) {
      self.set(StatusRegister::SPRITE_0_HIT, status);
    }

    pub fn set_vblank(&mut self, status: bool) {
      self.set(StatusRegister::IN_VBLANK, status);
    }

    pub fn reset_vblank(&mut self) {
        self.remove(StatusRegister::IN_VBLANK);
    }

    pub fn is_in_vblank(&self) -> bool {
        self.contains(StatusRegister::IN_VBLANK)
    }
}