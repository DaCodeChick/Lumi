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

/// Get opcode information for a given opcode byte
/// Returns None for illegal/unofficial opcodes
pub fn get_opcode_info(opcode: u8) -> Option<OpcodeInfo> {
    Some(match opcode {
        // ADC - Add with Carry
        0x69 => OpcodeInfo { mnemonic: "ADC", mode: AddressingMode::Immediate, cycles: 2, page_cross_cycle: false },
        0x65 => OpcodeInfo { mnemonic: "ADC", mode: AddressingMode::ZeroPage, cycles: 3, page_cross_cycle: false },
        0x75 => OpcodeInfo { mnemonic: "ADC", mode: AddressingMode::ZeroPageX, cycles: 4, page_cross_cycle: false },
        0x6D => OpcodeInfo { mnemonic: "ADC", mode: AddressingMode::Absolute, cycles: 4, page_cross_cycle: false },
        0x7D => OpcodeInfo { mnemonic: "ADC", mode: AddressingMode::AbsoluteX, cycles: 4, page_cross_cycle: true },
        0x79 => OpcodeInfo { mnemonic: "ADC", mode: AddressingMode::AbsoluteY, cycles: 4, page_cross_cycle: true },
        0x61 => OpcodeInfo { mnemonic: "ADC", mode: AddressingMode::IndexedIndirect, cycles: 6, page_cross_cycle: false },
        0x71 => OpcodeInfo { mnemonic: "ADC", mode: AddressingMode::IndirectIndexed, cycles: 5, page_cross_cycle: true },
        
        // AND - Logical AND
        0x29 => OpcodeInfo { mnemonic: "AND", mode: AddressingMode::Immediate, cycles: 2, page_cross_cycle: false },
        0x25 => OpcodeInfo { mnemonic: "AND", mode: AddressingMode::ZeroPage, cycles: 3, page_cross_cycle: false },
        0x35 => OpcodeInfo { mnemonic: "AND", mode: AddressingMode::ZeroPageX, cycles: 4, page_cross_cycle: false },
        0x2D => OpcodeInfo { mnemonic: "AND", mode: AddressingMode::Absolute, cycles: 4, page_cross_cycle: false },
        0x3D => OpcodeInfo { mnemonic: "AND", mode: AddressingMode::AbsoluteX, cycles: 4, page_cross_cycle: true },
        0x39 => OpcodeInfo { mnemonic: "AND", mode: AddressingMode::AbsoluteY, cycles: 4, page_cross_cycle: true },
        0x21 => OpcodeInfo { mnemonic: "AND", mode: AddressingMode::IndexedIndirect, cycles: 6, page_cross_cycle: false },
        0x31 => OpcodeInfo { mnemonic: "AND", mode: AddressingMode::IndirectIndexed, cycles: 5, page_cross_cycle: true },
        
        // ASL - Arithmetic Shift Left
        0x0A => OpcodeInfo { mnemonic: "ASL", mode: AddressingMode::Accumulator, cycles: 2, page_cross_cycle: false },
        0x06 => OpcodeInfo { mnemonic: "ASL", mode: AddressingMode::ZeroPage, cycles: 5, page_cross_cycle: false },
        0x16 => OpcodeInfo { mnemonic: "ASL", mode: AddressingMode::ZeroPageX, cycles: 6, page_cross_cycle: false },
        0x0E => OpcodeInfo { mnemonic: "ASL", mode: AddressingMode::Absolute, cycles: 6, page_cross_cycle: false },
        0x1E => OpcodeInfo { mnemonic: "ASL", mode: AddressingMode::AbsoluteX, cycles: 7, page_cross_cycle: false },
        
        // BCC - Branch if Carry Clear
        0x90 => OpcodeInfo { mnemonic: "BCC", mode: AddressingMode::Relative, cycles: 2, page_cross_cycle: false },
        
        // BCS - Branch if Carry Set
        0xB0 => OpcodeInfo { mnemonic: "BCS", mode: AddressingMode::Relative, cycles: 2, page_cross_cycle: false },
        
        // BEQ - Branch if Equal
        0xF0 => OpcodeInfo { mnemonic: "BEQ", mode: AddressingMode::Relative, cycles: 2, page_cross_cycle: false },
        
        // BIT - Bit Test
        0x24 => OpcodeInfo { mnemonic: "BIT", mode: AddressingMode::ZeroPage, cycles: 3, page_cross_cycle: false },
        0x2C => OpcodeInfo { mnemonic: "BIT", mode: AddressingMode::Absolute, cycles: 4, page_cross_cycle: false },
        
        // BMI - Branch if Minus
        0x30 => OpcodeInfo { mnemonic: "BMI", mode: AddressingMode::Relative, cycles: 2, page_cross_cycle: false },
        
        // BNE - Branch if Not Equal
        0xD0 => OpcodeInfo { mnemonic: "BNE", mode: AddressingMode::Relative, cycles: 2, page_cross_cycle: false },
        
        // BPL - Branch if Positive
        0x10 => OpcodeInfo { mnemonic: "BPL", mode: AddressingMode::Relative, cycles: 2, page_cross_cycle: false },
        
        // BRK - Force Interrupt
        0x00 => OpcodeInfo { mnemonic: "BRK", mode: AddressingMode::Implied, cycles: 7, page_cross_cycle: false },
        
        // BVC - Branch if Overflow Clear
        0x50 => OpcodeInfo { mnemonic: "BVC", mode: AddressingMode::Relative, cycles: 2, page_cross_cycle: false },
        
        // BVS - Branch if Overflow Set
        0x70 => OpcodeInfo { mnemonic: "BVS", mode: AddressingMode::Relative, cycles: 2, page_cross_cycle: false },
        
        // CLC - Clear Carry Flag
        0x18 => OpcodeInfo { mnemonic: "CLC", mode: AddressingMode::Implied, cycles: 2, page_cross_cycle: false },
        
        // CLD - Clear Decimal Mode
        0xD8 => OpcodeInfo { mnemonic: "CLD", mode: AddressingMode::Implied, cycles: 2, page_cross_cycle: false },
        
        // CLI - Clear Interrupt Disable
        0x58 => OpcodeInfo { mnemonic: "CLI", mode: AddressingMode::Implied, cycles: 2, page_cross_cycle: false },
        
        // CLV - Clear Overflow Flag
        0xB8 => OpcodeInfo { mnemonic: "CLV", mode: AddressingMode::Implied, cycles: 2, page_cross_cycle: false },
        
        // CMP - Compare
        0xC9 => OpcodeInfo { mnemonic: "CMP", mode: AddressingMode::Immediate, cycles: 2, page_cross_cycle: false },
        0xC5 => OpcodeInfo { mnemonic: "CMP", mode: AddressingMode::ZeroPage, cycles: 3, page_cross_cycle: false },
        0xD5 => OpcodeInfo { mnemonic: "CMP", mode: AddressingMode::ZeroPageX, cycles: 4, page_cross_cycle: false },
        0xCD => OpcodeInfo { mnemonic: "CMP", mode: AddressingMode::Absolute, cycles: 4, page_cross_cycle: false },
        0xDD => OpcodeInfo { mnemonic: "CMP", mode: AddressingMode::AbsoluteX, cycles: 4, page_cross_cycle: true },
        0xD9 => OpcodeInfo { mnemonic: "CMP", mode: AddressingMode::AbsoluteY, cycles: 4, page_cross_cycle: true },
        0xC1 => OpcodeInfo { mnemonic: "CMP", mode: AddressingMode::IndexedIndirect, cycles: 6, page_cross_cycle: false },
        0xD1 => OpcodeInfo { mnemonic: "CMP", mode: AddressingMode::IndirectIndexed, cycles: 5, page_cross_cycle: true },
        
        // CPX - Compare X Register
        0xE0 => OpcodeInfo { mnemonic: "CPX", mode: AddressingMode::Immediate, cycles: 2, page_cross_cycle: false },
        0xE4 => OpcodeInfo { mnemonic: "CPX", mode: AddressingMode::ZeroPage, cycles: 3, page_cross_cycle: false },
        0xEC => OpcodeInfo { mnemonic: "CPX", mode: AddressingMode::Absolute, cycles: 4, page_cross_cycle: false },
        
        // CPY - Compare Y Register
        0xC0 => OpcodeInfo { mnemonic: "CPY", mode: AddressingMode::Immediate, cycles: 2, page_cross_cycle: false },
        0xC4 => OpcodeInfo { mnemonic: "CPY", mode: AddressingMode::ZeroPage, cycles: 3, page_cross_cycle: false },
        0xCC => OpcodeInfo { mnemonic: "CPY", mode: AddressingMode::Absolute, cycles: 4, page_cross_cycle: false },
        
        // DEC - Decrement Memory
        0xC6 => OpcodeInfo { mnemonic: "DEC", mode: AddressingMode::ZeroPage, cycles: 5, page_cross_cycle: false },
        0xD6 => OpcodeInfo { mnemonic: "DEC", mode: AddressingMode::ZeroPageX, cycles: 6, page_cross_cycle: false },
        0xCE => OpcodeInfo { mnemonic: "DEC", mode: AddressingMode::Absolute, cycles: 6, page_cross_cycle: false },
        0xDE => OpcodeInfo { mnemonic: "DEC", mode: AddressingMode::AbsoluteX, cycles: 7, page_cross_cycle: false },
        
        // DEX - Decrement X Register
        0xCA => OpcodeInfo { mnemonic: "DEX", mode: AddressingMode::Implied, cycles: 2, page_cross_cycle: false },
        
        // DEY - Decrement Y Register
        0x88 => OpcodeInfo { mnemonic: "DEY", mode: AddressingMode::Implied, cycles: 2, page_cross_cycle: false },
        
        // EOR - Exclusive OR
        0x49 => OpcodeInfo { mnemonic: "EOR", mode: AddressingMode::Immediate, cycles: 2, page_cross_cycle: false },
        0x45 => OpcodeInfo { mnemonic: "EOR", mode: AddressingMode::ZeroPage, cycles: 3, page_cross_cycle: false },
        0x55 => OpcodeInfo { mnemonic: "EOR", mode: AddressingMode::ZeroPageX, cycles: 4, page_cross_cycle: false },
        0x4D => OpcodeInfo { mnemonic: "EOR", mode: AddressingMode::Absolute, cycles: 4, page_cross_cycle: false },
        0x5D => OpcodeInfo { mnemonic: "EOR", mode: AddressingMode::AbsoluteX, cycles: 4, page_cross_cycle: true },
        0x59 => OpcodeInfo { mnemonic: "EOR", mode: AddressingMode::AbsoluteY, cycles: 4, page_cross_cycle: true },
        0x41 => OpcodeInfo { mnemonic: "EOR", mode: AddressingMode::IndexedIndirect, cycles: 6, page_cross_cycle: false },
        0x51 => OpcodeInfo { mnemonic: "EOR", mode: AddressingMode::IndirectIndexed, cycles: 5, page_cross_cycle: true },
        
        // INC - Increment Memory
        0xE6 => OpcodeInfo { mnemonic: "INC", mode: AddressingMode::ZeroPage, cycles: 5, page_cross_cycle: false },
        0xF6 => OpcodeInfo { mnemonic: "INC", mode: AddressingMode::ZeroPageX, cycles: 6, page_cross_cycle: false },
        0xEE => OpcodeInfo { mnemonic: "INC", mode: AddressingMode::Absolute, cycles: 6, page_cross_cycle: false },
        0xFE => OpcodeInfo { mnemonic: "INC", mode: AddressingMode::AbsoluteX, cycles: 7, page_cross_cycle: false },
        
        // INX - Increment X Register
        0xE8 => OpcodeInfo { mnemonic: "INX", mode: AddressingMode::Implied, cycles: 2, page_cross_cycle: false },
        
        // INY - Increment Y Register
        0xC8 => OpcodeInfo { mnemonic: "INY", mode: AddressingMode::Implied, cycles: 2, page_cross_cycle: false },
        
        // JMP - Jump
        0x4C => OpcodeInfo { mnemonic: "JMP", mode: AddressingMode::Absolute, cycles: 3, page_cross_cycle: false },
        0x6C => OpcodeInfo { mnemonic: "JMP", mode: AddressingMode::Indirect, cycles: 5, page_cross_cycle: false },
        
        // JSR - Jump to Subroutine
        0x20 => OpcodeInfo { mnemonic: "JSR", mode: AddressingMode::Absolute, cycles: 6, page_cross_cycle: false },
        
        // LDA - Load Accumulator
        0xA9 => OpcodeInfo { mnemonic: "LDA", mode: AddressingMode::Immediate, cycles: 2, page_cross_cycle: false },
        0xA5 => OpcodeInfo { mnemonic: "LDA", mode: AddressingMode::ZeroPage, cycles: 3, page_cross_cycle: false },
        0xB5 => OpcodeInfo { mnemonic: "LDA", mode: AddressingMode::ZeroPageX, cycles: 4, page_cross_cycle: false },
        0xAD => OpcodeInfo { mnemonic: "LDA", mode: AddressingMode::Absolute, cycles: 4, page_cross_cycle: false },
        0xBD => OpcodeInfo { mnemonic: "LDA", mode: AddressingMode::AbsoluteX, cycles: 4, page_cross_cycle: true },
        0xB9 => OpcodeInfo { mnemonic: "LDA", mode: AddressingMode::AbsoluteY, cycles: 4, page_cross_cycle: true },
        0xA1 => OpcodeInfo { mnemonic: "LDA", mode: AddressingMode::IndexedIndirect, cycles: 6, page_cross_cycle: false },
        0xB1 => OpcodeInfo { mnemonic: "LDA", mode: AddressingMode::IndirectIndexed, cycles: 5, page_cross_cycle: true },
        
        // LDX - Load X Register
        0xA2 => OpcodeInfo { mnemonic: "LDX", mode: AddressingMode::Immediate, cycles: 2, page_cross_cycle: false },
        0xA6 => OpcodeInfo { mnemonic: "LDX", mode: AddressingMode::ZeroPage, cycles: 3, page_cross_cycle: false },
        0xB6 => OpcodeInfo { mnemonic: "LDX", mode: AddressingMode::ZeroPageY, cycles: 4, page_cross_cycle: false },
        0xAE => OpcodeInfo { mnemonic: "LDX", mode: AddressingMode::Absolute, cycles: 4, page_cross_cycle: false },
        0xBE => OpcodeInfo { mnemonic: "LDX", mode: AddressingMode::AbsoluteY, cycles: 4, page_cross_cycle: true },
        
        // LDY - Load Y Register
        0xA0 => OpcodeInfo { mnemonic: "LDY", mode: AddressingMode::Immediate, cycles: 2, page_cross_cycle: false },
        0xA4 => OpcodeInfo { mnemonic: "LDY", mode: AddressingMode::ZeroPage, cycles: 3, page_cross_cycle: false },
        0xB4 => OpcodeInfo { mnemonic: "LDY", mode: AddressingMode::ZeroPageX, cycles: 4, page_cross_cycle: false },
        0xAC => OpcodeInfo { mnemonic: "LDY", mode: AddressingMode::Absolute, cycles: 4, page_cross_cycle: false },
        0xBC => OpcodeInfo { mnemonic: "LDY", mode: AddressingMode::AbsoluteX, cycles: 4, page_cross_cycle: true },
        
        // LSR - Logical Shift Right
        0x4A => OpcodeInfo { mnemonic: "LSR", mode: AddressingMode::Accumulator, cycles: 2, page_cross_cycle: false },
        0x46 => OpcodeInfo { mnemonic: "LSR", mode: AddressingMode::ZeroPage, cycles: 5, page_cross_cycle: false },
        0x56 => OpcodeInfo { mnemonic: "LSR", mode: AddressingMode::ZeroPageX, cycles: 6, page_cross_cycle: false },
        0x4E => OpcodeInfo { mnemonic: "LSR", mode: AddressingMode::Absolute, cycles: 6, page_cross_cycle: false },
        0x5E => OpcodeInfo { mnemonic: "LSR", mode: AddressingMode::AbsoluteX, cycles: 7, page_cross_cycle: false },
        
        // NOP - No Operation
        0xEA => OpcodeInfo { mnemonic: "NOP", mode: AddressingMode::Implied, cycles: 2, page_cross_cycle: false },
        
        // ORA - Logical Inclusive OR
        0x09 => OpcodeInfo { mnemonic: "ORA", mode: AddressingMode::Immediate, cycles: 2, page_cross_cycle: false },
        0x05 => OpcodeInfo { mnemonic: "ORA", mode: AddressingMode::ZeroPage, cycles: 3, page_cross_cycle: false },
        0x15 => OpcodeInfo { mnemonic: "ORA", mode: AddressingMode::ZeroPageX, cycles: 4, page_cross_cycle: false },
        0x0D => OpcodeInfo { mnemonic: "ORA", mode: AddressingMode::Absolute, cycles: 4, page_cross_cycle: false },
        0x1D => OpcodeInfo { mnemonic: "ORA", mode: AddressingMode::AbsoluteX, cycles: 4, page_cross_cycle: true },
        0x19 => OpcodeInfo { mnemonic: "ORA", mode: AddressingMode::AbsoluteY, cycles: 4, page_cross_cycle: true },
        0x01 => OpcodeInfo { mnemonic: "ORA", mode: AddressingMode::IndexedIndirect, cycles: 6, page_cross_cycle: false },
        0x11 => OpcodeInfo { mnemonic: "ORA", mode: AddressingMode::IndirectIndexed, cycles: 5, page_cross_cycle: true },
        
        // PHA - Push Accumulator
        0x48 => OpcodeInfo { mnemonic: "PHA", mode: AddressingMode::Implied, cycles: 3, page_cross_cycle: false },
        
        // PHP - Push Processor Status
        0x08 => OpcodeInfo { mnemonic: "PHP", mode: AddressingMode::Implied, cycles: 3, page_cross_cycle: false },
        
        // PLA - Pull Accumulator
        0x68 => OpcodeInfo { mnemonic: "PLA", mode: AddressingMode::Implied, cycles: 4, page_cross_cycle: false },
        
        // PLP - Pull Processor Status
        0x28 => OpcodeInfo { mnemonic: "PLP", mode: AddressingMode::Implied, cycles: 4, page_cross_cycle: false },
        
        // ROL - Rotate Left
        0x2A => OpcodeInfo { mnemonic: "ROL", mode: AddressingMode::Accumulator, cycles: 2, page_cross_cycle: false },
        0x26 => OpcodeInfo { mnemonic: "ROL", mode: AddressingMode::ZeroPage, cycles: 5, page_cross_cycle: false },
        0x36 => OpcodeInfo { mnemonic: "ROL", mode: AddressingMode::ZeroPageX, cycles: 6, page_cross_cycle: false },
        0x2E => OpcodeInfo { mnemonic: "ROL", mode: AddressingMode::Absolute, cycles: 6, page_cross_cycle: false },
        0x3E => OpcodeInfo { mnemonic: "ROL", mode: AddressingMode::AbsoluteX, cycles: 7, page_cross_cycle: false },
        
        // ROR - Rotate Right
        0x6A => OpcodeInfo { mnemonic: "ROR", mode: AddressingMode::Accumulator, cycles: 2, page_cross_cycle: false },
        0x66 => OpcodeInfo { mnemonic: "ROR", mode: AddressingMode::ZeroPage, cycles: 5, page_cross_cycle: false },
        0x76 => OpcodeInfo { mnemonic: "ROR", mode: AddressingMode::ZeroPageX, cycles: 6, page_cross_cycle: false },
        0x6E => OpcodeInfo { mnemonic: "ROR", mode: AddressingMode::Absolute, cycles: 6, page_cross_cycle: false },
        0x7E => OpcodeInfo { mnemonic: "ROR", mode: AddressingMode::AbsoluteX, cycles: 7, page_cross_cycle: false },
        
        // RTI - Return from Interrupt
        0x40 => OpcodeInfo { mnemonic: "RTI", mode: AddressingMode::Implied, cycles: 6, page_cross_cycle: false },
        
        // RTS - Return from Subroutine
        0x60 => OpcodeInfo { mnemonic: "RTS", mode: AddressingMode::Implied, cycles: 6, page_cross_cycle: false },
        
        // SBC - Subtract with Carry
        0xE9 => OpcodeInfo { mnemonic: "SBC", mode: AddressingMode::Immediate, cycles: 2, page_cross_cycle: false },
        0xE5 => OpcodeInfo { mnemonic: "SBC", mode: AddressingMode::ZeroPage, cycles: 3, page_cross_cycle: false },
        0xF5 => OpcodeInfo { mnemonic: "SBC", mode: AddressingMode::ZeroPageX, cycles: 4, page_cross_cycle: false },
        0xED => OpcodeInfo { mnemonic: "SBC", mode: AddressingMode::Absolute, cycles: 4, page_cross_cycle: false },
        0xFD => OpcodeInfo { mnemonic: "SBC", mode: AddressingMode::AbsoluteX, cycles: 4, page_cross_cycle: true },
        0xF9 => OpcodeInfo { mnemonic: "SBC", mode: AddressingMode::AbsoluteY, cycles: 4, page_cross_cycle: true },
        0xE1 => OpcodeInfo { mnemonic: "SBC", mode: AddressingMode::IndexedIndirect, cycles: 6, page_cross_cycle: false },
        0xF1 => OpcodeInfo { mnemonic: "SBC", mode: AddressingMode::IndirectIndexed, cycles: 5, page_cross_cycle: true },
        
        // SEC - Set Carry Flag
        0x38 => OpcodeInfo { mnemonic: "SEC", mode: AddressingMode::Implied, cycles: 2, page_cross_cycle: false },
        
        // SED - Set Decimal Flag
        0xF8 => OpcodeInfo { mnemonic: "SED", mode: AddressingMode::Implied, cycles: 2, page_cross_cycle: false },
        
        // SEI - Set Interrupt Disable
        0x78 => OpcodeInfo { mnemonic: "SEI", mode: AddressingMode::Implied, cycles: 2, page_cross_cycle: false },
        
        // STA - Store Accumulator
        0x85 => OpcodeInfo { mnemonic: "STA", mode: AddressingMode::ZeroPage, cycles: 3, page_cross_cycle: false },
        0x95 => OpcodeInfo { mnemonic: "STA", mode: AddressingMode::ZeroPageX, cycles: 4, page_cross_cycle: false },
        0x8D => OpcodeInfo { mnemonic: "STA", mode: AddressingMode::Absolute, cycles: 4, page_cross_cycle: false },
        0x9D => OpcodeInfo { mnemonic: "STA", mode: AddressingMode::AbsoluteX, cycles: 5, page_cross_cycle: false },
        0x99 => OpcodeInfo { mnemonic: "STA", mode: AddressingMode::AbsoluteY, cycles: 5, page_cross_cycle: false },
        0x81 => OpcodeInfo { mnemonic: "STA", mode: AddressingMode::IndexedIndirect, cycles: 6, page_cross_cycle: false },
        0x91 => OpcodeInfo { mnemonic: "STA", mode: AddressingMode::IndirectIndexed, cycles: 6, page_cross_cycle: false },
        
        // STX - Store X Register
        0x86 => OpcodeInfo { mnemonic: "STX", mode: AddressingMode::ZeroPage, cycles: 3, page_cross_cycle: false },
        0x96 => OpcodeInfo { mnemonic: "STX", mode: AddressingMode::ZeroPageY, cycles: 4, page_cross_cycle: false },
        0x8E => OpcodeInfo { mnemonic: "STX", mode: AddressingMode::Absolute, cycles: 4, page_cross_cycle: false },
        
        // STY - Store Y Register
        0x84 => OpcodeInfo { mnemonic: "STY", mode: AddressingMode::ZeroPage, cycles: 3, page_cross_cycle: false },
        0x94 => OpcodeInfo { mnemonic: "STY", mode: AddressingMode::ZeroPageX, cycles: 4, page_cross_cycle: false },
        0x8C => OpcodeInfo { mnemonic: "STY", mode: AddressingMode::Absolute, cycles: 4, page_cross_cycle: false },
        
        // TAX - Transfer Accumulator to X
        0xAA => OpcodeInfo { mnemonic: "TAX", mode: AddressingMode::Implied, cycles: 2, page_cross_cycle: false },
        
        // TAY - Transfer Accumulator to Y
        0xA8 => OpcodeInfo { mnemonic: "TAY", mode: AddressingMode::Implied, cycles: 2, page_cross_cycle: false },
        
        // TSX - Transfer Stack Pointer to X
        0xBA => OpcodeInfo { mnemonic: "TSX", mode: AddressingMode::Implied, cycles: 2, page_cross_cycle: false },
        
        // TXA - Transfer X to Accumulator
        0x8A => OpcodeInfo { mnemonic: "TXA", mode: AddressingMode::Implied, cycles: 2, page_cross_cycle: false },
        
        // TXS - Transfer X to Stack Pointer
        0x9A => OpcodeInfo { mnemonic: "TXS", mode: AddressingMode::Implied, cycles: 2, page_cross_cycle: false },
        
        // TYA - Transfer Y to Accumulator
        0x98 => OpcodeInfo { mnemonic: "TYA", mode: AddressingMode::Implied, cycles: 2, page_cross_cycle: false },
        
        // Illegal/unofficial opcodes - return None
        _ => return None,
    })
}
