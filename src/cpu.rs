use crate::{bus::Bus, opcodes::get_opcode_details};
use bitflags::bitflags;

bitflags! {
    // N V B2 B D I Z C
    #[derive(Debug, Clone, PartialEq)]
    struct StatusFlags: u8 {
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

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum AddressingMode {
    Immediate,
    ZeroPage,
    ZeroPage_X,
    ZeroPage_Y,
    Absolute,
    Absolute_X,
    Absolute_Y,
    Indirect,
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
    pub bus: Bus,
}

pub trait Mem {
    fn mem_read(&self, addr: u16) -> u8;

    fn mem_write(&mut self, addr: u16, data: u8);

    fn mem_read_u16(&self, pos: u16) -> u16 {
        let lo = self.mem_read(pos) as u16;
        let hi = self.mem_read(pos + 1) as u16;
        (hi << 8) | (lo as u16)
    }

    fn mem_write_u16(&mut self, pos: u16, data: u16) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0xff) as u8;
        self.mem_write(pos, lo);
        self.mem_write(pos + 1, hi);
    }
}

impl Mem for CPU {
    fn mem_read(&self, addr: u16) -> u8 {
        self.bus.mem_read(addr)
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        self.bus.mem_write(addr, data);
    }

    fn mem_read_u16(&self, pos: u16) -> u16 {
        self.bus.mem_read_u16(pos)
    }

