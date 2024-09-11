use crate::{bus::Bus, opcodes::get_opcode_details};
use bitflags::bitflags;

bitflags! {
    // N V B2 B D I Z C
    #[derive(Debug, Clone, PartialEq)]
    pub struct StatusFlags: u8 {
        const CARRY             = 0b0000_0001;
        const ZERO              = 0b0000_0010;
        const INTERRUPT_DISABLE = 0b0000_0100;
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
    fn mem_read(&mut self, addr: u16) -> u8;

    fn mem_write(&mut self, addr: u16, data: u8) {
        panic!("Attempted to write data to a read-only address.");
    }

    fn mem_read_u16(&mut self, pos: u16) -> u16 {
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
    fn mem_read(&mut self, addr: u16) -> u8 {
        self.bus.mem_read(addr)
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        self.bus.mem_write(addr, data);
    }

    fn mem_read_u16(&mut self, pos: u16) -> u16 {
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
    pub fn new(bus: Bus) -> Self {
        CPU {
            register_a: 0,
            register_x: 0,
            register_y: 0,
            status: StatusFlags::from_bits_truncate(0b100100),
            program_counter: 0,
            stack_ptr: STACK_PTR_INIT,
            bus,
        }
    }

    /**
     * Addressing modes are cracked https://skilldrick.github.io/easy6502/#addressing.
     * Depending on context, we interpret the subsequent 1/2/3 bytes differently
     * to find the value we need as an operand for our command.
     */
    pub fn get_absolute_address(&mut self, mode: &AddressingMode, addr: u16) -> u16 {
        match mode {
            AddressingMode::Immediate => addr,
            AddressingMode::ZeroPage => self.mem_read(addr) as u16,
            AddressingMode::ZeroPage_X => {
                let addr = self.mem_read(addr);
                addr.wrapping_add(self.register_x) as u16
            }
            AddressingMode::ZeroPage_Y => {
                let addr = self.mem_read(addr);
                addr.wrapping_add(self.register_y) as u16
            }
            AddressingMode::Absolute => self.mem_read_u16(addr),
            AddressingMode::Absolute_X => {
                let addr = self.mem_read_u16(addr);
                addr.wrapping_add(self.register_x as u16) as u16
            }
            AddressingMode::Absolute_Y => {
                let addr = self.mem_read_u16(addr);
                addr.wrapping_add(self.register_y as u16) as u16
            }
            AddressingMode::Indirect => {
                let addr = self.mem_read_u16(addr);
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
                let addr: u8 = self.mem_read(addr);
                let x_addr = addr.wrapping_add(self.register_x);
                u16::from_le_bytes([
                    self.mem_read(x_addr as u16),
                    self.mem_read(x_addr.wrapping_add(1) as u16),
                ])
            }
            AddressingMode::Indirect_Y => {
                let addr = self.mem_read(addr);
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

    pub fn get_operand_address(&mut self, mode: &AddressingMode) -> u16 {
        self.get_absolute_address(mode, self.program_counter)
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
        self.register_y = 0;
        self.status = StatusFlags::from_bits_truncate(0b100100);
        self.stack_ptr = STACK_PTR_INIT;
        self.program_counter = self.mem_read_u16(0xFFFC);
    }

    pub fn load_and_run(&mut self, program: Vec<u8>) {
        self.load(program);
        self.reset();
        self.program_counter = 0x0600;
        self.run_with_callback(|_| {});
    }

    pub fn load(&mut self, program: Vec<u8>) {
        program.iter().enumerate().for_each(|(i, &byte)| {
            self.mem_write(0x0600 + i as u16, byte);
        });
    }

    fn has_jumped_or_branched(&self, other_addr: u16) -> bool {
        self.program_counter != other_addr
    }

    pub fn run_with_callback<F>(&mut self, mut callback: F)
    where
        F: FnMut(&mut CPU),
    {
        loop {
            callback(self);
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
                0x69 | 0x65 | 0x75 | 0x6D | 0x7D | 0x79 | 0x61 | 0x71 => {
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
                    self.status.remove(StatusFlags::INTERRUPT_DISABLE);
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
                0x4C => {
                    self.program_counter = self.mem_read_u16(self.program_counter);
                }
                0x6C => {
                    let addr = self.mem_read_u16(self.program_counter);
                    let indirect_ref = if addr & 0x00FF == 0x00FF {
                        let lo = self.mem_read(addr);
                        let hi = self.mem_read(addr & 0xFF00);
                        (hi as u16) << 8 | (lo as u16)
                    } else {
                        self.mem_read_u16(addr)
                    };

                    self.program_counter = indirect_ref;
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
                /* DCP */
                0xc7 | 0xd7 | 0xCF | 0xdF | 0xdb | 0xd3 | 0xc3 => {
                    let addr = self.get_operand_address(mode);
                    let mut data = self.mem_read(addr);
                    data = data.wrapping_sub(1);
                    self.mem_write(addr, data);
                    // self._update_zero_and_negative_flags(data);
                    if data <= self.register_a {
                        self.status.insert(StatusFlags::CARRY);
                    }

                    self.update_zero_and_negative_flags(self.register_a.wrapping_sub(data));
                }

                /* RLA */
                0x27 | 0x37 | 0x2F | 0x3F | 0x3b | 0x33 | 0x23 => {
                    let data = self.rol(mode);
                    self.and_with_register_a(data);
                }

                /* SLO */ //todo tests
                0x07 | 0x17 | 0x0F | 0x1f | 0x1b | 0x03 | 0x13 => {
                    let data = self.asl(mode);
                    self.or_with_register_a(data);
                }

                /* SRE */ //todo tests
                0x47 | 0x57 | 0x4F | 0x5f | 0x5b | 0x43 | 0x53 => {
                    let data = self.lsr(mode);
                    self.xor_with_register_a(data);
                }

                /* SKB */
                0x80 | 0x82 | 0x89 | 0xc2 | 0xe2 => {
                    /* 2 byte NOP (immediate ) */
                    // todo: might be worth doing the read
                }

                /* AXS */
                0xCB => {
                    let addr = self.get_operand_address(mode);
                    let data = self.mem_read(addr);
                    let x_and_a = self.register_x & self.register_a;
                    let result = x_and_a.wrapping_sub(data);

                    if data <= x_and_a {
                        self.status.insert(StatusFlags::CARRY);
                    }
                    self.update_zero_and_negative_flags(result);

                    self.register_x = result;
                }

                /* ARR */
                0x6B => {
                    let addr = self.get_operand_address(mode);
                    let data = self.mem_read(addr);
                    self.and_with_register_a(data);
                    self.ror_accumulator();
                    //todo: registers
                    let result = self.register_a;
                    let bit_5 = (result >> 5) & 1;
                    let bit_6 = (result >> 6) & 1;

                    if bit_6 == 1 {
                        self.status.insert(StatusFlags::CARRY)
                    } else {
                        self.status.remove(StatusFlags::CARRY)
                    }

                    if bit_5 ^ bit_6 == 1 {
                        self.status.insert(StatusFlags::OVERFLOW);
                    } else {
                        self.status.remove(StatusFlags::OVERFLOW);
                    }

                    self.update_zero_and_negative_flags(result);
                }

                /* unofficial SBC */
                0xeb => {
                    let addr = self.get_operand_address(mode);
                    let data = self.mem_read(addr);
                    self.sub_from_register_a(data);
                }

                /* ANC */
                0x0b | 0x2b => {
                    let addr = self.get_operand_address(mode);
                    let data = self.mem_read(addr);
                    self.and_with_register_a(data);
                    if self.status.contains(StatusFlags::NEGATIVE) {
                        self.status.insert(StatusFlags::CARRY);
                    } else {
                        self.status.remove(StatusFlags::CARRY);
                    }
                }

                /* ALR */
                0x4b => {
                    let addr = self.get_operand_address(mode);
                    let data = self.mem_read(addr);
                    self.and_with_register_a(data);
                    self.lsr_accumulator();
                }

                //todo: test for everything bellow

                /* NOP read */
                0x04 | 0x44 | 0x64 | 0x14 | 0x34 | 0x54 | 0x74 | 0xd4 | 0xf4 | 0x0c | 0x1c
                | 0x3c | 0x5c | 0x7c | 0xdc | 0xfc => {
                    let data = self.mem_read(self.program_counter);
                    /* do nothing */
                }

                /* RRA */
                0x67 | 0x77 | 0x6f | 0x7f | 0x7b | 0x63 | 0x73 => {
                    let data = self.ror(mode);
                    self.add_to_register_a(data);
                }

                /* ISB */
                0xe7 | 0xf7 | 0xef | 0xff | 0xfb | 0xe3 | 0xf3 => {
                    let data = self.inc(mode);
                    self.sub_from_register_a(data);
                }

                /* NOPs */
                0x02 | 0x12 | 0x22 | 0x32 | 0x42 | 0x52 | 0x62 | 0x72 | 0x92 | 0xb2 | 0xd2
                | 0xf2 => { /* do nothing */ }

                0x1a | 0x3a | 0x5a | 0x7a | 0xda | 0xfa => { /* do nothing */ }

                /* LAX */
                0xa7 | 0xb7 | 0xaf | 0xbf | 0xa3 | 0xb3 => {
                    let addr = self.get_operand_address(mode);
                    let data = self.mem_read(addr);
                    self.set_register_a(data);
                    self.register_x = self.register_a;
                }

                /* SAX */
                0x87 | 0x97 | 0x8f | 0x83 => {
                    let data = self.register_a & self.register_x;
                    let addr = self.get_operand_address(mode);
                    self.mem_write(addr, data);
                }

                /* LXA */
                0xab => {
                    self.lda(mode);
                    self.tax();
                }

                /* XAA */
                0x8b => {
                    self.register_a = self.register_x;
                    self.update_zero_and_negative_flags(self.register_a);
                    let addr = self.get_operand_address(mode);
                    let data = self.mem_read(addr);
                    self.and_with_register_a(data);
                }

                /* LAS */
                0xbb => {
                    let addr = self.get_operand_address(mode);
                    let mut data = self.mem_read(addr);
                    data = data & self.stack_ptr;
                    self.register_a = data;
                    self.register_x = data;
                    self.stack_ptr = data;
                    self.update_zero_and_negative_flags(data);
                }

                /* TAS */
                0x9b => {
                    let data = self.register_a & self.register_x;
                    self.stack_ptr = data;
                    let mem_address =
                        self.mem_read_u16(self.program_counter) + self.register_y as u16;

                    let data = ((mem_address >> 8) as u8 + 1) & self.stack_ptr;
                    self.mem_write(mem_address, data)
                }

                /* AHX  Indirect Y */
                0x93 => {
                    let pos: u8 = self.mem_read(self.program_counter);
                    let mem_address = self.mem_read_u16(pos as u16) + self.register_y as u16;
                    let data = self.register_a & self.register_x & (mem_address >> 8) as u8;
                    self.mem_write(mem_address, data)
                }

                /* AHX Absolute Y*/
                0x9f => {
                    let mem_address =
                        self.mem_read_u16(self.program_counter) + self.register_y as u16;

                    let data = self.register_a & self.register_x & (mem_address >> 8) as u8;
                    self.mem_write(mem_address, data)
                }

                /* SHX */
                0x9e => {
                    let mem_address =
                        self.mem_read_u16(self.program_counter) + self.register_y as u16;

                    // todo if cross page boundry {
                    //     mem_address &= (self.x as u16) << 8;
                    // }
                    let data = self.register_x & ((mem_address >> 8) as u8 + 1);
                    self.mem_write(mem_address, data)
                }

                /* SHY */
                0x9c => {
                    let mem_address =
                        self.mem_read_u16(self.program_counter) + self.register_x as u16;
                    let data = self.register_y & ((mem_address >> 8) as u8 + 1);
                    self.mem_write(mem_address, data)
                }
                _ => todo!(),
            }
            if !self.has_jumped_or_branched(program_counter_before_exec) {
                self.program_counter += opcode_details.additional_bytes as u16;
            }
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

    fn asl(&mut self, mode: &AddressingMode) -> u8 {
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
        result
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
        self.register_a = self.register_a ^ data;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn inc(&mut self, mode: &AddressingMode) -> u8 {
        let addr = self.get_operand_address(mode);
        let mut data = self.mem_read(addr);
        data = data.wrapping_add(1);
        self.mem_write(addr, data);
        self.update_zero_and_negative_flags(data);
        data
    }

    fn inx(&mut self) {
        self.register_x = self.register_x.wrapping_add(1);
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn iny(&mut self) {
        self.register_y = self.register_y.wrapping_add(1);
        self.update_zero_and_negative_flags(self.register_y);
    }

    fn jsr(&mut self, mode: &AddressingMode) {
        self.push_u16(self.program_counter + 1); // stack now has the last byte of the JSR arg -> next execution i will + 1 so i will be at the right instruction
        let addr = self.mem_read_u16(self.program_counter);
        self.program_counter = addr;
    }

    fn lda(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let data = self.mem_read(addr);
        self.set_register_a(data);
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

    fn lsr(&mut self, mode: &AddressingMode) -> u8 {
        let addr = self.get_operand_address(mode);
        let mut data = self.mem_read(addr);

        self.status.set(StatusFlags::CARRY, data & 0b0000_0001 == 1);
        data = data >> 1;
        self.mem_write(addr, data);
        self.update_zero_and_negative_flags(data);
        data
    }

    fn ora(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let data = self.mem_read(addr);
        self.set_register_a(self.register_a | data);
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
        self.status.insert(StatusFlags::BREAK2);
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

    fn rol(&mut self, mode: &AddressingMode) -> u8 {
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
        data
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

    fn ror(&mut self, mode: &AddressingMode) -> u8 {
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
        data
    }

    fn rti(&mut self) {
        let data = self.pop();
        self.status = StatusFlags::from_bits(data).unwrap();
        self.status.remove(StatusFlags::BREAK);
        self.status.insert(StatusFlags::BREAK2);
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
        self.status.insert(StatusFlags::INTERRUPT_DISABLE);
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
        self.status
            .set(StatusFlags::NEGATIVE, value & 0b1000_0000 > 0);
    }

    fn add_to_register_a(&mut self, data: u8) {
        let sum = self.register_a as u16
            + data as u16
            + (if self.status.contains(StatusFlags::CARRY) {
                1
            } else {
                0
            }) as u16;

        let carry = sum > 0xff;

        if carry {
            self.status.insert(StatusFlags::CARRY);
        } else {
            self.status.remove(StatusFlags::CARRY);
        }

        let result = sum as u8;

        if (data ^ result) & (result ^ self.register_a) & 0x80 != 0 {
            self.status.insert(StatusFlags::OVERFLOW);
        } else {
            self.status.remove(StatusFlags::OVERFLOW)
        }

        self.set_register_a(result);
    }

    fn set_register_a(&mut self, value: u8) {
        self.register_a = value;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn sub_from_register_a(&mut self, data: u8) {
        self.add_to_register_a(((data as i8).wrapping_neg().wrapping_sub(1)) as u8);
    }

    fn and_with_register_a(&mut self, data: u8) {
        self.set_register_a(data & self.register_a);
    }

    fn xor_with_register_a(&mut self, data: u8) {
        self.set_register_a(data ^ self.register_a);
    }

    fn or_with_register_a(&mut self, data: u8) {
        self.set_register_a(data | self.register_a);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::rom::test;

    #[test]
    fn test_0xa9_lda_immediate_load_data() {
        let mut cpu = CPU::new(Bus::new(test::test_rom()));
        cpu.load_and_run(vec![0xa9, 0x05, 0x00]);
        assert_eq!(cpu.register_a, 0b0000_0101);
        assert!(!cpu.status.contains(StatusFlags::ZERO));
        assert!(!cpu.status.contains(StatusFlags::NEGATIVE));
    }

    #[test]
    fn test_0xa9_lda_zero_flag() {
        let mut cpu = CPU::new(Bus::new(test::test_rom()));
        cpu.load_and_run(vec![0xa9, 0x00, 0x00]);
        assert_eq!(cpu.register_a, 0x00);
        assert!(cpu.status.contains(StatusFlags::ZERO));
        assert!(!cpu.status.contains(StatusFlags::NEGATIVE));
    }

    #[test]
    fn test_0xa9_lda_negative_flag() {
        let mut cpu = CPU::new(Bus::new(test::test_rom()));
        cpu.load_and_run(vec![0xa9, 0x09, 0x00]);
        assert_eq!(cpu.register_a, 0x09);
        assert!(!cpu.status.contains(StatusFlags::ZERO));
        assert!(!cpu.status.contains(StatusFlags::NEGATIVE));
    }

    #[test]
    fn test_0xa2_ldx_immediate_load_data() {
        let mut cpu = CPU::new(Bus::new(test::test_rom()));
        cpu.load_and_run(vec![0xa2, 0x05, 0x00]);
        assert_eq!(cpu.register_x, 0b0000_0101);
        assert!(!cpu.status.contains(StatusFlags::ZERO));
        assert!(!cpu.status.contains(StatusFlags::NEGATIVE));
    }

    #[test]
    fn test_0xa2_ldx_zero_flag() {
        let mut cpu = CPU::new(Bus::new(test::test_rom()));
        cpu.load_and_run(vec![0xa2, 0x00, 0x00]);
        assert_eq!(cpu.register_x, 0x00);
        assert!(cpu.status.contains(StatusFlags::ZERO));
        assert!(!cpu.status.contains(StatusFlags::NEGATIVE));
    }

    #[test]
    fn test_0xa2_ldx_negative_flag() {
        let mut cpu = CPU::new(Bus::new(test::test_rom()));
        cpu.load_and_run(vec![0xa2, 0x09, 0x00]);
        assert_eq!(cpu.register_x, 0x09);
        assert!(!cpu.status.contains(StatusFlags::ZERO));
        assert!(!cpu.status.contains(StatusFlags::NEGATIVE));
    }

    #[test]
    fn test_0xaa_tax_move_a_to_x() {
        let mut cpu = CPU::new(Bus::new(test::test_rom()));
        cpu.load_and_run(vec![0xa9, 10, 0xaa, 0x00]);

        assert_eq!(cpu.register_x, 10);
        assert!(!cpu.status.contains(StatusFlags::ZERO));
        assert!(!cpu.status.contains(StatusFlags::NEGATIVE));
    }

    #[test]
    fn test_5_ops_working_together() {
        let mut cpu = CPU::new(Bus::new(test::test_rom()));
        cpu.load_and_run(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);

        assert_eq!(cpu.register_x, 0xc1)
    }

    #[test]
    fn test_inx_overflow() {
        let mut cpu = CPU::new(Bus::new(test::test_rom()));
        cpu.load_and_run(vec![0xa2, 0xff, 0xe8, 0xe8, 0x00]);

        assert_eq!(cpu.register_x, 1)
    }

    #[test]
    fn test_0x85_sta() {
        let mut cpu = CPU::new(Bus::new(test::test_rom()));
        let reg_a_val = 0x09;
        let destination_addr = 0x28;
        cpu.load_and_run(vec![0xa9, reg_a_val, 0x85, destination_addr]);

        assert_eq!(cpu.register_a, reg_a_val);
        assert_eq!(cpu.mem_read(destination_addr as u16), reg_a_val);
    }

    #[test]
    fn test_0x95_sta() {
        let mut cpu = CPU::new(Bus::new(test::test_rom()));
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
            cpu.mem_read((destination_addr as u16).wrapping_add(reg_x_val as u16)),
            reg_a_val
        );
    }
}
