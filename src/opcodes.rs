use crate::cpu::AddressingMode;
use phf::phf_map;

#[derive(Debug)]
pub struct OpCode {
    pub code: u8,
    pub mnemonic: &'static str,
    pub additional_bytes: u8,
    pub cycles: u8,
    pub mode: AddressingMode,
}

impl OpCode {
    pub const fn new(
        code: u8,
        mnemonic: &'static str,
        additional_bytes: u8,
        cycles: u8,
        mode: AddressingMode,
    ) -> Self {
        OpCode {
            code,
            mnemonic,
            additional_bytes,
            cycles,
            mode,
        }
    }
}

static OP_CODES_MAP: phf::Map<u8, OpCode> = phf_map! {
    0x00u8 => OpCode::new(0x00, "BRK", 0, 7, AddressingMode::Implied),
    0xeau8 => OpCode::new(0xea, "NOP", 0, 2, AddressingMode::Implied),

    /* Arithmetic */
    0x69u8 => OpCode::new(0x69, "ADC", 1, 2, AddressingMode::Immediate),
    0x65u8 => OpCode::new(0x65, "ADC", 1, 3, AddressingMode::ZeroPage),
    0x75u8 => OpCode::new(0x75, "ADC", 1, 4, AddressingMode::ZeroPage_X),
    0x6du8 => OpCode::new(0x6d, "ADC", 2, 4, AddressingMode::Absolute),
    0x7du8 => OpCode::new(0x7d, "ADC", 2, 4/*+1 if page crossed*/, AddressingMode::Absolute_X),
    0x79u8 => OpCode::new(0x79, "ADC", 2, 4/*+1 if page crossed*/, AddressingMode::Absolute_Y),
    0x61u8 => OpCode::new(0x61, "ADC", 1, 6, AddressingMode::Indirect_X),
    0x71u8 => OpCode::new(0x71, "ADC", 1, 5/*+1 if page crossed*/, AddressingMode::Indirect_Y),

    0xe9u8 => OpCode::new(0xe9, "SBC", 1, 2, AddressingMode::Immediate),
    0xe5u8 => OpCode::new(0xe5, "SBC", 1, 3, AddressingMode::ZeroPage),
    0xf5u8 => OpCode::new(0xf5, "SBC", 1, 4, AddressingMode::ZeroPage_X),
    0xedu8 => OpCode::new(0xed, "SBC", 2, 4, AddressingMode::Absolute),
    0xfdu8 => OpCode::new(0xfd, "SBC", 2, 4/*+1 if page crossed*/, AddressingMode::Absolute_X),
    0xf9u8 => OpCode::new(0xf9, "SBC", 2, 4/*+1 if page crossed*/, AddressingMode::Absolute_Y),
    0xe1u8 => OpCode::new(0xe1, "SBC", 1, 6, AddressingMode::Indirect_X),
    0xf1u8 => OpCode::new(0xf1, "SBC", 1, 5/*+1 if page crossed*/, AddressingMode::Indirect_Y),

    0x29u8 => OpCode::new(0x29, "AND", 1, 2, AddressingMode::Immediate),
    0x25u8 => OpCode::new(0x25, "AND", 1, 3, AddressingMode::ZeroPage),
    0x35u8 => OpCode::new(0x35, "AND", 1, 4, AddressingMode::ZeroPage_X),
    0x2du8 => OpCode::new(0x2d, "AND", 2, 4, AddressingMode::Absolute),
    0x3du8 => OpCode::new(0x3d, "AND", 2, 4/*+1 if page crossed*/, AddressingMode::Absolute_X),
    0x39u8 => OpCode::new(0x39, "AND", 2, 4/*+1 if page crossed*/, AddressingMode::Absolute_Y),
    0x21u8 => OpCode::new(0x21, "AND", 1, 6, AddressingMode::Indirect_X),
    0x31u8 => OpCode::new(0x31, "AND", 1, 5/*+1 if page crossed*/, AddressingMode::Indirect_Y),

    0x49u8 => OpCode::new(0x49, "EOR", 1, 2, AddressingMode::Immediate),
    0x45u8 => OpCode::new(0x45, "EOR", 1, 3, AddressingMode::ZeroPage),
    0x55u8 => OpCode::new(0x55, "EOR", 1, 4, AddressingMode::ZeroPage_X),
    0x4du8 => OpCode::new(0x4d, "EOR", 2, 4, AddressingMode::Absolute),
    0x5du8 => OpCode::new(0x5d, "EOR", 2, 4/*+1 if page crossed*/, AddressingMode::Absolute_X),
    0x59u8 => OpCode::new(0x59, "EOR", 2, 4/*+1 if page crossed*/, AddressingMode::Absolute_Y),
    0x41u8 => OpCode::new(0x41, "EOR", 1, 6, AddressingMode::Indirect_X),
    0x51u8 => OpCode::new(0x51, "EOR", 1, 5/*+1 if page crossed*/, AddressingMode::Indirect_Y),

    0x09u8 => OpCode::new(0x09, "ORA", 1, 2, AddressingMode::Immediate),
    0x05u8 => OpCode::new(0x05, "ORA", 1, 3, AddressingMode::ZeroPage),
    0x15u8 => OpCode::new(0x15, "ORA", 1, 4, AddressingMode::ZeroPage_X),
    0x0du8 => OpCode::new(0x0d, "ORA", 2, 4, AddressingMode::Absolute),
    0x1du8 => OpCode::new(0x1d, "ORA", 2, 4/*+1 if page crossed*/, AddressingMode::Absolute_X),
    0x19u8 => OpCode::new(0x19, "ORA", 2, 4/*+1 if page crossed*/, AddressingMode::Absolute_Y),
    0x01u8 => OpCode::new(0x01, "ORA", 1, 6, AddressingMode::Indirect_X),
    0x11u8 => OpCode::new(0x11, "ORA", 1, 5/*+1 if page crossed*/, AddressingMode::Indirect_Y),

    /* Shifts */
    0x0au8 => OpCode::new(0x0a, "ASL", 0, 2, AddressingMode::Implied),
    0x06u8 => OpCode::new(0x06, "ASL", 1, 5, AddressingMode::ZeroPage),
    0x16u8 => OpCode::new(0x16, "ASL", 1, 6, AddressingMode::ZeroPage_X),
    0x0eu8 => OpCode::new(0x0e, "ASL", 2, 6, AddressingMode::Absolute),
    0x1eu8 => OpCode::new(0x1e, "ASL", 2, 7, AddressingMode::Absolute_X),

    0x4au8 => OpCode::new(0x4a, "LSR", 0, 2, AddressingMode::Implied),
    0x46u8 => OpCode::new(0x46, "LSR", 1, 5, AddressingMode::ZeroPage),
    0x56u8 => OpCode::new(0x56, "LSR", 1, 6, AddressingMode::ZeroPage_X),
    0x4eu8 => OpCode::new(0x4e, "LSR", 2, 6, AddressingMode::Absolute),
    0x5eu8 => OpCode::new(0x5e, "LSR", 2, 7, AddressingMode::Absolute_X),

    0x2au8 => OpCode::new(0x2a, "ROL", 0, 2, AddressingMode::Implied),
    0x26u8 => OpCode::new(0x26, "ROL", 1, 5, AddressingMode::ZeroPage),
    0x36u8 => OpCode::new(0x36, "ROL", 1, 6, AddressingMode::ZeroPage_X),
    0x2eu8 => OpCode::new(0x2e, "ROL", 2, 6, AddressingMode::Absolute),
    0x3eu8 => OpCode::new(0x3e, "ROL", 2, 7, AddressingMode::Absolute_X),

    0x6au8 => OpCode::new(0x6a, "ROR", 0, 2, AddressingMode::Implied),
    0x66u8 => OpCode::new(0x66, "ROR", 1, 5, AddressingMode::ZeroPage),
    0x76u8 => OpCode::new(0x76, "ROR", 1, 6, AddressingMode::ZeroPage_X),
    0x6eu8 => OpCode::new(0x6e, "ROR", 2, 6, AddressingMode::Absolute),
    0x7eu8 => OpCode::new(0x7e, "ROR", 2, 7, AddressingMode::Absolute_X),

    0xe6u8 => OpCode::new(0xe6, "INC", 1, 5, AddressingMode::ZeroPage),
    0xf6u8 => OpCode::new(0xf6, "INC", 1, 6, AddressingMode::ZeroPage_X),
    0xeeu8 => OpCode::new(0xee, "INC", 2, 6, AddressingMode::Absolute),
    0xfeu8 => OpCode::new(0xfe, "INC", 2, 7, AddressingMode::Absolute_X),

    0xe8u8 => OpCode::new(0xe8, "INX", 0, 2, AddressingMode::Implied),
    0xc8u8 => OpCode::new(0xc8, "INY", 0, 2, AddressingMode::Implied),

    0xc6u8 => OpCode::new(0xc6, "DEC", 1, 5, AddressingMode::ZeroPage),
    0xd6u8 => OpCode::new(0xd6, "DEC", 1, 6, AddressingMode::ZeroPage_X),
    0xceu8 => OpCode::new(0xce, "DEC", 2, 6, AddressingMode::Absolute),
    0xdeu8 => OpCode::new(0xde, "DEC", 2, 7, AddressingMode::Absolute_X),

    0xcau8 => OpCode::new(0xca, "DEX", 0, 2, AddressingMode::Implied),
    0x88u8 => OpCode::new(0x88, "DEY", 0, 2, AddressingMode::Implied),

    0xc9u8 => OpCode::new(0xc9, "CMP", 1, 2, AddressingMode::Immediate),
    0xc5u8 => OpCode::new(0xc5, "CMP", 1, 3, AddressingMode::ZeroPage),
    0xd5u8 => OpCode::new(0xd5, "CMP", 1, 4, AddressingMode::ZeroPage_X),
    0xcdu8 => OpCode::new(0xcd, "CMP", 2, 4, AddressingMode::Absolute),
    0xddu8 => OpCode::new(0xdd, "CMP", 2, 4/*+1 if page crossed*/, AddressingMode::Absolute_X),
    0xd9u8 => OpCode::new(0xd9, "CMP", 2, 4/*+1 if page crossed*/, AddressingMode::Absolute_Y),
    0xc1u8 => OpCode::new(0xc1, "CMP", 1, 6, AddressingMode::Indirect_X),
    0xd1u8 => OpCode::new(0xd1, "CMP", 1, 5/*+1 if page crossed*/, AddressingMode::Indirect_Y),

    0xc0u8 => OpCode::new(0xc0, "CPY", 1, 2, AddressingMode::Immediate),
    0xc4u8 => OpCode::new(0xc4, "CPY", 1, 3, AddressingMode::ZeroPage),
    0xccu8 => OpCode::new(0xcc, "CPY", 2, 4, AddressingMode::Absolute),

    0xe0u8 => OpCode::new(0xe0, "CPX", 1, 2, AddressingMode::Immediate),
    0xe4u8 => OpCode::new(0xe4, "CPX", 1, 3, AddressingMode::ZeroPage),
    0xecu8 => OpCode::new(0xec, "CPX", 2, 4, AddressingMode::Absolute),


    /* Branching */

    0x4cu8 => OpCode::new(0x4c, "JMP", 2, 3, AddressingMode::Absolute), //AddressingMode that acts as Immidiate
    0x6cu8 => OpCode::new(0x6c, "JMP", 2, 5, AddressingMode::Indirect), //AddressingMode:Indirect with 6502 bug

    0x20u8 => OpCode::new(0x20, "JSR", 2, 6, AddressingMode::Absolute),
    0x60u8 => OpCode::new(0x60, "RTS", 0, 6, AddressingMode::Implied),

    0x40u8 => OpCode::new(0x40, "RTI", 0, 6, AddressingMode::Implied),

    0xd0u8 => OpCode::new(0xd0, "BNE", 1, 2 /*(+1 if branch succeeds +2 if to a new page)*/, AddressingMode::Implied),
    0x70u8 => OpCode::new(0x70, "BVS", 1, 2 /*(+1 if branch succeeds +2 if to a new page)*/, AddressingMode::Implied),
    0x50u8 => OpCode::new(0x50, "BVC", 1, 2 /*(+1 if branch succeeds +2 if to a new page)*/, AddressingMode::Implied),
    0x30u8 => OpCode::new(0x30, "BMI", 1, 2 /*(+1 if branch succeeds +2 if to a new page)*/, AddressingMode::Implied),
    0xf0u8 => OpCode::new(0xf0, "BEQ", 1, 2 /*(+1 if branch succeeds +2 if to a new page)*/, AddressingMode::Implied),
    0xb0u8 => OpCode::new(0xb0, "BCS", 1, 2 /*(+1 if branch succeeds +2 if to a new page)*/, AddressingMode::Implied),
    0x90u8 => OpCode::new(0x90, "BCC", 1, 2 /*(+1 if branch succeeds +2 if to a new page)*/, AddressingMode::Implied),
    0x10u8 => OpCode::new(0x10, "BPL", 1, 2 /*(+1 if branch succeeds +2 if to a new page)*/, AddressingMode::Implied),

    0x24u8 => OpCode::new(0x24, "BIT", 1, 3, AddressingMode::ZeroPage),
    0x2cu8 => OpCode::new(0x2c, "BIT", 2, 4, AddressingMode::Absolute),


    /* Stores, Loads */
    0xa9u8 => OpCode::new(0xa9, "LDA", 1, 2, AddressingMode::Immediate),
    0xa5u8 => OpCode::new(0xa5, "LDA", 1, 3, AddressingMode::ZeroPage),
    0xb5u8 => OpCode::new(0xb5, "LDA", 1, 4, AddressingMode::ZeroPage_X),
    0xadu8 => OpCode::new(0xad, "LDA", 2, 4, AddressingMode::Absolute),
    0xbdu8 => OpCode::new(0xbd, "LDA", 2, 4/*+1 if page crossed*/, AddressingMode::Absolute_X),
    0xb9u8 => OpCode::new(0xb9, "LDA", 2, 4/*+1 if page crossed*/, AddressingMode::Absolute_Y),
    0xa1u8 => OpCode::new(0xa1, "LDA", 1, 6, AddressingMode::Indirect_X),
    0xb1u8 => OpCode::new(0xb1, "LDA", 1, 5/*+1 if page crossed*/, AddressingMode::Indirect_Y),

    0xa2u8 => OpCode::new(0xa2, "LDX", 1, 2, AddressingMode::Immediate),
    0xa6u8 => OpCode::new(0xa6, "LDX", 1, 3, AddressingMode::ZeroPage),
    0xb6u8 => OpCode::new(0xb6, "LDX", 1, 4, AddressingMode::ZeroPage_Y),
    0xaeu8 => OpCode::new(0xae, "LDX", 2, 4, AddressingMode::Absolute),
    0xbeu8 => OpCode::new(0xbe, "LDX", 2, 4/*+1 if page crossed*/, AddressingMode::Absolute_Y),

    0xa0u8 => OpCode::new(0xa0, "LDY", 1, 2, AddressingMode::Immediate),
    0xa4u8 => OpCode::new(0xa4, "LDY", 1, 3, AddressingMode::ZeroPage),
    0xb4u8 => OpCode::new(0xb4, "LDY", 1, 4, AddressingMode::ZeroPage_X),
    0xacu8 => OpCode::new(0xac, "LDY", 2, 4, AddressingMode::Absolute),
    0xbcu8 => OpCode::new(0xbc, "LDY", 2, 4/*+1 if page crossed*/, AddressingMode::Absolute_X),


    0x85u8 => OpCode::new(0x85, "STA", 1, 3, AddressingMode::ZeroPage),
    0x95u8 => OpCode::new(0x95, "STA", 1, 4, AddressingMode::ZeroPage_X),
    0x8du8 => OpCode::new(0x8d, "STA", 2, 4, AddressingMode::Absolute),
    0x9du8 => OpCode::new(0x9d, "STA", 2, 5, AddressingMode::Absolute_X),
    0x99u8 => OpCode::new(0x99, "STA", 2, 5, AddressingMode::Absolute_Y),
    0x81u8 => OpCode::new(0x81, "STA", 1, 6, AddressingMode::Indirect_X),
    0x91u8 => OpCode::new(0x91, "STA", 1, 6, AddressingMode::Indirect_Y),

    0x86u8 => OpCode::new(0x86, "STX", 1, 3, AddressingMode::ZeroPage),
    0x96u8 => OpCode::new(0x96, "STX", 1, 4, AddressingMode::ZeroPage_Y),
    0x8eu8 => OpCode::new(0x8e, "STX", 2, 4, AddressingMode::Absolute),

    0x84u8 => OpCode::new(0x84, "STY", 1, 3, AddressingMode::ZeroPage),
    0x94u8 => OpCode::new(0x94, "STY", 1, 4, AddressingMode::ZeroPage_X),
    0x8cu8 => OpCode::new(0x8c, "STY", 2, 4, AddressingMode::Absolute),


    /* Flags clear */

    0xD8u8 => OpCode::new(0xD8, "CLD", 0, 2, AddressingMode::Implied),
    0x58u8 => OpCode::new(0x58, "CLI", 0, 2, AddressingMode::Implied),
    0xb8u8 => OpCode::new(0xb8, "CLV", 0, 2, AddressingMode::Implied),
    0x18u8 => OpCode::new(0x18, "CLC", 0, 2, AddressingMode::Implied),
    0x38u8 => OpCode::new(0x38, "SEC", 0, 2, AddressingMode::Implied),
    0x78u8 => OpCode::new(0x78, "SEI", 0, 2, AddressingMode::Implied),
    0xf8u8 => OpCode::new(0xf8, "SED", 0, 2, AddressingMode::Implied),

    0xaau8 => OpCode::new(0xaa, "TAX", 0, 2, AddressingMode::Implied),
    0xa8u8 => OpCode::new(0xa8, "TAY", 0, 2, AddressingMode::Implied),
    0xbau8 => OpCode::new(0xba, "TSX", 0, 2, AddressingMode::Implied),
    0x8au8 => OpCode::new(0x8a, "TXA", 0, 2, AddressingMode::Implied),
    0x9au8 => OpCode::new(0x9a, "TXS", 0, 2, AddressingMode::Implied),
    0x98u8 => OpCode::new(0x98, "TYA", 0, 2, AddressingMode::Implied),

    /* Stack */
    0x48u8 => OpCode::new(0x48, "PHA", 0, 3, AddressingMode::Implied),
    0x68u8 => OpCode::new(0x68, "PLA", 0, 4, AddressingMode::Implied),
    0x08u8 => OpCode::new(0x08, "PHP", 0, 3, AddressingMode::Implied),
    0x28u8 => OpCode::new(0x28, "PLP", 0, 4, AddressingMode::Implied),
};

pub fn get_opcode_details(opcode: &u8) -> Option<&OpCode> {
    OP_CODES_MAP.get(opcode)
}
