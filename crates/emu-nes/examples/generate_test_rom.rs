/// Simple Test ROM Generator
/// 
/// This creates a minimal valid iNES ROM file for testing.
/// The ROM contains a simple test program that exercises basic CPU functionality.

use std::fs::File;
use std::io::Write;
use std::path::Path;

/// iNES file format header (16 bytes)
#[repr(C)]
struct INesHeader {
    /// "NES" followed by MS-DOS end-of-file marker
    magic: [u8; 4],
    /// Number of 16KB PRG-ROM banks
    prg_rom_banks: u8,
    /// Number of 8KB CHR-ROM banks (0 means CHR-RAM)
    chr_rom_banks: u8,
    /// Flags 6: Mapper, mirroring, battery, trainer
    flags6: u8,
    /// Flags 7: Mapper, VS/Playchoice, NES 2.0
    flags7: u8,
    /// Flags 8: PRG-RAM size (rarely used)
    flags8: u8,
    /// Flags 9: TV system (rarely used)
    flags9: u8,
    /// Flags 10: TV system, PRG-RAM presence (unofficial)
    flags10: u8,
    /// Unused padding
    padding: [u8; 5],
}

impl INesHeader {
    fn new(prg_banks: u8, chr_banks: u8, mapper: u8, mirroring: u8) -> Self {
        Self {
            magic: [b'N', b'E', b'S', 0x1A],
            prg_rom_banks: prg_banks,
            chr_rom_banks: chr_banks,
            flags6: (mapper & 0x0F) << 4 | mirroring & 0x01,
            flags7: mapper & 0xF0,
            flags8: 0,
            flags9: 0,
            flags10: 0,
            padding: [0; 5],
        }
    }
    
    fn to_bytes(&self) -> [u8; 16] {
        let mut bytes = [0u8; 16];
        bytes[0..4].copy_from_slice(&self.magic);
        bytes[4] = self.prg_rom_banks;
        bytes[5] = self.chr_rom_banks;
        bytes[6] = self.flags6;
        bytes[7] = self.flags7;
        bytes[8] = self.flags8;
        bytes[9] = self.flags9;
        bytes[10] = self.flags10;
        bytes[11..16].copy_from_slice(&self.padding);
        bytes
    }
}

fn create_test_rom(path: &Path) -> std::io::Result<()> {
    println!("Creating test ROM: {}", path.display());
    
    // Create iNES header
    // Mapper 0 (NROM), 1x 16KB PRG-ROM, 1x 8KB CHR-ROM, horizontal mirroring
    let header = INesHeader::new(1, 1, 0, 0);
    
    // Create test program (16KB PRG-ROM)
    let mut prg_rom = vec![0xEA; 0x4000]; // Fill with NOP
    
    #[rustfmt::skip]
    let program = vec![
        // Test program starting at $8000
        // Test 1: Basic arithmetic
        0xA9, 0x00,        // LDA #$00      ; Clear accumulator
        0x85, 0x00,        // STA $00       ; Store to $00
        
        0xA9, 0x0A,        // LDA #$0A      ; A = 10
        0x18,              // CLC           ; Clear carry
        0x69, 0x05,        // ADC #$05      ; A = 15
        0x85, 0x01,        // STA $01       ; Store result at $01
        
        // Test 2: Loop counter
        0xA2, 0x00,        // LDX #$00      ; X = 0
        // Loop start at $800E:
        0xE8,              // INX           ; X++
        0x8A,              // TXA           ; A = X
        0x85, 0x02,        // STA $02       ; Store X to $02
        0xE0, 0x0A,        // CPX #$0A      ; Compare X with 10
        0xD0, 0xF8,        // BNE -8        ; Branch back if X != 10
        
        // Test 3: Memory operations
        0xA9, 0x42,        // LDA #$42
        0x85, 0x10,        // STA $10       ; Write to $10
        0xA5, 0x10,        // LDA $10       ; Read back
        0x85, 0x11,        // STA $11       ; Store to $11
        
        // Test 4: Stack operations
        0xA9, 0x99,        // LDA #$99
        0x48,              // PHA           ; Push A
        0xA9, 0x00,        // LDA #$00      ; Clear A
        0x68,              // PLA           ; Pop A (should be $99)
        0x85, 0x20,        // STA $20       ; Store result
        
        // Test 5: Subroutine call
        0x20, 0x50, 0x80,  // JSR $8050     ; Call subroutine
        0x85, 0x30,        // STA $30       ; Store return value
        
        // Success marker
        0xA9, 0xFF,        // LDA #$FF
        0x85, 0x40,        // STA $40       ; Write $FF to $40 as success flag
        
        // Infinite loop
        0x4C, 0x3D, 0x80,  // JMP $803D     ; Jump to self
    ];
    
    // Subroutine at $8050
    #[rustfmt::skip]
    let subroutine = vec![
        0xA9, 0x55,        // LDA #$55      ; Return value
        0x60,              // RTS           ; Return
    ];
    
    // Copy program to ROM
    prg_rom[0..program.len()].copy_from_slice(&program);
    prg_rom[0x50..0x50 + subroutine.len()].copy_from_slice(&subroutine);
    
    // Set reset vector to $8000
    prg_rom[0x3FFC] = 0x00;
    prg_rom[0x3FFD] = 0x80;
    
    // Set IRQ vector (not used, but good practice)
    prg_rom[0x3FFE] = 0x00;
    prg_rom[0x3FFF] = 0x80;
    
    // Create CHR-ROM (8KB, empty for now)
    let chr_rom = vec![0; 0x2000];
    
    // Write file
    let mut file = File::create(path)?;
    file.write_all(&header.to_bytes())?;
    file.write_all(&prg_rom)?;
    file.write_all(&chr_rom)?;
    
    println!("Test ROM created successfully!");
    println!("  Size: {} bytes", 16 + prg_rom.len() + chr_rom.len());
    println!("  PRG-ROM: {} bytes (1 bank)", prg_rom.len());
    println!("  CHR-ROM: {} bytes (1 bank)", chr_rom.len());
    println!("\nTest program verification points:");
    println!("  $01 should be $0F (10 + 5 = 15)");
    println!("  $02 should be $0A (loop counter = 10)");
    println!("  $11 should be $42 (memory read/write)");
    println!("  $20 should be $99 (stack operations)");
    println!("  $30 should be $55 (subroutine return)");
    println!("  $40 should be $FF (success flag)");
    
    Ok(())
}

fn main() -> std::io::Result<()> {
    println!("=== LumiEmu Test ROM Generator ===\n");
    
    // Create test ROM in the project root
    let rom_path = Path::new("test.nes");
    create_test_rom(rom_path)?;
    
    println!("\nROM ready for testing: {}", rom_path.display());
    Ok(())
}
