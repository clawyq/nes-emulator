use crate::{cpu::{AddressingMode, Mem, CPU}, opcodes::get_opcode_details};

pub fn log(cpu: &mut CPU) -> String {
    let opcode = cpu.mem_read(cpu.program_counter);

    let opcode_details = get_opcode_details(&opcode).unwrap();
    let mut log: Vec<u8> = Vec::new();
    log.push(opcode);

    let (mem_addr, value) = match opcode_details.mode {
        AddressingMode::Immediate | AddressingMode::Implied => (0, 0),
        _ => {
            let (addr, _) = cpu.get_absolute_address(&opcode_details.mode, cpu.program_counter + 1);
            (addr, cpu.mem_read(addr))
        }
    };

    let tmp = match opcode_details.additional_bytes {
        0 => match opcode_details.code {
            0x0a | 0x4a | 0x2a | 0x6a => format!("A "),
            _ => String::from(""),
        },
        1 => {
            let address: u8 = cpu.mem_read(cpu.program_counter + 1);
            log.push(address);

            match opcode_details.mode {
                AddressingMode::Immediate => format!("#${:02x}", address),
                AddressingMode::ZeroPage => format!("${:02x} = {:02x}", mem_addr, value),
                AddressingMode::ZeroPage_X => format!(
                    "${:02x},X @ {:02x} = {:02x}",
                    address, mem_addr, value
                ),
                AddressingMode::ZeroPage_Y => format!(
                    "${:02x},Y @ {:02x} = {:02x}",
                    address, mem_addr, value
                ),
                AddressingMode::Indirect_X => format!(
                    "(${:02x},X) @ {:02x} = {:04x} = {:02x}",
                    address,
                    (address.wrapping_add(cpu.register_x)),
                    mem_addr,
                    value
                ),
                AddressingMode::Indirect_Y => format!(
                    "(${:02x}),Y = {:04x} @ {:04x} = {:02x}",
                    address,
                    (mem_addr.wrapping_sub(cpu.register_y as u16)),
                    mem_addr,
                    value
                ),
                AddressingMode::Implied => {
                    // assuming local jumps: BNE, BVS, etc....
                    let address: usize =
                        (cpu.program_counter as usize + 2).wrapping_add((address as i8) as usize);
                    format!("${:04x}", address)
                }

                _ => panic!(
                    "unexpected addressing mode {:?} has opcode_details-additional_bytes 2. code {:02x}",
                    opcode_details.mode, opcode_details.code
                ),
            }
        }
        2 => {
            let address_lo = cpu.mem_read(cpu.program_counter + 1);
            let address_hi = cpu.mem_read(cpu.program_counter + 2);
            log.push(address_lo);
            log.push(address_hi);

            let address = cpu.mem_read_u16(cpu.program_counter + 1);

            match opcode_details.mode {
                AddressingMode::Implied => {
                    if opcode_details.code == 0x6c {
                        //jmp indirect
                        let jmp_addr = if address & 0x00FF == 0x00FF {
                            let lo = cpu.mem_read(address);
                            let hi = cpu.mem_read(address & 0xFF00);
                            (hi as u16) << 8 | (lo as u16)
                        } else {
                            cpu.mem_read_u16(address)
                        };

                        // let jmp_addr = cpu.mem_read_u16(address);
                        format!("(${:04x}) = {:04x}", address, jmp_addr)
                    } else {
                        format!("${:04x}", address)
                    }
                }
                AddressingMode::Absolute => format!("${:04x} = {:02x}", mem_addr, value),
                AddressingMode::Absolute_X => format!(
                    "${:04x},X @ {:04x} = {:02x}",
                    address, mem_addr, value
                ),
                AddressingMode::Absolute_Y => format!(
                    "${:04x},Y @ {:04x} = {:02x}",
                    address, mem_addr, value
                ),
                _ => panic!(
                    "unexpected addressing mode {:?} has opcode_details-additional_bytes 2. code {:02x}",
                    opcode_details.mode, opcode_details.code
                ),
            }
        }
        _ => String::from(""),
    };

    let hex_str = log
        .iter()
        .map(|z| format!("{:02x}", z))
        .collect::<Vec<String>>()
        .join(" ");
    let asm_str = format!("{:04x}  {:8} {: >4} {}", cpu.program_counter, hex_str, opcode_details.mnemonic, tmp)
        .trim()
        .to_string();

    format!(
        "{:47} A:{:02x} X:{:02x} Y:{:02x} P:{:02x} SP:{:02x}",
        asm_str, cpu.register_a, cpu.register_x, cpu.register_y, cpu.status, cpu.stack_ptr,
    )
    .to_ascii_uppercase()
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::bus::Bus;
    use crate::rom::test::test_rom;

    #[test]
    fn test_format_trace() {
        let mut bus = Bus::new(test_rom());
        bus.mem_write(100, 0xa2);
        bus.mem_write(101, 0x01);
        bus.mem_write(102, 0xca);
        bus.mem_write(103, 0x88);
        bus.mem_write(104, 0x00);

        let mut cpu = CPU::new(bus);
        cpu.program_counter = 0x64;
        cpu.register_a = 1;
        cpu.register_x = 2;
        cpu.register_y = 3;
        let mut result: Vec<String> = vec![];
        cpu.run_with_callback(|cpu| {
            result.push(log(cpu));
        });
        assert_eq!(
            "0064  A2 01     LDX #$01                        A:01 X:02 Y:03 P:24 SP:FD",
            result[0]
        );
        assert_eq!(
            "0066  CA        DEX                             A:01 X:01 Y:03 P:24 SP:FD",
            result[1]
        );
        assert_eq!(
            "0067  88        DEY                             A:01 X:00 Y:03 P:26 SP:FD",
            result[2]
        );
    }

    #[test]
    fn test_format_mem_access() {
        let mut bus = Bus::new(test_rom());
        // ORA ($33), Y
        bus.mem_write(100, 0x11);
        bus.mem_write(101, 0x33);

        //data
        bus.mem_write(0x33, 00);
        bus.mem_write(0x34, 04);

        //target cell
        bus.mem_write(0x400, 0xAA);

        let mut cpu = CPU::new(bus);
        cpu.program_counter = 0x64;
        cpu.register_y = 0;
        let mut result: Vec<String> = vec![];
        cpu.run_with_callback(|cpu| {
            result.push(log(cpu));
        });
        assert_eq!(
            "0064  11 33     ORA ($33),Y = 0400 @ 0400 = AA  A:00 X:00 Y:00 P:24 SP:FD",
            result[0]
        );
    }
}
