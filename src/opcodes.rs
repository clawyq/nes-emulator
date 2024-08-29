use crate::cpu::AddressingMode;
use phf::phf_map;

pub struct OpCode {
  pub code: u8,
  pub mnemonic: &'static str,
  pub additional_bytes: u8,
  pub mode: AddressingMode
}

impl OpCode {
  pub const fn new(code: u8, mnemonic: &'static str, additional_bytes: u8, mode: AddressingMode) -> Self {
    OpCode { code, mnemonic, additional_bytes, mode }
  }
}

static OP_CODES_MAP: phf::Map<u8, OpCode> = phf_map! {
  0x00u8 => OpCode::new(0x00, "BRK", 0, AddressingMode::Implied),
  0x85u8 => OpCode::new(0x85, "STA", 1, AddressingMode::ZeroPage),
  0x95u8 => OpCode::new(0x95, "STA", 1, AddressingMode::ZeroPage_X),
  0xA2u8 => OpCode::new(0xa2, "LDX", 1, AddressingMode::Immediate),
  0xA5u8 => OpCode::new(0xa5, "LDA", 1, AddressingMode::ZeroPage),
  0xA9u8 => OpCode::new(0xa9, "LDA", 1, AddressingMode::Immediate),
  0xADu8 => OpCode::new(0xad, "LDA", 2, AddressingMode::Absolute),
  0xAAu8 => OpCode::new(0xaa, "TAX", 0, AddressingMode::Implied),
  0xE8u8 => OpCode::new(0xe8, "INX", 0, AddressingMode::Implied),
};

pub fn get_opcode_details(opcode: &u8) -> Option<&OpCode> {
  OP_CODES_MAP.get(opcode)
}