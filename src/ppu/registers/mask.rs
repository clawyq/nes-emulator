
use bitflags::bitflags;
bitflags! {
    pub struct MaskRegister: u8 {
        const GREYSCALE                 = 0b0000_0001;
        const SHOW_8LEFTMOST_BACKGROUND = 0b0000_0010;
        const SHOW_8LEFTMOST_SPRITES    = 0b0000_0100;
        const SHOW_BACKGROUND           = 0b0000_1000;
        const SHOW_SPRITES              = 0b0001_0000;
        const EMPHASISE_RED             = 0b0010_0000;
        const EMPHASISE_GREEN           = 0b0100_0000;
        const EMPHASISE_BLUE            = 0b1000_0000;
    }
}

impl MaskRegister {
    pub fn new() -> Self {
        MaskRegister::from_bits_truncate(0000_0000)
    }

    pub fn update(&mut self, data: u8) {
        *self = MaskRegister::from_bits_truncate(data);
    }

    pub fn is_greyscale(&self) -> bool {
        return self.contains(MaskRegister::GREYSCALE);

    }

    pub fn show_leftmost_background(&self) -> bool {
        return self.contains(MaskRegister::SHOW_8LEFTMOST_BACKGROUND);
    }

    pub fn show_leftmost_sprites(&self) -> bool {
        return self.contains(MaskRegister::SHOW_8LEFTMOST_SPRITES);
    }

    pub fn show_background(&self) -> bool {
        return self.contains(MaskRegister::SHOW_BACKGROUND);
    }

    pub fn show_sprites(&self) -> bool {
        return self.contains(MaskRegister::SHOW_SPRITES);
    }

    pub fn emphasise(&self) -> Vec<Colour> {
        let mut colours = vec![];
        if self.contains(MaskRegister::EMPHASISE_RED) {
            colours.push(Colour::RED);
        }

        if self.contains(MaskRegister::EMPHASISE_GREEN) {
            colours.push(Colour::GREEN);
        }

        if self.contains(MaskRegister::EMPHASISE_BLUE) {
            colours.push(Colour::BLUE);
        }

        return colours;
    }
}

enum Colour {
    RED,
    GREEN,
    BLUE,
}
