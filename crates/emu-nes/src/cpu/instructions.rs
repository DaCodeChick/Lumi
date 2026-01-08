//! 6502 instruction implementations

use super::{Cpu6502, CpuMemory, StatusFlags};
use emu_core::Result;

impl<M: CpuMemory> Cpu6502<M> {
    /// Execute an instruction given its opcode
    /// Returns the number of cycles consumed
    pub(super) fn execute(&mut self, opcode: u8) -> Result<u8> {
        use super::opcodes::*;
        
        let info = get_opcode_info(opcode)
            .ok_or_else(|| emu_core::EmulatorError::InvalidOpcode(opcode))?;
        
        let mut cycles = info.cycles;
        
        match opcode {
            // ADC - Add with Carry
            0x69 => { let v = self.fetch_byte(); self.adc(v); }
            0x65 => { let a = self.addr_zero_page(); let v = self.memory.read(a); self.adc(v); }
            0x75 => { let a = self.addr_zero_page_x(); let v = self.memory.read(a); self.adc(v); }
            0x6D => { let a = self.addr_absolute(); let v = self.memory.read(a); self.adc(v); }
            0x7D => { let (a, p) = self.addr_absolute_x(); let v = self.memory.read(a); self.adc(v); if p { cycles += 1; } }
            0x79 => { let (a, p) = self.addr_absolute_y(); let v = self.memory.read(a); self.adc(v); if p { cycles += 1; } }
            0x61 => { let a = self.addr_indexed_indirect(); let v = self.memory.read(a); self.adc(v); }
            0x71 => { let (a, p) = self.addr_indirect_indexed(); let v = self.memory.read(a); self.adc(v); if p { cycles += 1; } }
            
            // AND - Logical AND
            0x29 => { let v = self.fetch_byte(); self.and(v); }
            0x25 => { let a = self.addr_zero_page(); let v = self.memory.read(a); self.and(v); }
            0x35 => { let a = self.addr_zero_page_x(); let v = self.memory.read(a); self.and(v); }
            0x2D => { let a = self.addr_absolute(); let v = self.memory.read(a); self.and(v); }
            0x3D => { let (a, p) = self.addr_absolute_x(); let v = self.memory.read(a); self.and(v); if p { cycles += 1; } }
            0x39 => { let (a, p) = self.addr_absolute_y(); let v = self.memory.read(a); self.and(v); if p { cycles += 1; } }
            0x21 => { let a = self.addr_indexed_indirect(); let v = self.memory.read(a); self.and(v); }
            0x31 => { let (a, p) = self.addr_indirect_indexed(); let v = self.memory.read(a); self.and(v); if p { cycles += 1; } }
            
            // ASL - Arithmetic Shift Left
            0x0A => { self.asl_acc(); }
            0x06 => { let a = self.addr_zero_page(); self.asl(a); }
            0x16 => { let a = self.addr_zero_page_x(); self.asl(a); }
            0x0E => { let a = self.addr_absolute(); self.asl(a); }
            0x1E => { let (a, _) = self.addr_absolute_x(); self.asl(a); }
            
            // Branch instructions
            0x90 => { let o = self.fetch_byte() as i8; cycles = self.branch(!self.get_flag(StatusFlags::CARRY), o); }
            0xB0 => { let o = self.fetch_byte() as i8; cycles = self.branch(self.get_flag(StatusFlags::CARRY), o); }
            0xF0 => { let o = self.fetch_byte() as i8; cycles = self.branch(self.get_flag(StatusFlags::ZERO), o); }
            0x30 => { let o = self.fetch_byte() as i8; cycles = self.branch(self.get_flag(StatusFlags::NEGATIVE), o); }
            0xD0 => { let o = self.fetch_byte() as i8; cycles = self.branch(!self.get_flag(StatusFlags::ZERO), o); }
            0x10 => { let o = self.fetch_byte() as i8; cycles = self.branch(!self.get_flag(StatusFlags::NEGATIVE), o); }
            0x50 => { let o = self.fetch_byte() as i8; cycles = self.branch(!self.get_flag(StatusFlags::OVERFLOW), o); }
            0x70 => { let o = self.fetch_byte() as i8; cycles = self.branch(self.get_flag(StatusFlags::OVERFLOW), o); }
            
            // BIT - Bit Test
            0x24 => { let a = self.addr_zero_page(); let v = self.memory.read(a); self.bit(v); }
            0x2C => { let a = self.addr_absolute(); let v = self.memory.read(a); self.bit(v); }
            
            // BRK - Force Interrupt
            0x00 => { self.brk(); }
            
            // CLC, CLD, CLI, CLV - Clear flags
            0x18 => { self.set_flag(StatusFlags::CARRY, false); }
            0xD8 => { self.set_flag(StatusFlags::DECIMAL, false); }
            0x58 => { self.set_flag(StatusFlags::INTERRUPT, false); }
            0xB8 => { self.set_flag(StatusFlags::OVERFLOW, false); }
            
            // CMP - Compare Accumulator
            0xC9 => { let v = self.fetch_byte(); self.cmp(v); }
            0xC5 => { let a = self.addr_zero_page(); let v = self.memory.read(a); self.cmp(v); }
            0xD5 => { let a = self.addr_zero_page_x(); let v = self.memory.read(a); self.cmp(v); }
            0xCD => { let a = self.addr_absolute(); let v = self.memory.read(a); self.cmp(v); }
            0xDD => { let (a, p) = self.addr_absolute_x(); let v = self.memory.read(a); self.cmp(v); if p { cycles += 1; } }
            0xD9 => { let (a, p) = self.addr_absolute_y(); let v = self.memory.read(a); self.cmp(v); if p { cycles += 1; } }
            0xC1 => { let a = self.addr_indexed_indirect(); let v = self.memory.read(a); self.cmp(v); }
            0xD1 => { let (a, p) = self.addr_indirect_indexed(); let v = self.memory.read(a); self.cmp(v); if p { cycles += 1; } }
            
            // CPX - Compare X Register
            0xE0 => { let v = self.fetch_byte(); self.cpx(v); }
            0xE4 => { let a = self.addr_zero_page(); let v = self.memory.read(a); self.cpx(v); }
            0xEC => { let a = self.addr_absolute(); let v = self.memory.read(a); self.cpx(v); }
            
            // CPY - Compare Y Register
            0xC0 => { let v = self.fetch_byte(); self.cpy(v); }
            0xC4 => { let a = self.addr_zero_page(); let v = self.memory.read(a); self.cpy(v); }
            0xCC => { let a = self.addr_absolute(); let v = self.memory.read(a); self.cpy(v); }
            
            // DEC - Decrement Memory
            0xC6 => { let a = self.addr_zero_page(); self.dec(a); }
            0xD6 => { let a = self.addr_zero_page_x(); self.dec(a); }
            0xCE => { let a = self.addr_absolute(); self.dec(a); }
            0xDE => { let (a, _) = self.addr_absolute_x(); self.dec(a); }
            
            // DEX, DEY - Decrement X, Y
            0xCA => { self.x = self.x.wrapping_sub(1); self.update_zn(self.x); }
            0x88 => { self.y = self.y.wrapping_sub(1); self.update_zn(self.y); }
            
            // EOR - Exclusive OR
            0x49 => { let v = self.fetch_byte(); self.eor(v); }
            0x45 => { let a = self.addr_zero_page(); let v = self.memory.read(a); self.eor(v); }
            0x55 => { let a = self.addr_zero_page_x(); let v = self.memory.read(a); self.eor(v); }
            0x4D => { let a = self.addr_absolute(); let v = self.memory.read(a); self.eor(v); }
            0x5D => { let (a, p) = self.addr_absolute_x(); let v = self.memory.read(a); self.eor(v); if p { cycles += 1; } }
            0x59 => { let (a, p) = self.addr_absolute_y(); let v = self.memory.read(a); self.eor(v); if p { cycles += 1; } }
            0x41 => { let a = self.addr_indexed_indirect(); let v = self.memory.read(a); self.eor(v); }
            0x51 => { let (a, p) = self.addr_indirect_indexed(); let v = self.memory.read(a); self.eor(v); if p { cycles += 1; } }
            
            // INC - Increment Memory
            0xE6 => { let a = self.addr_zero_page(); self.inc(a); }
            0xF6 => { let a = self.addr_zero_page_x(); self.inc(a); }
            0xEE => { let a = self.addr_absolute(); self.inc(a); }
            0xFE => { let (a, _) = self.addr_absolute_x(); self.inc(a); }
            
            // INX, INY - Increment X, Y
            0xE8 => { self.x = self.x.wrapping_add(1); self.update_zn(self.x); }
            0xC8 => { self.y = self.y.wrapping_add(1); self.update_zn(self.y); }
            
            // JMP - Jump
            0x4C => { self.pc = self.addr_absolute(); }
            0x6C => { self.pc = self.addr_indirect(); }
            
            // JSR - Jump to Subroutine
            0x20 => { let a = self.fetch_word(); self.push_word(self.pc.wrapping_sub(1)); self.pc = a; }
            
            // LDA - Load Accumulator
            0xA9 => { self.a = self.fetch_byte(); self.update_zn(self.a); }
            0xA5 => { let a = self.addr_zero_page(); self.a = self.memory.read(a); self.update_zn(self.a); }
            0xB5 => { let a = self.addr_zero_page_x(); self.a = self.memory.read(a); self.update_zn(self.a); }
            0xAD => { let a = self.addr_absolute(); self.a = self.memory.read(a); self.update_zn(self.a); }
            0xBD => { let (a, p) = self.addr_absolute_x(); self.a = self.memory.read(a); self.update_zn(self.a); if p { cycles += 1; } }
            0xB9 => { let (a, p) = self.addr_absolute_y(); self.a = self.memory.read(a); self.update_zn(self.a); if p { cycles += 1; } }
            0xA1 => { let a = self.addr_indexed_indirect(); self.a = self.memory.read(a); self.update_zn(self.a); }
            0xB1 => { let (a, p) = self.addr_indirect_indexed(); self.a = self.memory.read(a); self.update_zn(self.a); if p { cycles += 1; } }
            
            // LDX - Load X Register
            0xA2 => { self.x = self.fetch_byte(); self.update_zn(self.x); }
            0xA6 => { let a = self.addr_zero_page(); self.x = self.memory.read(a); self.update_zn(self.x); }
            0xB6 => { let a = self.addr_zero_page_y(); self.x = self.memory.read(a); self.update_zn(self.x); }
            0xAE => { let a = self.addr_absolute(); self.x = self.memory.read(a); self.update_zn(self.x); }
            0xBE => { let (a, p) = self.addr_absolute_y(); self.x = self.memory.read(a); self.update_zn(self.x); if p { cycles += 1; } }
            
            // LDY - Load Y Register
            0xA0 => { self.y = self.fetch_byte(); self.update_zn(self.y); }
            0xA4 => { let a = self.addr_zero_page(); self.y = self.memory.read(a); self.update_zn(self.y); }
            0xB4 => { let a = self.addr_zero_page_x(); self.y = self.memory.read(a); self.update_zn(self.y); }
            0xAC => { let a = self.addr_absolute(); self.y = self.memory.read(a); self.update_zn(self.y); }
            0xBC => { let (a, p) = self.addr_absolute_x(); self.y = self.memory.read(a); self.update_zn(self.y); if p { cycles += 1; } }
            
            // LSR - Logical Shift Right
            0x4A => { self.lsr_acc(); }
            0x46 => { let a = self.addr_zero_page(); self.lsr(a); }
            0x56 => { let a = self.addr_zero_page_x(); self.lsr(a); }
            0x4E => { let a = self.addr_absolute(); self.lsr(a); }
            0x5E => { let (a, _) = self.addr_absolute_x(); self.lsr(a); }
            
            // NOP - No Operation
            0xEA => {}
            
            // ORA - Logical OR
            0x09 => { let v = self.fetch_byte(); self.ora(v); }
            0x05 => { let a = self.addr_zero_page(); let v = self.memory.read(a); self.ora(v); }
            0x15 => { let a = self.addr_zero_page_x(); let v = self.memory.read(a); self.ora(v); }
            0x0D => { let a = self.addr_absolute(); let v = self.memory.read(a); self.ora(v); }
            0x1D => { let (a, p) = self.addr_absolute_x(); let v = self.memory.read(a); self.ora(v); if p { cycles += 1; } }
            0x19 => { let (a, p) = self.addr_absolute_y(); let v = self.memory.read(a); self.ora(v); if p { cycles += 1; } }
            0x01 => { let a = self.addr_indexed_indirect(); let v = self.memory.read(a); self.ora(v); }
            0x11 => { let (a, p) = self.addr_indirect_indexed(); let v = self.memory.read(a); self.ora(v); if p { cycles += 1; } }
            
            // PHA, PHP - Push Accumulator, Processor Status
            0x48 => { self.push(self.a); }
            0x08 => { self.push(self.status.bits() | StatusFlags::BREAK.bits() | StatusFlags::UNUSED.bits()); }
            
            // PLA, PLP - Pull Accumulator, Processor Status
            0x68 => { self.a = self.pop(); self.update_zn(self.a); }
            0x28 => { let s = self.pop(); self.status = StatusFlags::from_bits_truncate(s) | StatusFlags::UNUSED; self.status.remove(StatusFlags::BREAK); }
            
            // ROL - Rotate Left
            0x2A => { self.rol_acc(); }
            0x26 => { let a = self.addr_zero_page(); self.rol(a); }
            0x36 => { let a = self.addr_zero_page_x(); self.rol(a); }
            0x2E => { let a = self.addr_absolute(); self.rol(a); }
            0x3E => { let (a, _) = self.addr_absolute_x(); self.rol(a); }
            
            // ROR - Rotate Right
            0x6A => { self.ror_acc(); }
            0x66 => { let a = self.addr_zero_page(); self.ror(a); }
            0x76 => { let a = self.addr_zero_page_x(); self.ror(a); }
            0x6E => { let a = self.addr_absolute(); self.ror(a); }
            0x7E => { let (a, _) = self.addr_absolute_x(); self.ror(a); }
            
            // RTI - Return from Interrupt
            0x40 => { let s = self.pop(); self.status = StatusFlags::from_bits_truncate(s) | StatusFlags::UNUSED; self.status.remove(StatusFlags::BREAK); self.pc = self.pop_word(); }
            
            // RTS - Return from Subroutine
            0x60 => { self.pc = self.pop_word().wrapping_add(1); }
            
            // SBC - Subtract with Carry
            0xE9 => { let v = self.fetch_byte(); self.sbc(v); }
            0xE5 => { let a = self.addr_zero_page(); let v = self.memory.read(a); self.sbc(v); }
            0xF5 => { let a = self.addr_zero_page_x(); let v = self.memory.read(a); self.sbc(v); }
            0xED => { let a = self.addr_absolute(); let v = self.memory.read(a); self.sbc(v); }
            0xFD => { let (a, p) = self.addr_absolute_x(); let v = self.memory.read(a); self.sbc(v); if p { cycles += 1; } }
            0xF9 => { let (a, p) = self.addr_absolute_y(); let v = self.memory.read(a); self.sbc(v); if p { cycles += 1; } }
            0xE1 => { let a = self.addr_indexed_indirect(); let v = self.memory.read(a); self.sbc(v); }
            0xF1 => { let (a, p) = self.addr_indirect_indexed(); let v = self.memory.read(a); self.sbc(v); if p { cycles += 1; } }
            
            // SEC, SED, SEI - Set flags
            0x38 => { self.set_flag(StatusFlags::CARRY, true); }
            0xF8 => { self.set_flag(StatusFlags::DECIMAL, true); }
            0x78 => { self.set_flag(StatusFlags::INTERRUPT, true); }
            
            // STA - Store Accumulator
            0x85 => { let a = self.addr_zero_page(); self.memory.write(a, self.a); }
            0x95 => { let a = self.addr_zero_page_x(); self.memory.write(a, self.a); }
            0x8D => { let a = self.addr_absolute(); self.memory.write(a, self.a); }
            0x9D => { let (a, _) = self.addr_absolute_x(); self.memory.write(a, self.a); }
            0x99 => { let (a, _) = self.addr_absolute_y(); self.memory.write(a, self.a); }
            0x81 => { let a = self.addr_indexed_indirect(); self.memory.write(a, self.a); }
            0x91 => { let (a, _) = self.addr_indirect_indexed(); self.memory.write(a, self.a); }
            
            // STX - Store X Register
            0x86 => { let a = self.addr_zero_page(); self.memory.write(a, self.x); }
            0x96 => { let a = self.addr_zero_page_y(); self.memory.write(a, self.x); }
            0x8E => { let a = self.addr_absolute(); self.memory.write(a, self.x); }
            
            // STY - Store Y Register
            0x84 => { let a = self.addr_zero_page(); self.memory.write(a, self.y); }
            0x94 => { let a = self.addr_zero_page_x(); self.memory.write(a, self.y); }
            0x8C => { let a = self.addr_absolute(); self.memory.write(a, self.y); }
            
            // TAX, TAY, TSX, TXA, TXS, TYA - Transfer instructions
            0xAA => { self.x = self.a; self.update_zn(self.x); }
            0xA8 => { self.y = self.a; self.update_zn(self.y); }
            0xBA => { self.x = self.sp; self.update_zn(self.x); }
            0x8A => { self.a = self.x; self.update_zn(self.a); }
            0x9A => { self.sp = self.x; }
            0x98 => { self.a = self.y; self.update_zn(self.a); }
            
            _ => return Err(emu_core::EmulatorError::InvalidOpcode(opcode)),
        }
        
        self.cycles += cycles as u64;
        Ok(cycles)
    }
    
