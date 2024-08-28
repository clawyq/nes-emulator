pub struct CPU {
    pub register_a: u8,
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
                0xA9 => { // LDA 
                    // Reads an extra byte for parameter 
                    let param = program[self.program_counter as usize];
                    self.program_counter += 1;

                    // load into the accumulator
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
