/// NES Cartridge and iNES ROM file format support
/// 
/// Implements loading and parsing of iNES format ROM files (.nes)
/// and provides memory mapping for different mappers.

use std::fs::File;
use std::io::Read;
use std::path::Path;
use emu_core::{EmulatorError, Result};

/// Mirroring mode for nametables
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mirroring {
    Horizontal,
    Vertical,
    FourScreen,
}

/// iNES file format header
#[derive(Debug)]
pub struct INesHeader {
    /// Number of 16KB PRG-ROM banks
    pub prg_rom_banks: u8,
    /// Number of 8KB CHR-ROM banks (0 means CHR-RAM)
    pub chr_rom_banks: u8,
    /// Mapper number (0-255)
    pub mapper: u8,
    /// Nametable mirroring
    pub mirroring: Mirroring,
    /// Has battery-backed PRG-RAM
    pub has_battery: bool,
    /// Has 512-byte trainer
    pub has_trainer: bool,
}

impl INesHeader {
    /// Parse iNES header from 16 bytes
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 16 {
            return Err(EmulatorError::RomLoadError("Header too short".into()));
        }
        
        // Check magic number "NES\x1A"
        if &bytes[0..4] != b"NES\x1A" {
            return Err(EmulatorError::RomLoadError("Invalid iNES magic number".into()));
        }
        
        let prg_rom_banks = bytes[4];
        let chr_rom_banks = bytes[5];
        let flags6 = bytes[6];
        let flags7 = bytes[7];
        
        // Extract mapper number (upper 4 bits of flags7, lower 4 bits of flags6)
        let mapper = (flags7 & 0xF0) | (flags6 >> 4);
        
        // Extract mirroring
        let four_screen = flags6 & 0x08 != 0;
        let vertical = flags6 & 0x01 != 0;
        let mirroring = if four_screen {
            Mirroring::FourScreen
        } else if vertical {
            Mirroring::Vertical
        } else {
            Mirroring::Horizontal
        };
        
        let has_battery = flags6 & 0x02 != 0;
        let has_trainer = flags6 & 0x04 != 0;
        
        Ok(Self {
            prg_rom_banks,
            chr_rom_banks,
            mapper,
            mirroring,
            has_battery,
            has_trainer,
        })
    }
}

/// NES Cartridge
pub struct Cartridge {
    /// PRG-ROM (program code)
    pub(crate) prg_rom: Vec<u8>,
    /// CHR-ROM (graphics data) - may be empty if using CHR-RAM
    pub(crate) chr_rom: Vec<u8>,
    /// Cartridge header
    pub(crate) header: INesHeader,
    /// Mapper state (for banking)
    pub(crate) mapper_state: MapperState,
}

/// Mapper-specific state
#[derive(Debug, Default)]
pub(crate) struct MapperState {
    /// Current PRG bank (for mapper 66)
    pub(crate) prg_bank: u8,
    /// Current CHR bank (for mapper 66)
    pub(crate) chr_bank: u8,
}

impl Cartridge {
    /// Load a cartridge from an iNES file
    pub fn load(path: &Path) -> Result<Self> {
        let mut file = File::open(path)
            .map_err(|e| EmulatorError::RomLoadError(format!("Failed to open ROM: {}", e)))?;
        
        // Read header
        let mut header_bytes = [0u8; 16];
        file.read_exact(&mut header_bytes)
            .map_err(|e| EmulatorError::RomLoadError(format!("Failed to read header: {}", e)))?;
        
        let header = INesHeader::parse(&header_bytes)?;
        
        // Skip trainer if present
        if header.has_trainer {
            let mut trainer = [0u8; 512];
            file.read_exact(&mut trainer)
                .map_err(|e| EmulatorError::RomLoadError(format!("Failed to read trainer: {}", e)))?;
        }
        
        // Read PRG-ROM
        let prg_size = header.prg_rom_banks as usize * 0x4000; // 16KB banks
        let mut prg_rom = vec![0u8; prg_size];
        file.read_exact(&mut prg_rom)
            .map_err(|e| EmulatorError::RomLoadError(format!("Failed to read PRG-ROM: {}", e)))?;
        
        // Read CHR-ROM (if present)
        let chr_size = header.chr_rom_banks as usize * 0x2000; // 8KB banks
        let chr_rom = if chr_size > 0 {
            let mut chr = vec![0u8; chr_size];
            file.read_exact(&mut chr)
                .map_err(|e| EmulatorError::RomLoadError(format!("Failed to read CHR-ROM: {}", e)))?;
            chr
        } else {
            // CHR-RAM: 8KB
            vec![0; 0x2000]
        };
        
        Ok(Self {
            prg_rom,
            chr_rom,
            header,
            mapper_state: MapperState::default(),
        })
    }
    
    /// Get PRG-ROM data
    pub fn prg_rom(&self) -> &[u8] {
        &self.prg_rom
    }
    
    /// Get CHR-ROM data
    pub fn chr_rom(&self) -> &[u8] {
        &self.chr_rom
    }
    
    /// Get CHR-ROM data (mutable) for CHR-RAM
    pub fn chr_rom_mut(&mut self) -> &mut [u8] {
        &mut self.chr_rom
    }
    
    /// Get cartridge header
    pub fn header(&self) -> &INesHeader {
        &self.header
    }
    
    /// Read from PRG-ROM address space ($8000-$FFFF)
    /// Implements Mapper 0 (NROM) and Mapper 66 (GxROM) logic
    pub fn read_prg(&self, addr: u16) -> u8 {
        match self.header.mapper {
            0 => self.read_prg_mapper0(addr),
            66 => self.read_prg_mapper66(addr),
            _ => {
                // Unsupported mapper - return open bus
                0xFF
            }
        }
    }
    