    // Helper methods for instruction implementations
    
    /// ADC - Add with Carry
    fn adc(&mut self, value: u8) {
        let carry = if self.get_flag(StatusFlags::CARRY) { 1 } else { 0 };
        let sum = self.a as u16 + value as u16 + carry;
        
        self.set_flag(StatusFlags::CARRY, sum > 0xFF);
        
        let result = sum as u8;
        let overflow = (self.a ^ result) & (value ^ result) & 0x80 != 0;
        self.set_flag(StatusFlags::OVERFLOW, overflow);
        
        self.a = result;
        self.update_zn(self.a);
    }
    
    /// SBC - Subtract with Carry
    fn sbc(&mut self, value: u8) {
        self.adc(!value);
    }
    
    /// AND - Logical AND
    fn and(&mut self, value: u8) {
        self.a &= value;
        self.update_zn(self.a);
    }
    
    /// ORA - Logical OR
    fn ora(&mut self, value: u8) {
        self.a |= value;
        self.update_zn(self.a);
    }
    
    /// EOR - Exclusive OR
    fn eor(&mut self, value: u8) {
        self.a ^= value;
        self.update_zn(self.a);
    }
    
    /// BIT - Bit Test
    fn bit(&mut self, value: u8) {
        self.set_flag(StatusFlags::ZERO, self.a & value == 0);
        self.set_flag(StatusFlags::OVERFLOW, value & 0x40 != 0);
        self.set_flag(StatusFlags::NEGATIVE, value & 0x80 != 0);
    }
    
