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

use crate::apu::Apu;
use crate::cpu::CpuMemory;
use crate::cartridge::Cartridge;
use crate::ppu::Ppu;
use emu_core::{Controller, MemoryBus, MemoryObserver, EmulatorContext};

/// NES Memory system
pub struct NesMemory {
    /// 2KB of internal RAM ($0000-$07FF, mirrored to $1FFF)
    ram: [u8; 0x0800],
    
    /// PPU (handles $2000-$2007 registers)
    ppu: Ppu,
    
    /// APU (handles $4000-$4017 registers)
    apu: Apu,
    
    /// Controller 1
    controller1: Controller,
    
    /// Controller 2
    controller2: Controller,
    
    /// Cartridge (optional)
    cartridge: Option<Cartridge>,
    
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
            ppu: Ppu::new(),
            apu: Apu::new(),
            controller1: Controller::new(),
            controller2: Controller::new(),
            cartridge: None,
            observers: Vec::new(),
            context: EmulatorContext {
                frame: 0,
                cycle: 0,
                pc: 0,
                last_input: 0,
            },
        }
    }
    
    /// Get PPU reference
    pub fn ppu(&self) -> &Ppu {
        &self.ppu
    }
    
    /// Get mutable PPU reference
    pub fn ppu_mut(&mut self) -> &mut Ppu {
        &mut self.ppu
    }
    
    /// Get APU reference
    pub fn apu(&self) -> &Apu {
        &self.apu
    }
    
    /// Get mutable APU reference
    pub fn apu_mut(&mut self) -> &mut Apu {
        &mut self.apu
    }
    
    /// Get controller 1 reference
    pub fn controller1(&mut self) -> &mut Controller {
        &mut self.controller1
    }
    
    /// Get controller 2 reference
    pub fn controller2(&mut self) -> &mut Controller {
        &mut self.controller2
    }
    
    /// Load a cartridge
    pub fn load_cartridge(&mut self, cartridge: Cartridge) {
        // Load CHR-ROM into PPU
        // For mappers with CHR banking (like mapper 66), only load the first bank
        if cartridge.header().mapper == 66 {
            // Load only first 8KB bank for mapper 66
            let chr_bank_0 = if cartridge.chr_rom().len() >= 0x2000 {
                cartridge.chr_rom()[0..0x2000].to_vec()
            } else {
                cartridge.chr_rom().to_vec()
            };
            self.ppu.load_chr_rom(chr_bank_0);
        } else {
            // Mapper 0: load all CHR-ROM (max 8KB)
            self.ppu.load_chr_rom(cartridge.chr_rom().to_vec());
        }
        self.cartridge = Some(cartridge);
    }
    
    /// Load PRG-ROM data directly (for testing, bypasses cartridge system)
    pub fn load_prg_rom(&mut self, data: Vec<u8>) {
        // Create a fake cartridge for testing
        let fake_cart = Cartridge {
            prg_rom: data,
            chr_rom: vec![0; 0x2000],
            header: crate::cartridge::INesHeader {
                prg_rom_banks: 1,
                chr_rom_banks: 1,
                mapper: 0,
                mirroring: crate::cartridge::Mirroring::Horizontal,
                has_battery: false,
                has_trainer: false,
            },
            mapper_state: Default::default(),
        };
        self.cartridge = Some(fake_cart);
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
                self.ppu.read_register(addr)
            }
            
            // APU and I/O registers
            0x4000..=0x4017 => {
                match addr {
                    0x4016 => {
                        // Controller 1
                        // Return bit 0 = controller data, bits 1-4 = open bus, bits 5-7 = 0
                        self.controller1.read() | 0x40
                    }
                    0x4017 => {
                        // Controller 2
                        self.controller2.read() | 0x40
                    }
                    0x4015 => {
                        // APU status register
                        self.apu.read_register(addr)
                    }
                    _ => {
                        // Other APU registers (write-only)
                        0
                    }
                }
            }
            
            // Cartridge space
            0x4020..=0xFFFF => {
                if let Some(ref cart) = self.cartridge {
                    cart.read_prg(addr)
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
                self.ppu.write_register(addr, value);
            }
            
            // APU and I/O registers
            0x4000..=0x4017 => {
                match addr {
                    0x4016 => {
                        // Controller strobe
                        self.controller1.write(value);
                        self.controller2.write(value);
                    }
                    0x4000..=0x4015 | 0x4017 => {
                        // APU registers
                        self.apu.write_register(addr, value);
                    }
                    _ => {}
                }
            }
            
            // Cartridge space - mapper registers
            0x4020..=0xFFFF => {
                if let Some(ref mut cart) = self.cartridge {
                    let old_chr_bank = cart.mapper_state.chr_bank;
                    let old_prg_bank = cart.mapper_state.prg_bank;
                    cart.write_prg(addr, value);
                    
                    // For Mapper 66: Update PPU CHR bank if it changed
                    if cart.header().mapper == 66 {
                        if cart.mapper_state.chr_bank != old_chr_bank {
                            let chr_bank = cart.mapper_state.chr_bank as usize;
                            println!("Mapper 66: CHR bank changed to {} (value=${:02X} at ${:04X})", chr_bank, value, addr);
                            self.ppu.load_chr_bank(cart.chr_rom(), chr_bank);
                        }
                        if cart.mapper_state.prg_bank != old_prg_bank {
                            println!("Mapper 66: PRG bank changed to {} (value=${:02X} at ${:04X})", cart.mapper_state.prg_bank, value, addr);
                        }
                    }
                }
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
        // Test with $2004 (OAMDATA) which is read/write
        CpuMemory::write(&mut mem, 0x2003, 0x00); // Set OAMADDR to 0
        CpuMemory::write(&mut mem, 0x2004, 0x42); // Write to OAM
        CpuMemory::write(&mut mem, 0x2003, 0x00); // Reset OAMADDR
        assert_eq!(CpuMemory::read(&mut mem, 0x2004), 0x42); // Read from $2004
        assert_eq!(CpuMemory::read(&mut mem, 0x200C), 0x42); // Mirror at $200C
        assert_eq!(CpuMemory::read(&mut mem, 0x3004), 0x42); // Mirror at $3004
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
