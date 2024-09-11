pub struct ScrollRegister {
    scroll_x: u8,
    scroll_y: u8,
    address_latch: bool
}

impl ScrollRegister {

    pub fn new() -> Self {
        ScrollRegister { scroll_x: 0, scroll_y: 0, address_latch: false }
    }

    pub fn write(&mut self, data: u8) {
        if self.address_latch {
            self.scroll_x = data
        } else {
            self.scroll_y = data
        }
        self.address_latch = !self.address_latch;
    }

    pub fn reset_latch(&mut self) {
        self.address_latch = false;
    }
}