    /// Mapper 0 (NROM) PRG-ROM read
    /// - 16KB: $8000-$BFFF and $C000-$FFFF mirror the same 16KB
    /// - 32KB: $8000-$FFFF is linear
    fn read_prg_mapper0(&self, addr: u16) -> u8 {
        let rom_addr = if self.prg_rom.len() <= 0x4000 {
            // 16KB ROM: mirror at $C000
            (addr & 0x3FFF) as usize
        } else {
            // 32KB ROM: linear mapping
            (addr - 0x8000) as usize
        };
        
        if rom_addr < self.prg_rom.len() {
            self.prg_rom[rom_addr]
        } else {
            0xFF
        }
    }
    
    /// Mapper 66 (GxROM) PRG-ROM read
    /// - 32KB banks switchable
    /// - Bank selected by bits 4-5 of mapper register
    fn read_prg_mapper66(&self, addr: u16) -> u8 {
        let bank = self.mapper_state.prg_bank as usize;
        let offset = (addr - 0x8000) as usize;
        let rom_addr = (bank * 0x8000) + offset;
        
        if rom_addr < self.prg_rom.len() {
            self.prg_rom[rom_addr]
        } else {
            0xFF
        }
    }
    
    /// Write to PRG address space (for mapper register updates)
    pub fn write_prg(&mut self, addr: u16, value: u8) {
        match self.header.mapper {
            0 => {
                // Mapper 0 has no writable registers
            }
            66 => self.write_prg_mapper66(addr, value),
            _ => {}
        }
    }
    
    /// Mapper 66 (GxROM) register write
    /// Write to $8000-$FFFF sets banking
    /// Bits 4-5: PRG bank (0-3)
    /// Bits 0-1: CHR bank (0-3)
    fn write_prg_mapper66(&mut self, _addr: u16, value: u8) {
        self.mapper_state.prg_bank = (value >> 4) & 0x03;
        self.mapper_state.chr_bank = value & 0x03;
    }
    
    /// Read from CHR-ROM/RAM address space ($0000-$1FFF)
    /// Used by PPU for pattern tables
    pub fn read_chr(&self, addr: u16) -> u8 {
        match self.header.mapper {
            0 => {
                // Mapper 0: direct access
                let addr = addr as usize;
                if addr < self.chr_rom.len() {
                    self.chr_rom[addr]
                } else {
                    0
                }
            }
            66 => {
                // Mapper 66: 8KB banks
                let bank = self.mapper_state.chr_bank as usize;
                let offset = addr as usize;
                let chr_addr = (bank * 0x2000) + offset;
                
                if chr_addr < self.chr_rom.len() {
                    self.chr_rom[chr_addr]
                } else {
                    0
                }
            }
            _ => 0,
        }
    }
    
    /// Write to CHR-ROM/RAM address space ($0000-$1FFF)
    /// Only works for CHR-RAM (when chr_rom_banks == 0)
    pub fn write_chr(&mut self, addr: u16, value: u8) {
        match self.header.mapper {
            0 => {
                // Mapper 0: direct access if CHR-RAM
                if self.header.chr_rom_banks == 0 {
                    let addr = addr as usize;
                    if addr < self.chr_rom.len() {
                        self.chr_rom[addr] = value;
                    }
                }
            }
            66 => {
                // Mapper 66: CHR-RAM write through bank
                if self.header.chr_rom_banks == 0 {
                    let bank = self.mapper_state.chr_bank as usize;
                    let offset = addr as usize;
                    let chr_addr = (bank * 0x2000) + offset;
                    
                    if chr_addr < self.chr_rom.len() {
                        self.chr_rom[chr_addr] = value;
                    }
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ines_header_parse() {
        let header_bytes = [
            b'N', b'E', b'S', 0x1A,  // Magic
            0x02,                     // 2 PRG banks
            0x01,                     // 1 CHR bank
            0x00,                     // Flags 6: mapper 0, horizontal mirroring
            0x00,                     // Flags 7: mapper 0
            0x00, 0x00, 0x00, 0x00,  // Unused
            0x00, 0x00, 0x00, 0x00,  // Unused
        ];
        
        let header = INesHeader::parse(&header_bytes).unwrap();
        assert_eq!(header.prg_rom_banks, 2);
        assert_eq!(header.chr_rom_banks, 1);
        assert_eq!(header.mapper, 0);
        assert_eq!(header.mirroring, Mirroring::Horizontal);
        assert!(!header.has_battery);
        assert!(!header.has_trainer);
    }
    
    #[test]
    fn test_ines_header_vertical_mirroring() {
        let header_bytes = [
            b'N', b'E', b'S', 0x1A,
            0x01, 0x01,
            0x01,  // Vertical mirroring bit set
            0x00,
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];
        
        let header = INesHeader::parse(&header_bytes).unwrap();
        assert_eq!(header.mirroring, Mirroring::Vertical);
    }
    
    #[test]
    fn test_ines_header_mapper_number() {
        let header_bytes = [
            b'N', b'E', b'S', 0x1A,
            0x01, 0x01,
            0x10,  // Mapper low nibble = 1
            0x20,  // Mapper high nibble = 2
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];
        
        let header = INesHeader::parse(&header_bytes).unwrap();
        assert_eq!(header.mapper, 0x21);  // 0x20 | 0x01
    }
    
    #[test]
    fn test_invalid_magic() {
        let header_bytes = [
            b'X', b'X', b'X', 0x1A,
            0x01, 0x01, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];
        
        assert!(INesHeader::parse(&header_bytes).is_err());
    }
}
