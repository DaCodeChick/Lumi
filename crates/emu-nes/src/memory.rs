/// NES Memory System
/// 
/// The NES has a 16-bit address space with the following memory map:
/// - $0000-$07FF: 2KB internal RAM
/// - $0800-$1FFF: Mirrors of $0000-$07FF (repeats 3 times)
/// - $2000-$2007: PPU registers
/// - $2008-$3FFF: Mirrors of $2000-$2007 (repeats ~1024 times)
/// - $4000-$4017: APU and I/O registers
/// - $4018-$401F: APU and I/O functionality that is normally disabled
/// - $4020-$FFFF: Cartridge space (PRG-ROM, PRG-RAM, and mapper registers)
///
/// For Phase 2, we'll implement basic RAM and stub out PPU/APU registers.
/// Cartridge memory will be handled by the Cartridge module.

use crate::cpu::CpuMemory;
use emu_core::{MemoryBus, MemoryObserver, EmulatorContext};

/// NES Memory system
pub struct NesMemory {
    /// 2KB of internal RAM ($0000-$07FF, mirrored to $1FFF)
    ram: [u8; 0x0800],
    
    /// PPU register values (for now just storage, no real PPU yet)
    ppu_regs: [u8; 8],
    
    /// APU/IO register values (stubbed for now)
    apu_io_regs: [u8; 0x18],
    
    /// Cartridge reference (optional, for now)
    /// In the future, this will be a trait object for different mappers
    cartridge_prg: Option<Vec<u8>>,
    
    /// Memory observers for AI pattern detection
    observers: Vec<Box<dyn MemoryObserver>>,
    
    /// Current emulator context
    context: EmulatorContext,
}

impl NesMemory {
    /// Create a new NES memory system
    pub fn new() -> Self {
        Self {
            ram: [0; 0x0800],
            ppu_regs: [0; 8],
            apu_io_regs: [0; 0x18],
            cartridge_prg: None,
            observers: Vec::new(),
            context: EmulatorContext {
                frame: 0,
                cycle: 0,
                pc: 0,
                last_input: 0,
            },
        }
    }
    
    /// Load PRG-ROM data (temporary, will be replaced with proper cartridge system)
    pub fn load_prg_rom(&mut self, data: Vec<u8>) {
        self.cartridge_prg = Some(data);
    }
    
    /// Internal read without observer notification
    fn read_internal(&mut self, addr: u16) -> u8 {
        match addr {
            // 2KB internal RAM + mirrors
            0x0000..=0x1FFF => {
                let mirrored_addr = (addr & 0x07FF) as usize;
                self.ram[mirrored_addr]
            }
            
            // PPU registers (mirrored every 8 bytes)
            0x2000..=0x3FFF => {
                let reg = (addr & 0x0007) as usize;
                self.ppu_regs[reg]
            }
            
            // APU and I/O registers
            0x4000..=0x4017 => {
                let reg = (addr - 0x4000) as usize;
                self.apu_io_regs[reg]
            }
            
            // Cartridge space
            0x4020..=0xFFFF => {
                if let Some(ref prg) = self.cartridge_prg {
                    // Simple mapper 0 (NROM): 16KB or 32KB PRG-ROM
                    let rom_addr = if prg.len() <= 0x4000 {
                        // 16KB ROM: mirror at $C000
                        (addr & 0x3FFF) as usize
                    } else {
                        // 32KB ROM: linear mapping
                        (addr - 0x8000) as usize
                    };
                    
                    if rom_addr < prg.len() {
                        prg[rom_addr]
                    } else {
                        0xFF // Open bus
                    }
                } else {
                    0xFF // No cartridge loaded
                }
            }
            
            _ => 0xFF, // Open bus
        }
    }
    
    /// Internal write without observer notification
    fn write_internal(&mut self, addr: u16, value: u8) {
        match addr {
            // 2KB internal RAM + mirrors
            0x0000..=0x1FFF => {
                let mirrored_addr = (addr & 0x07FF) as usize;
                self.ram[mirrored_addr] = value;
            }
            
            // PPU registers (mirrored every 8 bytes)
            0x2000..=0x3FFF => {
                let reg = (addr & 0x0007) as usize;
                self.ppu_regs[reg] = value;
                // TODO: Actually communicate with PPU
            }
            
            // APU and I/O registers
            0x4000..=0x4017 => {
                let reg = (addr - 0x4000) as usize;
                self.apu_io_regs[reg] = value;
                // TODO: Actually communicate with APU/IO
            }
            
            // Cartridge space - writes to ROM are typically ignored
            // (unless there's RAM or mapper registers, which we'll handle later)
            0x4020..=0xFFFF => {
                // Ignore writes to ROM for now
            }
            
            _ => {
                // Ignore writes to invalid addresses
            }
        }
    }
}

impl Default for NesMemory {
    fn default() -> Self {
        Self::new()
    }
}