    /// Compare helper
    fn compare(&mut self, register: u8, value: u8) {
        let result = register.wrapping_sub(value);
        self.set_flag(StatusFlags::CARRY, register >= value);
        self.update_zn(result);
    }
    
    /// CMP - Compare Accumulator
    fn cmp(&mut self, value: u8) {
        self.compare(self.a, value);
    }
    
    /// CPX - Compare X
    fn cpx(&mut self, value: u8) {
        self.compare(self.x, value);
    }
    
    /// CPY - Compare Y
    fn cpy(&mut self, value: u8) {
        self.compare(self.y, value);
    }
    
    /// INC - Increment Memory
    fn inc(&mut self, addr: u16) {
        let value = self.memory.read(addr).wrapping_add(1);
        self.memory.write(addr, value);
        self.update_zn(value);
    }
    
    /// DEC - Decrement Memory
    fn dec(&mut self, addr: u16) {
        let value = self.memory.read(addr).wrapping_sub(1);
        self.memory.write(addr, value);
        self.update_zn(value);
    }
    
    /// ASL - Arithmetic Shift Left (Accumulator)
    fn asl_acc(&mut self) {
        self.set_flag(StatusFlags::CARRY, self.a & 0x80 != 0);
        self.a <<= 1;
        self.update_zn(self.a);
    }
    
