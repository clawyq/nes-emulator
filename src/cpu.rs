#[allow(non_camel_case_types)]
pub enum AddressingMode {
    Immediate,
    ZeroPage,
    ZeroPage_X,
    ZeroPage_Y,
    Absolute,
    Absolute_X,
    Absolute_Y,
    Indirect_X,
    Indirect_Y,
    NoneAddressing,
}

pub struct CPU {
    pub register_a: u8,
    pub register_x: u8,
    pub register_y: u8,
    pub status: u8, // N V NOT_USED B D I Z C
    pub program_counter: u16,
    memory: [u8; 0xFFFF],
}

/**
 * 4 responsibilities
 *  1. Fetch next instruction to execute
 *  2. Decode instruction
 *  3. Execute instruction
 *  4. Repeat
 *
 * Instruction reference according to https://www.nesdev.org/obelisk-6502-guide/reference.html
 */
impl CPU {
    pub fn new() -> Self {
        CPU {
            register_a: 0,
            register_x: 0,
            register_y: 0,
            status: 0,
            program_counter: 0,
            memory: [0; 0xFFFF],
        }
    }

    /**
     * Addressing modes are cracked https://skilldrick.github.io/easy6502/#addressing.
     * Depending on context, we interpret the subsequent 1/2/3 bytes differently
     * to find the value we need as an operand for our command.
     */
    fn get_operand_address(&self, mode: &AddressingMode) -> u16 {
        match mode {
            AddressingMode::Immediate => self.program_counter,
            AddressingMode::ZeroPage => self.mem_read(self.program_counter) as u16,
            AddressingMode::ZeroPage_X => {
                let addr = self.mem_read(self.program_counter);
                addr.wrapping_add(self.register_x) as u16
            },
            AddressingMode::ZeroPage_Y => {
                let addr = self.mem_read(self.program_counter);
                addr.wrapping_add(self.register_y) as u16
            },
            AddressingMode::Absolute => self.mem_read_u16(self.program_counter),
            AddressingMode::Absolute_X => {
                let addr = self.mem_read_u16(self.program_counter);
                addr.wrapping_add(self.register_x as u16) as u16
            },
            AddressingMode::Absolute_Y => {
                let addr = self.mem_read_u16(self.program_counter);
                addr.wrapping_add(self.register_y as u16) as u16
            },
            AddressingMode::Indirect_X => {
                let addr = self.mem_read(self.program_counter);
                let x_addr = addr.wrapping_add(self.register_x) as u16;
                let lo_addr = self.mem_read(x_addr) as u16;
                let hi_addr = self.mem_read(x_addr.wrapping_add(1)) as u16;
                hi_addr << 8 | lo_addr
            },
            AddressingMode::Indirect_Y => {
                let addr = self.mem_read(self.program_counter);
                let lo_addr = self.mem_read(addr as u16) as u16;
                let hi_addr = self.mem_read(addr.wrapping_add(1) as u16) as u16;
                let preoffset_addr = (hi_addr << 8) | lo_addr;
                preoffset_addr.wrapping_add(self.register_y as u16)
            },
            AddressingMode::NoneAddressing => panic!("Go to sleep. This is not working.")
        }
    }

    fn mem_read(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        self.memory[addr as usize] = data;
    }

    fn mem_read_u16(&self, addr: u16) -> u16 {
        let lo = self.mem_read(addr) as u16;
        let hi = self.mem_read(addr + 1) as u16;
        (hi << 8) | lo
    }

    fn mem_write_u16(&mut self, addr: u16, data: u16) {
        let lo = (data & 0xFF) as u8;
        let hi = (data >> 8) as u8;
        self.mem_write(addr, lo);
        self.mem_write(addr + 1, hi);
    }

    fn reset(&mut self) {
        self.register_a = 0;
        self.register_x = 0;
        self.status = 0;
        self.program_counter = self.mem_read_u16(0xFFFC);
    }

    pub fn load_and_run(&mut self, program: Vec<u8>) {
        self.load(program);
        self.reset();
        self.run();
    }

    fn load(&mut self, program: Vec<u8>) {
        self.memory[0x8000..(0x8000 + program.len())].copy_from_slice(&program[..]);
        self.mem_write_u16(0xFFFC, 0x8000);
    }

