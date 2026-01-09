/// Top-level NES System
/// 
/// Ties together CPU, memory, and cartridge into a complete NES emulator.

use crate::{Cartridge, Cpu6502, NesMemory};
use crate::cpu::CpuMemory;
use emu_core::{Cpu, EmulatorError, Result};
use std::path::Path;

/// NES Emulator System
pub struct NesSystem {
    /// 6502 CPU
    cpu: Cpu6502<NesMemory>,
    /// Frame counter
    frame: u64,
}

impl NesSystem {
    /// Create a new NES system with a cartridge loaded from file
    pub fn new(rom_path: &Path) -> Result<Self> {
        // Load cartridge
        let cartridge = Cartridge::load(rom_path)?;
        
        // Check mapper support
        if cartridge.header().mapper != 0 {
            return Err(EmulatorError::UnsupportedMapper(cartridge.header().mapper));
        }
        
        // Create memory system and load cartridge
        let mut memory = NesMemory::new();
        memory.load_cartridge(cartridge);
        
        // Create CPU
        let mut cpu = Cpu6502::new(memory);
        
        // Reset CPU (this will read the reset vector from $FFFC-$FFFD)
        cpu.reset();
        
        Ok(Self {
            cpu,
            frame: 0,
        })
    }
    
    /// Load a ROM from a file path (convenience method)
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        Self::new(path.as_ref())
    }
    
    /// Create a NES system with raw PRG-ROM data (for testing)
    pub fn with_prg_rom(prg_rom: Vec<u8>) -> Result<Self> {
        let mut memory = NesMemory::new();
        memory.load_prg_rom(prg_rom);
        
        let mut cpu = Cpu6502::new(memory);
        cpu.reset();
        
        Ok(Self {
            cpu,
            frame: 0,
        })
    }
    
    /// Reset the system
    pub fn reset(&mut self) {
        self.cpu.reset();
        self.frame = 0;
    }
    
    /// Step one CPU instruction
    pub fn step(&mut self) -> Result<u8> {
        let cycles = self.cpu.step()?;
        
        // PPU runs 3x faster than CPU
        for _ in 0..(cycles * 3) {
            self.cpu.memory().ppu_mut().tick();
            
            // Check for NMI interrupt
            if self.cpu.memory().ppu().nmi_interrupt {
                self.cpu.memory().ppu_mut().nmi_interrupt = false;
                self.cpu.nmi();
            }
        }
        
        Ok(cycles)
    }
    
    /// Run for a specified number of cycles
    pub fn run_cycles(&mut self, cycles: u64) -> Result<()> {
        let target = self.cpu.cycles + cycles;
        while self.cpu.cycles < target {
            self.cpu.step()?;
        }
        Ok(())
    }
    
    /// Run for one frame (approximately 29780 cycles for NTSC)
    pub fn run_frame(&mut self) -> Result<()> {
        const CYCLES_PER_FRAME: u64 = 29780;
        self.run_cycles(CYCLES_PER_FRAME)?;
        self.frame += 1;
        Ok(())
    }
    
    /// Get current frame number
    pub fn frame(&self) -> u64 {
        self.frame
    }
    
    /// Get CPU reference
    pub fn cpu(&self) -> &Cpu6502<NesMemory> {
        &self.cpu
    }
    
    /// Get mutable CPU reference
    pub fn cpu_mut(&mut self) -> &mut Cpu6502<NesMemory> {
        &mut self.cpu
    }
    
    /// Read from memory
    pub fn read_memory(&mut self, addr: u16) -> u8 {
        self.cpu.memory().read(addr)
    }
    
    /// Get framebuffer from PPU
    pub fn framebuffer(&mut self) -> &[u8] {
        self.cpu.memory().ppu().framebuffer()
    }
    
    /// Get PPU reference
    pub fn ppu(&mut self) -> &crate::ppu::Ppu {
        self.cpu.memory().ppu()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_system_creation() {
        // Create a simple test ROM
        let mut prg_rom = vec![0xEA; 0x4000]; // NOP instructions
        
        // Set reset vector to $8000
        prg_rom[0x3FFC] = 0x00;
        prg_rom[0x3FFD] = 0x80;
        
        let system = NesSystem::with_prg_rom(prg_rom).unwrap();
        
        // CPU should have initialized with PC at reset vector
        assert_eq!(system.cpu().pc, 0x8000);
    }
    
    #[test]
    fn test_system_step() {
        // Create a simple program
        let mut prg_rom = vec![0xEA; 0x4000];
        
        // Program: LDA #$42, STA $00
        prg_rom[0] = 0xA9; // LDA #$42
        prg_rom[1] = 0x42;
        prg_rom[2] = 0x85; // STA $00
        prg_rom[3] = 0x00;
        
        // Reset vector
        prg_rom[0x3FFC] = 0x00;
        prg_rom[0x3FFD] = 0x80;
        
        let mut system = NesSystem::with_prg_rom(prg_rom).unwrap();
        
        // Step twice
        system.step().unwrap();
        assert_eq!(system.cpu().a, 0x42);
        
        system.step().unwrap();
        assert_eq!(system.read_memory(0x00), 0x42);
    }
}