    /// ASL - Arithmetic Shift Left (Memory)
    fn asl(&mut self, addr: u16) {
        let mut value = self.memory.read(addr);
        self.set_flag(StatusFlags::CARRY, value & 0x80 != 0);
        value <<= 1;
        self.memory.write(addr, value);
        self.update_zn(value);
    }
    
    /// LSR - Logical Shift Right (Accumulator)
    fn lsr_acc(&mut self) {
        self.set_flag(StatusFlags::CARRY, self.a & 0x01 != 0);
        self.a >>= 1;
        self.update_zn(self.a);
    }
    
    /// LSR - Logical Shift Right (Memory)
    fn lsr(&mut self, addr: u16) {
        let mut value = self.memory.read(addr);
        self.set_flag(StatusFlags::CARRY, value & 0x01 != 0);
        value >>= 1;
        self.memory.write(addr, value);
        self.update_zn(value);
    }
    
    /// ROL - Rotate Left (Accumulator)
    fn rol_acc(&mut self) {
        let carry = if self.get_flag(StatusFlags::CARRY) { 1 } else { 0 };
        self.set_flag(StatusFlags::CARRY, self.a & 0x80 != 0);
        self.a = (self.a << 1) | carry;
        self.update_zn(self.a);
    }
    
    /// ROL - Rotate Left (Memory)
    fn rol(&mut self, addr: u16) {
        let mut value = self.memory.read(addr);
        let carry = if self.get_flag(StatusFlags::CARRY) { 1 } else { 0 };
        self.set_flag(StatusFlags::CARRY, value & 0x80 != 0);
        value = (value << 1) | carry;
        self.memory.write(addr, value);
        self.update_zn(value);
    }
    
