//! 6502 CPU implementation for NES

mod instructions;
mod opcodes;

use bitflags::bitflags;
use emu_core::{Cpu as CpuTrait, EmulatorError, Result};

bitflags! {
    /// CPU status flags
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct StatusFlags: u8 {
        const CARRY     = 0b0000_0001;  // C
        const ZERO      = 0b0000_0010;  // Z
        const INTERRUPT = 0b0000_0100;  // I (interrupt disable)
        const DECIMAL   = 0b0000_1000;  // D (decimal mode - not used in NES)
        const BREAK     = 0b0001_0000;  // B (break command)
        const UNUSED    = 0b0010_0000;  // Always set
        const OVERFLOW  = 0b0100_0000;  // V
        const NEGATIVE  = 0b1000_0000;  // N
    }
}

/// Memory interface for the CPU
///
/// The CPU needs to read/write memory without caring about the
/// actual implementation (RAM, ROM, memory-mapped IO, etc.)
pub trait CpuMemory {
    fn read(&mut self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, value: u8);

    fn read_word(&mut self, addr: u16) -> u16 {
        let lo = self.read(addr) as u16;
        let hi = self.read(addr.wrapping_add(1)) as u16;
        (hi << 8) | lo
    }
}

/// 6502 CPU implementation
pub struct Cpu6502<M: CpuMemory> {
    /// Accumulator
    pub a: u8,
    /// X register
    pub x: u8,
    /// Y register
    pub y: u8,
    /// Stack pointer (points into 0x0100-0x01FF)
    pub sp: u8,
    /// Program counter
    pub pc: u16,
    /// Status flags
    pub status: StatusFlags,
    /// Memory interface
    memory: M,
    /// Total cycles executed
    pub cycles: u64,
}

impl<M: CpuMemory> Cpu6502<M> {
    /// Create a new CPU with the given memory interface
    pub fn new(memory: M) -> Self {
        Self {
            a: 0,
            x: 0,
            y: 0,
            sp: 0xFD,
            pc: 0,
            status: StatusFlags::INTERRUPT | StatusFlags::UNUSED,
            memory,
            cycles: 0,
        }
    }

    /// Set a status flag
    #[inline]
    pub fn set_flag(&mut self, flag: StatusFlags, value: bool) {
        if value {
            self.status.insert(flag);
        } else {
            self.status.remove(flag);
        }
    }

    /// Get a status flag
    #[inline]
    pub fn get_flag(&self, flag: StatusFlags) -> bool {
        self.status.contains(flag)
    }

    /// Update zero and negative flags based on value
    #[inline]
    pub fn update_zn(&mut self, value: u8) {
        self.set_flag(StatusFlags::ZERO, value == 0);
        self.set_flag(StatusFlags::NEGATIVE, value & 0x80 != 0);
    }

    /// Push a byte onto the stack
    #[inline]
    fn push(&mut self, value: u8) {
        self.memory.write(0x0100 | self.sp as u16, value);
        self.sp = self.sp.wrapping_sub(1);
    }

    /// Push a 16-bit word onto the stack (high byte first)
    #[inline]
    fn push_word(&mut self, value: u16) {
        self.push((value >> 8) as u8);
        self.push((value & 0xFF) as u8);
    }