    fn mem_write_u16(&mut self, pos: u16, data: u16) {
        self.bus.mem_write_u16(pos, data)
    }
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
            status: StatusFlags::from_bits_truncate(0),
            program_counter: 0,
            stack_ptr: STACK_PTR_INIT,
            bus: Bus::new()
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
            AddressingMode::Indirect => {
                let addr = self.mem_read_u16(self.program_counter);
                let lo = self.mem_read(addr);
                let hi = if addr & 0x00FF == 0x00FF {
                    // if im at the page boundary, stay on the same page (ignore)
                    self.mem_read(addr & 0xFF00)
                } else {
                    self.mem_read(addr.wrapping_add(1))
                };
                u16::from_le_bytes([lo, hi])
            }
            AddressingMode::Indirect_X => {
                let addr = self.mem_read(self.program_counter);
                let x_addr = addr.wrapping_add(self.register_x) as u16;
                u16::from_le_bytes([self.mem_read(x_addr), self.mem_read(x_addr.wrapping_add(1))])
            }
            AddressingMode::Indirect_Y => {
                let addr = self.mem_read(self.program_counter);
                let preoffset_addr = u16::from_le_bytes([
                    self.mem_read(addr as u16),
                    self.mem_read(addr.wrapping_add(1) as u16),
                ]);
                preoffset_addr.wrapping_add(self.register_y as u16)
            }
            AddressingMode::Implied => {
                panic!("Go to sleep. Why you tryna find a new address bruv.")
            }
        }
    }

    fn push(&mut self, data: u8) {
        self.mem_write(STACK_ADDR + self.stack_ptr as u16, data);
        self.stack_ptr = self.stack_ptr.wrapping_sub(1);
    }

    fn pop(&mut self) -> u8 {
        self.stack_ptr = self.stack_ptr.wrapping_add(1);
        self.mem_read(STACK_ADDR + self.stack_ptr as u16)
    }

    fn push_u16(&mut self, data: u16) {
        self.push((data >> 8) as u8);
        self.push((data & 0xFF) as u8);
    }

    fn pop_u16(&mut self) -> u16 {
        u16::from_le_bytes([self.pop(), self.pop()])
    }

    pub fn reset(&mut self) {
        self.register_a = 0;
        self.register_x = 0;
        self.status = StatusFlags::from_bits_truncate(0);
        self.stack_ptr = STACK_PTR_INIT;
        self.program_counter = self.mem_read_u16(0xFFFC);
    }

    pub fn load_and_run(&mut self, program: Vec<u8>) {
        self.load(program);
        self.reset();
        self.run_with_callback(|_| {});
    }

    pub fn load(&mut self, program: Vec<u8>) {
        self.memory[0x0600..(0x0600 + program.len())].copy_from_slice(&program[..]);
        self.mem_write_u16(0xFFFC, 0x0600);
    }

    fn has_jumped_or_branched(&self, other_addr: u16) -> bool {
        self.program_counter != other_addr
    }

    pub fn run_with_callback<F>(&mut self, mut callback: F)
    where
        F: FnMut(&mut CPU),
    {
        loop {
            let opcode = self.mem_read(self.program_counter);
            let opcode_details =
                get_opcode_details(&opcode).expect(&format!("Opcode {opcode} is not recognised."));
            let mode: &AddressingMode = &(opcode_details.mode);

            self.program_counter += 1 as u16;
            let program_counter_before_exec = self.program_counter;
            match opcode {
                0x00 => {
                    return;
                }
                0x69 | 0x67 | 0x75 | 0x6D | 0x7D | 0x79 | 0x61 | 0x71 => {
                    self.adc(mode);
                }
                0x29 | 0x25 | 0x35 | 0x2D | 0x3D | 0x39 | 0x21 | 0x31 => {
                    self.and(mode);
                }
                0x0A => {
                    self.asl_accumulator();
                }
                0x06 | 0x16 | 0x0E | 0x1E => {
                    self.asl(mode);
                }
                // BPL
                0x10 => {
                    self.branch(!self.status.contains(StatusFlags::NEGATIVE));
                }
                // BVC
                0x50 => {
                    self.branch(!self.status.contains(StatusFlags::OVERFLOW));
                }
                // BVS
                0x70 => {
                    self.branch(self.status.contains(StatusFlags::OVERFLOW));
                }
                //BCC
                0x90 => {
                    self.branch(!self.status.contains(StatusFlags::CARRY));
                }
                //BCS
                0xB0 => {
                    self.branch(self.status.contains(StatusFlags::CARRY));
                }
                //BNE
                0xD0 => {
                    self.branch(!self.status.contains(StatusFlags::ZERO));
                }
                //BEQ
                0xF0 => {
                    self.branch(self.status.contains(StatusFlags::ZERO));
                }
                // BMI
                0x30 => {
                    self.branch(self.status.contains(StatusFlags::NEGATIVE));
                }
                // CLC
                0x18 => {
                    self.status.remove(StatusFlags::CARRY);
                }
                // CLV
                0xB8 => {
                    self.status.remove(StatusFlags::OVERFLOW);
                }
                // CLD
                0xD8 => {
                    self.status.remove(StatusFlags::DECIMAL);
                }
                // CLI
                0x58 => {
                    self.status.remove(StatusFlags::INTERRUPT_DSIABLE);
                }
                0x24 | 0x2C => {
                    self.bit(mode);
                }
                // CMP
                0xC9 | 0xC5 | 0xD5 | 0xCD | 0xDD | 0xD9 | 0xC1 | 0xD1 => {
                    self.compare(self.register_a, mode);
                }
                // CPX
                0xE0 | 0xE4 | 0xEC => {
                    self.compare(self.register_x, mode);
                }
                // CPY
                0xC0 | 0xC4 | 0xCC => {
                    self.compare(self.register_y, mode);
                }
                // DEC
                0xC6 | 0xD6 | 0xCE | 0xDE => {
                    self.dec(mode);
                }
                0xCA => {
                    self.dex();
                }
                0x88 => {
                    self.dey();
                }
                0x49 | 0x45 | 0x55 | 0x4D | 0x5D | 0x59 | 0x41 | 0x51 => {
                    self.eor(mode);
                }
                0xE6 | 0xF6 | 0xEE | 0xFE => {
                    self.inc(mode);
                }
                0xE8 => {
                    self.inx();
                }
                0xC8 => {
                    self.iny();
                }
                0x4C | 0x6C => {
                    self.jmp(mode);
                }
                0x20 => {
                    self.jsr(mode);
                }
                0xA2 | 0xA6 | 0xB6 | 0xAE | 0xBE => {
                    self.ldx(mode);
                }
                0xA0 | 0xA4 | 0xB4 | 0xAC | 0xBC => {
                    self.ldy(mode);
                }
                0xA9 | 0xA5 | 0xB5 | 0xAD | 0xBD | 0xB9 | 0xA1 | 0xB1 => {
                    self.lda(mode);
                }
                0x4A => {
                    self.lsr_accumulator();
                }
                0x46 | 0x56 | 0x4E | 0x5E => {
                    self.lsr(mode);
                }
                0xEA => {} // NOP
                0x09 | 0x05 | 0x15 | 0x0D | 0x1D | 0x19 | 0x01 | 0x11 => {
                    self.ora(mode);
                }
                0x48 => {
                    self.pha();
                }
                0x08 => {
                    self.php();
                }
                0x68 => {
                    self.pla();
                }
                0x28 => {
                    self.plp();
                }
                0x2A => {
                    self.rol_accumulator();
                }
                0x26 | 0x36 | 0x2E | 0x3E => {
                    self.rol(mode);
                }
                0x6A => {
                    self.ror_accumulator();
                }
                0x66 | 0x76 | 0x6E | 0x7E => {
                    self.ror(mode);
                }
                0x40 => {
                    self.rti();
                }
                0x60 => {
                    self.rts();
                }
                0xE9 | 0xE5 | 0xF5 | 0xED | 0xFD | 0xF9 | 0xE1 | 0xF1 => {
                    self.sbc(mode);
                }
                0x38 => {
                    self.sec();
                }
                0xF8 => {
                    self.sed();
                }
                0x78 => {
                    self.sei();
                }
                0x85 | 0x95 | 0x8D | 0x9D | 0x99 | 0x81 | 0x91 => {
                    self.sta(mode);
                }
                0x86 | 0x96 | 0x8E => {
                    self.stx(mode);
                }
                0x84 | 0x94 | 0x8C => {
                    self.sty(mode);
                }
                0xAA => {
                    self.tax();
                }
                0xA8 => {
                    self.tay();
                }
                0xBA => {
                    self.tsx();
                }
                0x8A => {
                    self.txa();
                }
                0x9A => {
                    self.txs();
                }
                0x98 => {
                    self.tya();
                }
                _ => todo!(),
            }
            if !self.has_jumped_or_branched(program_counter_before_exec) {
                self.program_counter += opcode_details.additional_bytes as u16;
            }
            callback(self);
        }
    }

    fn set_accumulator(&mut self, value: u8) {
        self.register_a = value;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn add_to_accumulator(&mut self, value: u8) {
        let sum = self.register_a as u16
            + value as u16
            + (if self.status.contains(StatusFlags::CARRY) {
                1
            } else {
                0
            }) as u16;
        self.status.set(StatusFlags::CARRY, sum > 0xFF);
        self.status.set(
            StatusFlags::OVERFLOW,
            (value ^ (sum as u8)) & (self.register_a ^ (sum as u8)) & 0x80 != 0,
        );
        self.register_a = sum as u8;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn adc(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let data = self.mem_read(addr);
        self.add_to_accumulator(data);
    }

    fn sbc(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let data = self.mem_read(addr);
        self.add_to_accumulator((data as i8).wrapping_neg().wrapping_sub(1) as u8);
    }

    fn and(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let data = self.mem_read(addr);
        self.register_a &= data;

        self.update_zero_and_negative_flags(self.register_a);
    }

    fn asl_accumulator(&mut self) {
        let mut data = self.register_a;
        if data >> 7 == 1 {
            self.status.insert(StatusFlags::CARRY)
        } else {
            self.status.remove(StatusFlags::CARRY)
        }
        self.set_accumulator(data << 1);
    }

    fn asl(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let mut data = self.mem_read(addr);
        if data >> 7 == 1 {
            self.status.insert(StatusFlags::CARRY)
        } else {
            self.status.remove(StatusFlags::CARRY)
        }

        let result = data << 1;
        self.mem_write(addr, result);
        self.update_zero_and_negative_flags(result);
    }

    fn branch(&mut self, condition_to_jump: bool) {
        if !condition_to_jump {
            return;
        }

        let jump_dist = self.mem_read(self.program_counter) as i8;
        let destination: u16 = self
            .program_counter
            .wrapping_add(1)
            .wrapping_add(jump_dist as u16);
        self.program_counter = destination;
    }

    fn bit(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let data = self.mem_read(addr);
        let result = self.register_a & data;
        if result == 0 {
            self.status.insert(StatusFlags::ZERO);
        } else {
            self.status.remove(StatusFlags::ZERO);
        }
        self.status
            .set(StatusFlags::NEGATIVE, data & 0b1000_0000 > 0);
        self.status
            .set(StatusFlags::OVERFLOW, data & 0b0100_0000 > 0);
    }

    fn compare(&mut self, compare_value: u8, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let data = self.mem_read(addr);

        self.status.set(StatusFlags::CARRY, compare_value >= data);
        self.update_zero_and_negative_flags(compare_value.wrapping_sub(data));
    }

    fn dec(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let mut data = self.mem_read(addr);
        data = data.wrapping_sub(1);
        self.mem_write(addr, data);
        self.update_zero_and_negative_flags(data);
    }

    fn dex(&mut self) {
        self.register_x = self.register_x.wrapping_sub(1);
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn dey(&mut self) {
        self.register_y = self.register_y.wrapping_sub(1);
        self.update_zero_and_negative_flags(self.register_y);
    }

    fn eor(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let data = self.mem_read(addr);
        self.update_zero_and_negative_flags(self.register_a ^ data);
    }

    fn inc(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let mut data = self.mem_read(addr);
        data = data.wrapping_add(1);
        self.mem_write(addr, data);
        self.update_zero_and_negative_flags(data);
    }

    fn inx(&mut self) {
        self.register_x = self.register_x.wrapping_add(1);
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn iny(&mut self) {
        self.register_y = self.register_y.wrapping_add(1);
        self.update_zero_and_negative_flags(self.register_y);
    }

    fn jmp(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.program_counter = addr;
    }

    fn jsr(&mut self, mode: &AddressingMode) {
        self.push_u16(self.program_counter + 1); // stack now has the last byte of the JSR arg -> next execution i will + 1 so i will be at the right instruction
        let addr = self.get_operand_address(mode);
        self.program_counter = addr;
    }

    fn lda(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let data = self.mem_read(addr);

        self.register_a = data;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn ldx(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let data = self.mem_read(addr);

        self.register_x = data;
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn ldy(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let data = self.mem_read(addr);

        self.register_y = data;
        self.update_zero_and_negative_flags(self.register_y);
    }

    fn lsr_accumulator(&mut self) {
        let data = self.register_a;
        self.status.set(StatusFlags::CARRY, data & 0b0000_0001 == 1);
        self.register_a = data >> 1;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn lsr(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let mut data = self.mem_read(addr);

        self.status.set(StatusFlags::CARRY, data & 0b0000_0001 == 1);
        data = data >> 1;
        self.mem_write(addr, data);
        self.update_zero_and_negative_flags(data);
    }

    fn ora(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let mut data = self.mem_read(addr);
        self.register_a |= data;
        self.update_zero_and_negative_flags(data);
    }

    fn pha(&mut self) {
        self.push(self.register_a);
    }

    fn php(&mut self) {
        let mut status = self.status.clone();
        status.insert(StatusFlags::BREAK);
        status.insert(StatusFlags::BREAK2);
        self.push(status.bits());
    }

    fn pla(&mut self) {
        let data = self.pop();
        self.register_a = data;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn plp(&mut self) {
        let data = self.pop();
        self.status = StatusFlags::from_bits(data).unwrap();
        self.status.remove(StatusFlags::BREAK);
        self.status.remove(StatusFlags::BREAK2);
    }

    fn rol_accumulator(&mut self) {
        let new_carry = self.register_a >> 7;
        self.register_a = self.register_a << 1
            | if self.status.contains(StatusFlags::CARRY) {
                1
            } else {
                0
            };
        self.status.set(StatusFlags::CARRY, new_carry == 1);
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn rol(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let mut data = self.mem_read(addr);
        let new_carry = data >> 7;
        data = data << 1
            | if self.status.contains(StatusFlags::CARRY) {
                1
            } else {
                0
            };
        self.status.set(StatusFlags::CARRY, new_carry == 1);
        self.mem_write(addr, data);
        self.update_zero_and_negative_flags(data);
    }

    fn ror_accumulator(&mut self) {
        let new_carry = self.register_a & 0b0000_0001 == 1;
        self.register_a = self.register_a >> 1
            | if self.status.contains(StatusFlags::CARRY) {
                0b1000_0000
            } else {
                0
            };
        self.status.set(StatusFlags::CARRY, new_carry);
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn ror(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let mut data = self.mem_read(addr);
        let new_carry = data & 0b0000_0001 == 1;
        data = data >> 1
            | if self.status.contains(StatusFlags::CARRY) {
                0b1000_0000
            } else {
                0
            };
        self.status.set(StatusFlags::CARRY, new_carry);
        self.mem_write(addr, data);
        self.update_zero_and_negative_flags(data);
    }

    fn rti(&mut self) {
        let data = self.pop();
        self.status = StatusFlags::from_bits(data).unwrap();
        self.status.remove(StatusFlags::BREAK);
        self.status.remove(StatusFlags::BREAK2);
        self.program_counter = self.pop_u16();
    }

    fn rts(&mut self) {
        self.program_counter = self.pop_u16().wrapping_add(1);
    }

    fn sec(&mut self) {
        self.status.insert(StatusFlags::CARRY);
    }

    fn sed(&mut self) {
        self.status.insert(StatusFlags::DECIMAL);
    }

    fn sei(&mut self) {
        self.status.insert(StatusFlags::INTERRUPT_DSIABLE);
    }

    fn sta(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_a);
    }

    fn stx(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_x);
    }

    fn sty(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_y);
    }

    fn tax(&mut self) {
        self.register_x = self.register_a;
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn tay(&mut self) {
        self.register_y = self.register_a;
        self.update_zero_and_negative_flags(self.register_y);
    }

    fn tsx(&mut self) {
        self.register_x = self.stack_ptr;
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn txa(&mut self) {
        self.register_a = self.register_x;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn tya(&mut self) {
        self.register_a = self.register_y;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn txs(&mut self) {
        self.stack_ptr = self.register_x;
    }

    fn update_zero_and_negative_flags(&mut self, value: u8) {
        self.status.set(StatusFlags::ZERO, value == 0);
        self.status.set(StatusFlags::NEGATIVE, value & 0b1000_0000 > 0);
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
        cpu.load_and_run(vec![
            0xa2,
            reg_x_val,
            0xa9,
            reg_a_val,
            0x95,
            destination_addr,
        ]);

        assert_eq!(cpu.register_a, reg_a_val);
        assert_eq!(
            cpu.memory[(destination_addr.wrapping_add(reg_x_val)) as usize],
            reg_a_val
        );
    }
}
