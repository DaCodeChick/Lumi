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
}