    /// ROR - Rotate Right (Accumulator)
    fn ror_acc(&mut self) {
        let carry = if self.get_flag(StatusFlags::CARRY) { 0x80 } else { 0 };
        self.set_flag(StatusFlags::CARRY, self.a & 0x01 != 0);
        self.a = (self.a >> 1) | carry;
        self.update_zn(self.a);
    }
    
    /// ROR - Rotate Right (Memory)
    fn ror(&mut self, addr: u16) {
        let mut value = self.memory.read(addr);
        let carry = if self.get_flag(StatusFlags::CARRY) { 0x80 } else { 0 };
        self.set_flag(StatusFlags::CARRY, value & 0x01 != 0);
        value = (value >> 1) | carry;
        self.memory.write(addr, value);
        self.update_zn(value);
    }
    
    /// Branch helper - returns cycle count
    fn branch(&mut self, condition: bool, offset: i8) -> u8 {
        if !condition {
            return 2; // Branch not taken
        }
        
        let old_pc = self.pc;
        self.pc = self.pc.wrapping_add(offset as i16 as u16);
        
        // +1 cycle if branch taken, +1 more if page boundary crossed
        let page_crossed = (old_pc & 0xFF00) != (self.pc & 0xFF00);
        if page_crossed { 4 } else { 3 }
    }
    
    /// BRK - Force Interrupt
    fn brk(&mut self) {
        self.pc = self.pc.wrapping_add(1);
        self.push_word(self.pc);
        self.push(self.status.bits() | StatusFlags::BREAK.bits() | StatusFlags::UNUSED.bits());
        self.set_flag(StatusFlags::INTERRUPT, true);
        self.pc = self.memory.read_word(0xFFFE);
    }
}
