pub struct CPU {
    pub register_a: u8,
    pub register_x: u8,
    pub status: u8, // N V NOT_USED B D I Z C
    pub program_counter: u16,
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
            status: 0,
            program_counter: 0,
        }
    }

    pub fn interpret(&mut self, program: Vec<u8>) {
        self.program_counter = 0;

        loop {
            let opscode = program[self.program_counter as usize];
            self.program_counter += 1;
            match opscode {
                0x00 => { return; },
                0xA2 => { // LDX
                    // Reads an extra byte for parameter 
                    let param = program[self.program_counter as usize];
                    self.program_counter += 1;

                    // load into x
                    self.register_x = param;

                    if self.register_x == 0 {
                        self.status = self.status | 0b0000_0010;
                    } else {
                        self.status = self.status & 0b1111_1101;
                    }

                    // Setting the N(egative) flag
                    if self.register_x & 0b1000_0000 != 0 {
                        self.status = self.status | 0b1000_0000;
                    } else {
                        self.status = self.status & 0b0111_1111;
                    }
                },
                0xA9 => { // LDA
                    // Reads an extra byte for parameter 
                    let param = program[self.program_counter as usize];
                    self.program_counter += 1;

                    // load into accumulator
                    self.register_a = param;

                    // setting the Z(ero) flag
                    if self.register_a == 0 {
                        self.status = self.status | 0b0000_0010;
                    } else {
                        self.status = self.status & 0b1111_1101;
                    }

                    // Setting the N(egative) flag
                    if self.register_a & 0b1000_0000 != 0 {
                        self.status = self.status | 0b1000_0000;
                    } else {
                        self.status = self.status & 0b0111_1111;
                    }
                },
                _ => todo!()
            }
        }
    }
}

#[cfg(test)]
mod test {
   use super::*;
 
   #[test]
   fn test_0xa9_lda_immediate_load_data() {
       let mut cpu = CPU::new();
       cpu.interpret(vec![0xa9, 0x05, 0x00]);
       assert_eq!(cpu.register_a, 0b0000_0101);
       assert_eq!(cpu.status, 0);
       assert!(cpu.status & 0b0000_0010 == 0b00); // Zero unset?
       assert!(cpu.status & 0b1000_0000 == 0); // Negative unset?
   }

    #[test]
    fn test_0xa9_lda_zero_flag() {
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xa9, 0x00, 0x00]);
       assert_eq!(cpu.register_a, 0x00);
       assert_eq!(cpu.status, 2);
       assert!(cpu.status & 0b0000_0010 == 0b10); // Zero set?
       assert!(cpu.status & 0b1000_0000 == 0); // Negative unset?
    }

    #[test]
    fn test_0xa9_lda_negative_flag() {
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xa9, 0x09, 0x00]);
       assert_eq!(cpu.register_a, 0x09);
       assert_eq!(cpu.status, 0);
       assert!(cpu.status & 0b0000_0010 == 0b00); // Zero set?
       assert!(cpu.status & 0b1000_0000 == 0); // Negative unset?
    }

    #[test]
    fn test_0xa2_ldx_immediate_load_data() {
        let mut cpu = CPU::new();
        cpu.interpret(vec![0xa2, 0x05, 0x00]);
        assert_eq!(cpu.register_x, 0b0000_0101);
        assert_eq!(cpu.status, 0);
        assert!(cpu.status & 0b0000_0010 == 0b00); // Zero unset?
        assert!(cpu.status & 0b1000_0000 == 0); // Negative unset?
    }
 
     #[test]
     fn test_0xa2_ldx_zero_flag() {
         let mut cpu = CPU::new();
         cpu.interpret(vec![0xa2, 0x00, 0x00]);
        assert_eq!(cpu.register_x, 0x00);
        assert_eq!(cpu.status, 2);
        assert!(cpu.status & 0b0000_0010 == 0b10); // Zero set?
        assert!(cpu.status & 0b1000_0000 == 0); // Negative unset?
     }
 
     #[test]
     fn test_0xa2_ldx_negative_flag() {
         let mut cpu = CPU::new();
         cpu.interpret(vec![0xa2, 0x09, 0x00]);
        assert_eq!(cpu.register_x, 0x09);
        assert_eq!(cpu.status, 0);
        assert!(cpu.status & 0b0000_0010 == 0b00); // Zero set?
        assert!(cpu.status & 0b1000_0000 == 0); // Negative unset?
     }
}