    /// Pop a byte from the stack
    #[inline]
    fn pop(&mut self) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        self.memory.read(0x0100 | self.sp as u16)
    }

    /// Pop a 16-bit word from the stack (low byte first)
    #[inline]
    fn pop_word(&mut self) -> u16 {
        let lo = self.pop() as u16;
        let hi = self.pop() as u16;
        (hi << 8) | lo
    }

    /// Read the next byte at PC and increment PC
    #[inline]
    fn fetch_byte(&mut self) -> u8 {
        let byte = self.memory.read(self.pc);
        self.pc = self.pc.wrapping_add(1);
        byte
    }

    /// Read the next word at PC and increment PC by 2
    #[inline]
    fn fetch_word(&mut self) -> u16 {
        let word = self.memory.read_word(self.pc);
        self.pc = self.pc.wrapping_add(2);
        word
    }

    // Addressing mode helpers (used by instructions)

    /// Get address for Zero Page addressing (addr = $00nn)
    #[inline]
    pub(super) fn addr_zero_page(&mut self) -> u16 {
        self.fetch_byte() as u16
    }

    /// Get address for Zero Page,X addressing (addr = $00nn + X)
    #[inline]
    pub(super) fn addr_zero_page_x(&mut self) -> u16 {
        self.fetch_byte().wrapping_add(self.x) as u16
    }

    /// Get address for Zero Page,Y addressing (addr = $00nn + Y)
    #[inline]
    pub(super) fn addr_zero_page_y(&mut self) -> u16 {
        self.fetch_byte().wrapping_add(self.y) as u16
    }

    /// Get address for Absolute addressing (addr = $nnnn)
    #[inline]
    pub(super) fn addr_absolute(&mut self) -> u16 {
        self.fetch_word()
    }

    /// Get address for Absolute,X addressing (addr = $nnnn + X)
    /// Returns (address, page_crossed)
    #[inline]
    pub(super) fn addr_absolute_x(&mut self) -> (u16, bool) {
        let base = self.fetch_word();
        let addr = base.wrapping_add(self.x as u16);
        let page_crossed = (base & 0xFF00) != (addr & 0xFF00);
        (addr, page_crossed)
    }

    /// Get address for Absolute,Y addressing (addr = $nnnn + Y)
    /// Returns (address, page_crossed)
    #[inline]
    pub(super) fn addr_absolute_y(&mut self) -> (u16, bool) {
        let base = self.fetch_word();
        let addr = base.wrapping_add(self.y as u16);
        let page_crossed = (base & 0xFF00) != (addr & 0xFF00);
        (addr, page_crossed)
    }

    /// Get address for Indirect addressing (JMP only)
    /// addr = contents of $nnnn
    #[inline]
    pub(super) fn addr_indirect(&mut self) -> u16 {
        let ptr = self.fetch_word();
        
        // 6502 bug: if ptr is at page boundary (e.g. $xxFF),
        // it wraps within the same page instead of crossing to next page
        if ptr & 0xFF == 0xFF {
            let lo = self.memory.read(ptr) as u16;
            let hi = self.memory.read(ptr & 0xFF00) as u16;
            (hi << 8) | lo
        } else {
            self.memory.read_word(ptr)
        }
    }

    /// Get address for Indexed Indirect addressing (addr = contents of ($nn + X))
    #[inline]
    pub(super) fn addr_indexed_indirect(&mut self) -> u16 {
        let ptr = self.fetch_byte().wrapping_add(self.x);
        let lo = self.memory.read(ptr as u16) as u16;
        let hi = self.memory.read(ptr.wrapping_add(1) as u16) as u16;
        (hi << 8) | lo
    }

    /// Get address for Indirect Indexed addressing (addr = contents of ($nn) + Y)
    /// Returns (address, page_crossed)
    #[inline]
    pub(super) fn addr_indirect_indexed(&mut self) -> (u16, bool) {
        let ptr = self.fetch_byte();
        let lo = self.memory.read(ptr as u16) as u16;
        let hi = self.memory.read(ptr.wrapping_add(1) as u16) as u16;
        let base = (hi << 8) | lo;
        let addr = base.wrapping_add(self.y as u16);
        let page_crossed = (base & 0xFF00) != (addr & 0xFF00);
        (addr, page_crossed)
    }
}

impl<M: CpuMemory> CpuTrait for Cpu6502<M> {
    fn reset(&mut self) {
        self.a = 0;
        self.x = 0;
        self.y = 0;
        self.sp = 0xFD;
        self.status = StatusFlags::INTERRUPT | StatusFlags::UNUSED;
        
        // Read reset vector from 0xFFFC-0xFFFD
        self.pc = self.memory.read_word(0xFFFC);
        
        self.cycles = 0;
    }