impl CpuMemory for NesMemory {
    fn read(&mut self, addr: u16) -> u8 {
        let value = self.read_internal(addr);
        
        // Notify observers
        let context = self.context;
        for observer in &mut self.observers {
            observer.on_read(addr, value, &context);
        }
        
        value
    }
    
    fn write(&mut self, addr: u16, value: u8) {
        // Get old value for observers
        let old_value = self.read_internal(addr);
        
        // Perform write
        self.write_internal(addr, value);
        
        // Notify observers
        let context = self.context;
        for observer in &mut self.observers {
            observer.on_write(addr, old_value, value, &context);
        }
    }
}

impl MemoryBus for NesMemory {
    fn read(&mut self, addr: u16) -> u8 {
        // Use CpuMemory implementation
        CpuMemory::read(self, addr)
    }
    
    fn write(&mut self, addr: u16, value: u8) {
        // Use CpuMemory implementation
        CpuMemory::write(self, addr, value)
    }
    
    fn attach_observer(&mut self, observer: Box<dyn MemoryObserver>) {
        self.observers.push(observer);
    }
    
    fn clear_observers(&mut self) {
        self.observers.clear();
    }
    
    fn context(&self) -> EmulatorContext {
        self.context
    }
    
    fn update_context(&mut self, context: EmulatorContext) {
        self.context = context;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ram_basic_readwrite() {
        let mut mem = NesMemory::new();
        
        CpuMemory::write(&mut mem, 0x0000, 0x42);
        assert_eq!(CpuMemory::read(&mut mem, 0x0000), 0x42);
        
        CpuMemory::write(&mut mem, 0x07FF, 0x99);
        assert_eq!(CpuMemory::read(&mut mem, 0x07FF), 0x99);
    }
    
    #[test]
    fn test_ram_mirroring() {
        let mut mem = NesMemory::new();
        
        // Write to $0000, should be mirrored at $0800, $1000, $1800
        CpuMemory::write(&mut mem, 0x0000, 0x42);
        assert_eq!(CpuMemory::read(&mut mem, 0x0800), 0x42);
        assert_eq!(CpuMemory::read(&mut mem, 0x1000), 0x42);
        assert_eq!(CpuMemory::read(&mut mem, 0x1800), 0x42);
        
        // Write to $0234 (via mirror $1234), should be readable at all mirrors
        CpuMemory::write(&mut mem, 0x1234, 0x99);
        assert_eq!(CpuMemory::read(&mut mem, 0x0234), 0x99); // Original
        assert_eq!(CpuMemory::read(&mut mem, 0x0A34), 0x99); // Mirror in $0800 range
        assert_eq!(CpuMemory::read(&mut mem, 0x1A34), 0x99); // Mirror in $1800 range
    }
    
    #[test]
    fn test_ppu_register_mirroring() {
        let mut mem = NesMemory::new();
        
        // PPU has 8 registers, mirrored throughout $2000-$3FFF
        CpuMemory::write(&mut mem, 0x2000, 0x42);
        assert_eq!(CpuMemory::read(&mut mem, 0x2000), 0x42);
        assert_eq!(CpuMemory::read(&mut mem, 0x2008), 0x42);
        assert_eq!(CpuMemory::read(&mut mem, 0x3000), 0x42);
        
        CpuMemory::write(&mut mem, 0x2007, 0x99);
        assert_eq!(CpuMemory::read(&mut mem, 0x2007), 0x99);
        assert_eq!(CpuMemory::read(&mut mem, 0x200F), 0x99);
        assert_eq!(CpuMemory::read(&mut mem, 0x3FFF), 0x99);
    }
    
    #[test]
    fn test_cartridge_16kb() {
        let mut mem = NesMemory::new();
        
        // Load 16KB ROM
        let rom = vec![0x42; 0x4000];
        mem.load_prg_rom(rom);
        
        // Read from $8000-$BFFF
        assert_eq!(CpuMemory::read(&mut mem, 0x8000), 0x42);
        assert_eq!(CpuMemory::read(&mut mem, 0xBFFF), 0x42);
        
        // $C000-$FFFF should mirror $8000-$BFFF
        assert_eq!(CpuMemory::read(&mut mem, 0xC000), 0x42);
        assert_eq!(CpuMemory::read(&mut mem, 0xFFFF), 0x42);
    }
    
    #[test]
    fn test_cartridge_32kb() {
        let mut mem = NesMemory::new();
        
        // Load 32KB ROM with different patterns in each half
        let mut rom = vec![0x11; 0x4000];
        rom.extend(vec![0x22; 0x4000]);
        mem.load_prg_rom(rom);
        
        // First 16KB
        assert_eq!(CpuMemory::read(&mut mem, 0x8000), 0x11);
        assert_eq!(CpuMemory::read(&mut mem, 0xBFFF), 0x11);
        
        // Second 16KB
        assert_eq!(CpuMemory::read(&mut mem, 0xC000), 0x22);
        assert_eq!(CpuMemory::read(&mut mem, 0xFFFF), 0x22);
    }
}
