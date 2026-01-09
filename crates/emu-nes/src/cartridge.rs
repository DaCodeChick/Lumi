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
    /// Implements Mapper 0 (NROM) logic
    pub fn read_prg(&self, addr: u16) -> u8 {
        match self.header.mapper {
            0 => self.read_prg_mapper0(addr),
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
    
    /// Write to PRG address space (usually ignored, but some mappers use this for banking)
    pub fn write_prg(&mut self, _addr: u16, _value: u8) {
        // Mapper 0 has no writable registers
        // In the future, other mappers will handle banking here
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
