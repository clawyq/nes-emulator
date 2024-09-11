
use bitflags::bitflags;
bitflags! {
    pub struct ControlRegister: u8 {
        const NAMETABLE_1              = 0b0000_0001;
        const NAMETABLE_2              = 0b0000_0010;
        const VRAM_INCREMENT_MODE      = 0b0000_0100;
        const SPRITE_PATTERN_ADDR      = 0b0000_1000;
        const BACKGROUND_PATTERN_ADDR  = 0b0001_0000;
        const SPRITE_SIZE              = 0b0010_0000;
        const MASTER_SLAVE_SELECT      = 0b0100_0000;
        const GENERATE_NMI             = 0b1000_0000;
    }
}

impl ControlRegister {
    pub fn new() -> Self {
        ControlRegister::from_bits_truncate(0000_0000)
    }

    pub fn update(&mut self, data: u8) {
        *self = ControlRegister::from_bits_truncate(data);
    }

    pub fn get_nametable_address(&self) -> u16 {
        match self.bits() & 0x11 {
            0 => 0x2000,
            1 => 0x2400,
            2 => 0x2800,
            3 => 0x2C00,
            _ => panic!("how tf")
        }
    }

    pub fn get_vram_jump_dist(&self) -> u8 {
        if self.contains(ControlRegister::VRAM_INCREMENT_MODE) {
            32
        } else {
            1
        }
    }

    pub fn get_sprite_pattern_table_address(&self) -> u16 {
        if self.contains(ControlRegister::SPRITE_PATTERN_ADDR) {
            0x1000
        } else {
            0x0000
        }
    }

    pub fn get_background_pattern_table_address(&self) -> u16 {
        if self.contains(ControlRegister::BACKGROUND_PATTERN_ADDR) {
            0x1000
        } else {
            0x0000
        }
    }
    pub fn get_sprite_size(&self) -> u8 {
        if self.contains(ControlRegister::SPRITE_SIZE) {
            16
        } else {
            8
        }
    }

    pub fn get_master_slave_selection(&self) -> u8 {
        if self.contains(ControlRegister::MASTER_SLAVE_SELECT) {
            1
        } else {
            0
        }
    }

    pub fn generate_nmi(&self) -> bool {
        return self.contains(ControlRegister::GENERATE_NMI);
    }
}