    fn step(&mut self) -> Result<u8> {
        // Fetch opcode
        let opcode = self.fetch_byte();
        
        // Execute instruction (to be implemented)
        self.execute(opcode)
    }

    fn pc(&self) -> u16 {
        self.pc
    }

    fn sp(&self) -> u8 {
        self.sp
    }

    fn a(&self) -> u8 {
        self.a
    }

    fn x(&self) -> u8 {
        self.x
    }

    fn y(&self) -> u8 {
        self.y
    }

    fn status(&self) -> u8 {
        self.status.bits()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestMemory {
        ram: Vec<u8>,
    }

    impl TestMemory {
        fn new() -> Self {
            Self { ram: vec![0; 0x10000] }
        }
    }

    impl CpuMemory for TestMemory {
        fn read(&mut self, addr: u16) -> u8 {
            self.ram[addr as usize]
        }

        fn write(&mut self, addr: u16, value: u8) {
            self.ram[addr as usize] = value;
        }
    }

    #[test]
    fn test_cpu_creation() {
        let memory = TestMemory::new();
        let cpu = Cpu6502::new(memory);
        
        assert_eq!(cpu.a, 0);
        assert_eq!(cpu.x, 0);
        assert_eq!(cpu.y, 0);
        assert_eq!(cpu.sp, 0xFD);
    }

    #[test]
    fn test_stack_operations() {
        let memory = TestMemory::new();
        let mut cpu = Cpu6502::new(memory);
        
        cpu.push(0x42);
        assert_eq!(cpu.sp, 0xFC);
        
        let value = cpu.pop();
        assert_eq!(value, 0x42);
        assert_eq!(cpu.sp, 0xFD);
    }

    #[test]
    fn test_status_flags() {
        let memory = TestMemory::new();
        let mut cpu = Cpu6502::new(memory);
        
        cpu.set_flag(StatusFlags::ZERO, true);
        assert!(cpu.get_flag(StatusFlags::ZERO));
        
        cpu.set_flag(StatusFlags::ZERO, false);
        assert!(!cpu.get_flag(StatusFlags::ZERO));
    }

    #[test]
    fn test_lda_immediate() {
        let mut memory = TestMemory::new();
        memory.ram[0] = 0xA9;  // LDA #$42
        memory.ram[1] = 0x42;
        
        let mut cpu = Cpu6502::new(memory);
        cpu.pc = 0;
        
        let cycles = cpu.step().unwrap();
        
        assert_eq!(cpu.a, 0x42);
        assert_eq!(cpu.pc, 2);
        assert_eq!(cycles, 2);
        assert!(!cpu.get_flag(StatusFlags::ZERO));
        assert!(!cpu.get_flag(StatusFlags::NEGATIVE));
    }

    #[test]
    fn test_lda_zero_flag() {
        let mut memory = TestMemory::new();
        memory.ram[0] = 0xA9;  // LDA #$00
        memory.ram[1] = 0x00;
        
        let mut cpu = Cpu6502::new(memory);
        cpu.pc = 0;
        
        cpu.step().unwrap();
        
        assert_eq!(cpu.a, 0x00);
        assert!(cpu.get_flag(StatusFlags::ZERO));
        assert!(!cpu.get_flag(StatusFlags::NEGATIVE));
    }

    #[test]
    fn test_lda_negative_flag() {
        let mut memory = TestMemory::new();
        memory.ram[0] = 0xA9;  // LDA #$FF
        memory.ram[1] = 0xFF;
        
        let mut cpu = Cpu6502::new(memory);
        cpu.pc = 0;
        
        cpu.step().unwrap();
        
        assert_eq!(cpu.a, 0xFF);
        assert!(!cpu.get_flag(StatusFlags::ZERO));
        assert!(cpu.get_flag(StatusFlags::NEGATIVE));
    }

    #[test]
    fn test_adc_no_carry() {
        let mut memory = TestMemory::new();
        memory.ram[0] = 0xA9;  // LDA #$10
        memory.ram[1] = 0x10;
        memory.ram[2] = 0x69;  // ADC #$20
        memory.ram[3] = 0x20;
        
        let mut cpu = Cpu6502::new(memory);
        cpu.pc = 0;
        
        cpu.step().unwrap();  // LDA
        cpu.step().unwrap();  // ADC
        
        assert_eq!(cpu.a, 0x30);
        assert!(!cpu.get_flag(StatusFlags::CARRY));
        assert!(!cpu.get_flag(StatusFlags::ZERO));
        assert!(!cpu.get_flag(StatusFlags::OVERFLOW));
    }

    #[test]
    fn test_adc_with_carry() {
        let mut memory = TestMemory::new();
        memory.ram[0] = 0xA9;  // LDA #$FF
        memory.ram[1] = 0xFF;
        memory.ram[2] = 0x69;  // ADC #$02
        memory.ram[3] = 0x02;
        
        let mut cpu = Cpu6502::new(memory);
        cpu.pc = 0;
        
        cpu.step().unwrap();  // LDA
        cpu.step().unwrap();  // ADC
        
        assert_eq!(cpu.a, 0x01);
        assert!(cpu.get_flag(StatusFlags::CARRY));
    }

    #[test]
    fn test_sbc() {
        let mut memory = TestMemory::new();
        memory.ram[0] = 0xA9;  // LDA #$50
        memory.ram[1] = 0x50;
        memory.ram[2] = 0x38;  // SEC (set carry)
        memory.ram[3] = 0xE9;  // SBC #$30
        memory.ram[4] = 0x30;
        
        let mut cpu = Cpu6502::new(memory);
        cpu.pc = 0;
        
        cpu.step().unwrap();  // LDA
        cpu.step().unwrap();  // SEC
        cpu.step().unwrap();  // SBC
        
        assert_eq!(cpu.a, 0x20);
        assert!(cpu.get_flag(StatusFlags::CARRY));
    }

    #[test]
    fn test_inc_dec() {
        let mut memory = TestMemory::new();
        memory.ram[0] = 0xA9;  // LDA #$10
        memory.ram[1] = 0x10;
        memory.ram[2] = 0x85;  // STA $20
        memory.ram[3] = 0x20;
        memory.ram[4] = 0xE6;  // INC $20
        memory.ram[5] = 0x20;
        memory.ram[6] = 0xA5;  // LDA $20
        memory.ram[7] = 0x20;
        memory.ram[8] = 0xC6;  // DEC $20
        memory.ram[9] = 0x20;
        memory.ram[10] = 0xA5; // LDA $20
        memory.ram[11] = 0x20;
        
        let mut cpu = Cpu6502::new(memory);
        cpu.pc = 0;
        
        cpu.step().unwrap();  // LDA #$10
        cpu.step().unwrap();  // STA $20
        
        cpu.step().unwrap();  // INC $20
        cpu.step().unwrap();  // LDA $20
        assert_eq!(cpu.a, 0x11);
        
        cpu.step().unwrap();  // DEC $20
        cpu.step().unwrap();  // LDA $20
        assert_eq!(cpu.a, 0x10);
    }

    #[test]
    fn test_inx_iny_dex_dey() {
        let mut memory = TestMemory::new();
        memory.ram[0] = 0xE8;  // INX
        memory.ram[1] = 0xC8;  // INY
        memory.ram[2] = 0xCA;  // DEX
        memory.ram[3] = 0x88;  // DEY
        
        let mut cpu = Cpu6502::new(memory);
        cpu.pc = 0;
        
        assert_eq!(cpu.x, 0);
        assert_eq!(cpu.y, 0);
        
        cpu.step().unwrap();  // INX
        assert_eq!(cpu.x, 1);
        
        cpu.step().unwrap();  // INY
        assert_eq!(cpu.y, 1);
        
        cpu.step().unwrap();  // DEX
        assert_eq!(cpu.x, 0);
        
        cpu.step().unwrap();  // DEY
        assert_eq!(cpu.y, 0);
    }

    #[test]
    fn test_transfer_instructions() {
        let mut memory = TestMemory::new();
        memory.ram[0] = 0xA9;  // LDA #$42
        memory.ram[1] = 0x42;
        memory.ram[2] = 0xAA;  // TAX
        memory.ram[3] = 0xA8;  // TAY
        memory.ram[4] = 0x8A;  // TXA
        memory.ram[5] = 0x98;  // TYA
        
        let mut cpu = Cpu6502::new(memory);
        cpu.pc = 0;
        
        cpu.step().unwrap();  // LDA
        assert_eq!(cpu.a, 0x42);
        
        cpu.step().unwrap();  // TAX
        assert_eq!(cpu.x, 0x42);
        
        cpu.a = 0x30;
        cpu.step().unwrap();  // TAY
        assert_eq!(cpu.y, 0x30);
        
        cpu.step().unwrap();  // TXA
        assert_eq!(cpu.a, 0x42);
        
        cpu.step().unwrap();  // TYA
        assert_eq!(cpu.a, 0x30);
    }

    #[test]
    fn test_and_or_eor() {
        let mut memory = TestMemory::new();
        memory.ram[0] = 0xA9;  // LDA #$FF
        memory.ram[1] = 0xFF;
        memory.ram[2] = 0x29;  // AND #$0F
        memory.ram[3] = 0x0F;
        memory.ram[4] = 0x09;  // ORA #$F0
        memory.ram[5] = 0xF0;
        memory.ram[6] = 0x49;  // EOR #$AA
        memory.ram[7] = 0xAA;
        
        let mut cpu = Cpu6502::new(memory);
        cpu.pc = 0;
        
        cpu.step().unwrap();  // LDA
        assert_eq!(cpu.a, 0xFF);
        
        cpu.step().unwrap();  // AND
        assert_eq!(cpu.a, 0x0F);
        
        cpu.step().unwrap();  // ORA
        assert_eq!(cpu.a, 0xFF);
        
        cpu.step().unwrap();  // EOR
        assert_eq!(cpu.a, 0x55);
    }

    #[test]
    fn test_asl_lsr() {
        let mut memory = TestMemory::new();
        memory.ram[0] = 0xA9;  // LDA #$80
        memory.ram[1] = 0x80;
        memory.ram[2] = 0x0A;  // ASL A
        memory.ram[3] = 0x4A;  // LSR A
        
        let mut cpu = Cpu6502::new(memory);
        cpu.pc = 0;
        
        cpu.step().unwrap();  // LDA
        assert_eq!(cpu.a, 0x80);
        
        cpu.step().unwrap();  // ASL
        assert_eq!(cpu.a, 0x00);
        assert!(cpu.get_flag(StatusFlags::CARRY));
        assert!(cpu.get_flag(StatusFlags::ZERO));
        
        cpu.a = 0x01;
        cpu.set_flag(StatusFlags::CARRY, false);
        cpu.step().unwrap();  // LSR
        assert_eq!(cpu.a, 0x00);
        assert!(cpu.get_flag(StatusFlags::CARRY));
    }

    #[test]
    fn test_cmp() {
        let mut memory = TestMemory::new();
        memory.ram[0] = 0xA9;  // LDA #$50
        memory.ram[1] = 0x50;
        memory.ram[2] = 0xC9;  // CMP #$50 (equal)
        memory.ram[3] = 0x50;
        memory.ram[4] = 0xC9;  // CMP #$30 (greater)
        memory.ram[5] = 0x30;
        memory.ram[6] = 0xC9;  // CMP #$60 (less)
        memory.ram[7] = 0x60;
        
        let mut cpu = Cpu6502::new(memory);
        cpu.pc = 0;
        
        cpu.step().unwrap();  // LDA
        
        cpu.step().unwrap();  // CMP (equal)
        assert!(cpu.get_flag(StatusFlags::CARRY));
        assert!(cpu.get_flag(StatusFlags::ZERO));
        
        cpu.step().unwrap();  // CMP (greater)
        assert!(cpu.get_flag(StatusFlags::CARRY));
        assert!(!cpu.get_flag(StatusFlags::ZERO));
        
        cpu.step().unwrap();  // CMP (less)
        assert!(!cpu.get_flag(StatusFlags::CARRY));
        assert!(!cpu.get_flag(StatusFlags::ZERO));
    }

    #[test]
    fn test_branch_not_taken() {
        let mut memory = TestMemory::new();
        memory.ram[0] = 0xD0;  // BNE (branch if not zero)
        memory.ram[1] = 0x05;  // offset +5
        
        let mut cpu = Cpu6502::new(memory);
        cpu.pc = 0;
        cpu.set_flag(StatusFlags::ZERO, true);  // Z=1, so BNE not taken
        
        let cycles = cpu.step().unwrap();
        
        assert_eq!(cpu.pc, 2);  // Should advance normally
        assert_eq!(cycles, 2);  // 2 cycles for branch not taken
    }

    #[test]
    fn test_branch_taken_no_page_cross() {
        let mut memory = TestMemory::new();
        memory.ram[0] = 0xD0;  // BNE
        memory.ram[1] = 0x05;  // offset +5
        
        let mut cpu = Cpu6502::new(memory);
        cpu.pc = 0;
        cpu.set_flag(StatusFlags::ZERO, false);  // Z=0, so BNE taken
        
        let cycles = cpu.step().unwrap();
        
        assert_eq!(cpu.pc, 7);  // PC=2 + offset 5 = 7
        assert_eq!(cycles, 3);  // 3 cycles for branch taken without page cross
    }

    #[test]
    fn test_jmp_absolute() {
        let mut memory = TestMemory::new();
        memory.ram[0] = 0x4C;  // JMP $1234
        memory.ram[1] = 0x34;
        memory.ram[2] = 0x12;
        
        let mut cpu = Cpu6502::new(memory);
        cpu.pc = 0;
        
        cpu.step().unwrap();
        
        assert_eq!(cpu.pc, 0x1234);
    }

    #[test]
    fn test_jsr_rts() {
        let mut memory = TestMemory::new();
        memory.ram[0x00] = 0x20;  // JSR $1000
        memory.ram[0x01] = 0x00;
        memory.ram[0x02] = 0x10;
        memory.ram[0x1000] = 0x60;  // RTS
        
        let mut cpu = Cpu6502::new(memory);
        cpu.pc = 0;
        
        cpu.step().unwrap();  // JSR
        assert_eq!(cpu.pc, 0x1000);
        assert_eq!(cpu.sp, 0xFB);  // SP decremented by 2
        
        cpu.step().unwrap();  // RTS
        assert_eq!(cpu.pc, 0x03);  // Returns to next instruction after JSR
        assert_eq!(cpu.sp, 0xFD);  // SP restored
    }

    #[test]
    fn test_stack_push_pop() {
        let mut memory = TestMemory::new();
        memory.ram[0] = 0xA9;  // LDA #$42
        memory.ram[1] = 0x42;
        memory.ram[2] = 0x48;  // PHA
        memory.ram[3] = 0xA9;  // LDA #$00
        memory.ram[4] = 0x00;
        memory.ram[5] = 0x68;  // PLA
        
        let mut cpu = Cpu6502::new(memory);
        cpu.pc = 0;
        
        cpu.step().unwrap();  // LDA #$42
        cpu.step().unwrap();  // PHA
        assert_eq!(cpu.sp, 0xFC);
        
        cpu.step().unwrap();  // LDA #$00
        assert_eq!(cpu.a, 0x00);
        
        cpu.step().unwrap();  // PLA
        assert_eq!(cpu.a, 0x42);
        assert_eq!(cpu.sp, 0xFD);
    }
}
