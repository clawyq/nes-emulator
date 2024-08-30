use crate::opcodes::get_opcode_details;
use bitflags::bitflags;

bitflags! {
    // N V B2 B D I Z C
    #[derive(PartialEq)]
    struct StatusFlags: u8 {
        const RESET             = 0b0000_0000;
        const CARRY             = 0b0000_0001;
        const ZERO              = 0b0000_0010;
        const INTERRUPT_DSIABLE = 0b0000_0100;
        const DECIMAL           = 0b0000_1000;
        const BREAK             = 0b0001_0000;
        const BREAK2            = 0b0010_0000;
        const OVERFLOW          = 0b0100_0000;
        const NEGATIVE          = 0b1000_0000;
    }
}

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
    Implied,
}

pub struct CPU {
    pub register_a: u8,
    pub register_x: u8,
    pub register_y: u8,
    pub status: StatusFlags,
    pub stack_ptr: u8,
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

const STACK_ADDR: u16 = 0x0100;
const STACK_PTR_INIT: u8 = 0xFD;
impl CPU {
    pub fn new() -> Self {
        CPU {
            register_a: 0,
            register_x: 0,
            register_y: 0,
            status: StatusFlags::RESET,
            program_counter: 0,
            stack_ptr: STACK_PTR_INIT,
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
            }
            AddressingMode::ZeroPage_Y => {
                let addr = self.mem_read(self.program_counter);
                addr.wrapping_add(self.register_y) as u16
            }
            AddressingMode::Absolute => self.mem_read_u16(self.program_counter),
            AddressingMode::Absolute_X => {
                let addr = self.mem_read_u16(self.program_counter);
                addr.wrapping_add(self.register_x as u16) as u16
            }
            AddressingMode::Absolute_Y => {
                let addr = self.mem_read_u16(self.program_counter);
                addr.wrapping_add(self.register_y as u16) as u16
            }
            AddressingMode::Indirect_X => {
                let addr = self.mem_read(self.program_counter);
                let x_addr = addr.wrapping_add(self.register_x) as u16;
                u16::from_le_bytes([self.mem_read(x_addr), self.mem_read(x_addr.wrapping_add(1))])
            }
            AddressingMode::Indirect_Y => {
                let addr = self.mem_read(self.program_counter);
                let preoffset_addr = u16::from_le_bytes([self.mem_read(addr as u16), self.mem_read(addr.wrapping_add(1) as u16)]);
                preoffset_addr.wrapping_add(self.register_y as u16)
            }
            AddressingMode::Implied => panic!("Go to sleep. This is not working."),
        }
    }

    fn mem_read(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        self.memory[addr as usize] = data;
    }

    fn mem_read_u16(&self, addr: u16) -> u16 {
        u16::from_le_bytes([self.mem_read(addr), self.mem_read(addr.wrapping_add(1))])
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
        self.status = StatusFlags::RESET;
        self.stack_ptr = STACK_PTR_INIT;
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
            let opcode = self.mem_read(self.program_counter);
            let opcode_details = get_opcode_details(&opcode).expect(&format!("Opcode {opcode} is not recognised."));
            self.program_counter += 1 as u16;
            match opcode {
                0x00 => {
                    return;
                }
                0x85 | 0x95 => {
                    self.sta(&(opcode_details.mode));
                }
                0xA2 => {
                    let param = self.memory[self.program_counter as usize];
                    self.ldx(param);
                }
                0xA5 | 0xA9 | 0xAD => {
                    self.lda(&(opcode_details.mode));
                }
                0xAA => {
                    self.tax();
                }
                0xE8 => {
                    self.inx();
                }
                _ => todo!(),
            }
            self.program_counter+= opcode_details.additional_bytes as u16;
        }
    }

    fn lda(&mut self, mode: &AddressingMode) {
        let operand_addr = self.get_operand_address(mode);
        let operand_value = self.mem_read(operand_addr);

        self.register_a = operand_value;
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

    fn sta(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_a);
    }

    fn update_zero_and_negative_flags(&mut self, value: u8) {
        // setting the Z(ero) flag
        if value == 0 {
            self.status.insert(StatusFlags::ZERO);
        } else {
            self.status.remove(StatusFlags::ZERO);
        }

        // Setting the N(egative) flag
        if value & 0b1000_0000 != 0 {
            self.status.insert(StatusFlags::NEGATIVE);
        } else {
            self.status.remove(StatusFlags::NEGATIVE);
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
        assert!(!cpu.status.contains(StatusFlags::ZERO));
        assert!(!cpu.status.contains(StatusFlags::NEGATIVE));
    }

    #[test]
    fn test_0xa9_lda_zero_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x00, 0x00]);
        assert_eq!(cpu.register_a, 0x00);
        assert!(cpu.status.contains(StatusFlags::ZERO));
        assert!(!cpu.status.contains(StatusFlags::NEGATIVE));
    }

    #[test]
    fn test_0xa9_lda_negative_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x09, 0x00]);
        assert_eq!(cpu.register_a, 0x09);
        assert!(!cpu.status.contains(StatusFlags::ZERO));
        assert!(!cpu.status.contains(StatusFlags::NEGATIVE));
    }

    #[test]
    fn test_0xa2_ldx_immediate_load_data() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa2, 0x05, 0x00]);
        assert_eq!(cpu.register_x, 0b0000_0101);
        assert!(!cpu.status.contains(StatusFlags::ZERO));
        assert!(!cpu.status.contains(StatusFlags::NEGATIVE));
    }

    #[test]
    fn test_0xa2_ldx_zero_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa2, 0x00, 0x00]);
        assert_eq!(cpu.register_x, 0x00);
        assert!(cpu.status.contains(StatusFlags::ZERO));
        assert!(!cpu.status.contains(StatusFlags::NEGATIVE));
    }

    #[test]
    fn test_0xa2_ldx_negative_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa2, 0x09, 0x00]);
        assert_eq!(cpu.register_x, 0x09);
        assert!(!cpu.status.contains(StatusFlags::ZERO));
        assert!(!cpu.status.contains(StatusFlags::NEGATIVE));
    }

    #[test]
    fn test_0xaa_tax_move_a_to_x() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 10, 0xaa, 0x00]);

        assert_eq!(cpu.register_x, 10);
        assert!(!cpu.status.contains(StatusFlags::ZERO));
        assert!(!cpu.status.contains(StatusFlags::NEGATIVE));
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

    #[test]
    fn test_0x85_sta() {
        let mut cpu = CPU::new();
        let reg_a_val = 0x09;
        let destination_addr = 0x28;
        cpu.load_and_run(vec![0xa9, reg_a_val, 0x85, destination_addr]);

        assert_eq!(cpu.register_a, reg_a_val);
        assert_eq!(cpu.memory[destination_addr as usize], reg_a_val);
    }

    #[test]
    fn test_0x95_sta() {
        let mut cpu = CPU::new();
        let reg_a_val = 0x09;
        let reg_x_val = 0x02;
        let destination_addr = 0x28;
        cpu.load_and_run(vec![0xa2, reg_x_val, 0xa9, reg_a_val, 0x95, destination_addr]);

        assert_eq!(cpu.register_a, reg_a_val);
        assert_eq!(cpu.memory[(destination_addr.wrapping_add(reg_x_val)) as usize], reg_a_val);
    }
}