    pub fn run(&mut self) {
        loop {
            let opscode = self.mem_read(self.program_counter);
            self.program_counter += 1 as u16;
            match opscode {
                0x00 => {
                    return;
                }
                0xA2 => {
                    // Reads an extra byte for parameter
                    let param = self.memory[self.program_counter as usize];
                    self.program_counter += 1;
                    self.ldx(param);
                }
                0xA9 => {
                    // Reads an extra byte for parameter
                    let param = self.memory[self.program_counter as usize];
                    self.program_counter += 1;
                    self.lda(param);
                }
                0xAA => {
                    self.tax();
                }
                0xE8 => {
                    self.inx();
                }
                _ => todo!(),
            }
        }
    }

    fn lda(&mut self, value: u8) {
        self.register_a = value;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn ldx(&mut self, value: u8) {
        self.register_x = value;
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn tax(&mut self) {
        self.register_x = self.register_a;
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn inx(&mut self) {
        self.register_x = self.register_x.wrapping_add(1);
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn update_zero_and_negative_flags(&mut self, value: u8) {
        // setting the Z(ero) flag
        if value == 0 {
            self.status = self.status | 0b0000_0010;
        } else {
            self.status = self.status & 0b1111_1101;
        }

        // Setting the N(egative) flag
        if value & 0b1000_0000 != 0 {
            self.status = self.status | 0b1000_0000;
        } else {
            self.status = self.status & 0b0111_1111;
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_0xa9_lda_immediate_load_data() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x05, 0x00]);
        assert_eq!(cpu.register_a, 0b0000_0101);
        assert_eq!(cpu.status, 0);
        assert!(cpu.status & 0b0000_0010 == 0b00); // Zero unset?
        assert!(cpu.status & 0b1000_0000 == 0); // Negative unset?
    }

    #[test]
    fn test_0xa9_lda_zero_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x00, 0x00]);
        assert_eq!(cpu.register_a, 0x00);
        assert_eq!(cpu.status, 2);
        assert!(cpu.status & 0b0000_0010 == 0b10); // Zero set?
        assert!(cpu.status & 0b1000_0000 == 0); // Negative unset?
    }

    #[test]
    fn test_0xa9_lda_negative_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x09, 0x00]);
        assert_eq!(cpu.register_a, 0x09);
        assert_eq!(cpu.status, 0);
        assert!(cpu.status & 0b0000_0010 == 0b00); // Zero set?
        assert!(cpu.status & 0b1000_0000 == 0); // Negative unset?
    }

    #[test]
    fn test_0xa2_ldx_immediate_load_data() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa2, 0x05, 0x00]);
        assert_eq!(cpu.register_x, 0b0000_0101);
        assert_eq!(cpu.status, 0);
        assert!(cpu.status & 0b0000_0010 == 0b00); // Zero unset?
        assert!(cpu.status & 0b1000_0000 == 0); // Negative unset?
    }

    #[test]
    fn test_0xa2_ldx_zero_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa2, 0x00, 0x00]);
        assert_eq!(cpu.register_x, 0x00);
        assert_eq!(cpu.status, 2);
        assert!(cpu.status & 0b0000_0010 == 0b10); // Zero set?
        assert!(cpu.status & 0b1000_0000 == 0); // Negative unset?
    }

    #[test]
    fn test_0xa2_ldx_negative_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa2, 0x09, 0x00]);
        assert_eq!(cpu.register_x, 0x09);
        assert_eq!(cpu.status, 0);
        assert!(cpu.status & 0b0000_0010 == 0b00); // Zero set?
        assert!(cpu.status & 0b1000_0000 == 0); // Negative unset?
    }

    #[test]
    fn test_0xaa_tax_move_a_to_x() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 10, 0xaa, 0x00]);

        assert_eq!(cpu.register_x, 10);
        assert!(cpu.status & 0b0000_0010 == 0b00); // Zero set?
        assert!(cpu.status & 0b1000_0000 == 0); // Negative unset?
    }

    #[test]
    fn test_5_ops_working_together() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);

        assert_eq!(cpu.register_x, 0xc1)
    }

    #[test]
    fn test_inx_overflow() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa2, 0xff, 0xe8, 0xe8, 0x00]);

        assert_eq!(cpu.register_x, 1)
    }
}
