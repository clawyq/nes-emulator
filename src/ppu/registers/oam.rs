pub struct Oam {
    addr: u8,
    data: [u8; 256]
}

impl Oam {
    pub fn new() -> Self {
        Oam {
            addr: 0,
            data: [0; 256]
        }
    }

    pub fn write_addr(&mut self, addr: u8) {
        self.addr = addr;
    }

    pub fn read_data(&self) -> u8 {
        self.data[self.addr as usize]
    }

    pub fn write_data(&mut self, data: u8) {
        self.data[self.addr as usize] = data;
        self.addr = self.addr.wrapping_add(1);
    }
}
