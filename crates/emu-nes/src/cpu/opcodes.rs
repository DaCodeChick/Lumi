//! 6502 opcode definitions and addressing modes

/// Addressing modes for 6502
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressingMode {
    Implied,
    Accumulator,
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Relative,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Indirect,
    IndexedIndirect,  // (Indirect,X)
    IndirectIndexed,  // (Indirect),Y
}

/// Opcode information
pub struct OpcodeInfo {
    pub mnemonic: &'static str,
    pub mode: AddressingMode,
    pub cycles: u8,
    pub page_cross_cycle: bool,  // Add 1 cycle if page boundary crossed
}

// Opcode lookup table will be implemented here
