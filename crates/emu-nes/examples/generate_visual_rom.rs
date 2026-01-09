/// Generate a visual test ROM for the NES emulator
/// 
/// This ROM:
/// 1. Sets up the PPU with rendering enabled
/// 2. Loads a palette with multiple colors
/// 3. Draws patterns to the nametable
/// 4. Displays a simple test pattern on screen

use std::fs::File;
use std::io::{self, Write};

fn main() -> io::Result<()> {
    println!("Generating visual test ROM...\n");
    
    // Create PRG-ROM (16KB)
    let mut prg_rom = vec![0; 0x4000];
    
    // Create CHR-ROM (8KB) with some simple tile patterns
    let chr_rom = generate_chr_rom();
    
    // Generate the 6502 assembly program
    generate_program(&mut prg_rom);
    
    // Create iNES header
    let header = create_ines_header(1, 1); // 1x16KB PRG, 1x8KB CHR
    
    // Write the complete ROM file
    let output_path = "visual_test.nes";
    let mut file = File::create(output_path)?;
    file.write_all(&header)?;
    file.write_all(&prg_rom)?;
    file.write_all(&chr_rom)?;
    
    println!("ROM generated: {}", output_path);
    println!("  PRG-ROM: {} bytes", prg_rom.len());
    println!("  CHR-ROM: {} bytes", chr_rom.len());
    println!("  Total: {} bytes\n", header.len() + prg_rom.len() + chr_rom.len());
    
    Ok(())
}

fn create_ines_header(prg_banks: u8, chr_banks: u8) -> Vec<u8> {
    vec![
        0x4E, 0x45, 0x53, 0x1A, // "NES^Z" magic
        prg_banks,               // PRG-ROM size in 16KB units
        chr_banks,               // CHR-ROM size in 8KB units
        0x00,                    // Mapper 0, horizontal mirroring
        0x00,                    // Mapper 0 upper bits
        0x00, 0x00, 0x00, 0x00,  // Padding
        0x00, 0x00, 0x00, 0x00,  // More padding
    ]
}

fn generate_chr_rom() -> Vec<u8> {
    let mut chr = vec![0; 0x2000]; // 8KB
    
    // Generate some simple tile patterns
    // Tile 0: Solid (all pixels on)
    for i in 0..8 {
        chr[i] = 0xFF;
        chr[i + 8] = 0xFF;
    }
    
    // Tile 1: Checkerboard pattern
    for i in 0..8 {
        if i % 2 == 0 {
            chr[16 + i] = 0xAA; // 10101010
            chr[16 + i + 8] = 0xAA;
        } else {
            chr[16 + i] = 0x55; // 01010101
            chr[16 + i + 8] = 0x55;
        }
    }
    
    // Tile 2: Horizontal stripes
    for i in 0..8 {
        if i < 4 {
            chr[32 + i] = 0xFF;
            chr[32 + i + 8] = 0xFF;
        }
    }
    
    // Tile 3: Vertical stripes
    for i in 0..8 {
        chr[48 + i] = 0xF0; // 11110000
        chr[48 + i + 8] = 0xF0;
    }
    
    // Tile 4: Border
    for i in 0..8 {
        if i == 0 || i == 7 {
            chr[64 + i] = 0xFF;
            chr[64 + i + 8] = 0xFF;
        } else {
            chr[64 + i] = 0x81; // 10000001
            chr[64 + i + 8] = 0x81;
        }
    }
    
    chr
}

fn generate_program(prg_rom: &mut [u8]) {
    let mut pc = 0;
    
    // RESET handler - this is where the program starts
    let reset_addr = 0x8000;
    
    // Program start at $8000
    pc = 0x0000; // Offset in PRG-ROM (maps to $8000 in CPU space)
    
    // Wait for PPU to be ready (2 VBlanks)
    // First VBlank
    prg_rom[pc] = 0xA2; pc += 1; // LDX #$00
    prg_rom[pc] = 0x00; pc += 1;
    
    // Wait for VBlank (bit 7 of $2002)
    let vblank_wait1 = pc;
    prg_rom[pc] = 0xAD; pc += 1; // LDA $2002
    prg_rom[pc] = 0x02; pc += 1;
    prg_rom[pc] = 0x20; pc += 1;
    prg_rom[pc] = 0x10; pc += 1; // BPL (branch if bit 7 = 0)
    prg_rom[pc] = ((vblank_wait1 as i8) - (pc as i8) - 1) as u8; pc += 1;
    
    // Second VBlank
    let vblank_wait2 = pc;
    prg_rom[pc] = 0xAD; pc += 1; // LDA $2002
    prg_rom[pc] = 0x02; pc += 1;
    prg_rom[pc] = 0x20; pc += 1;
    prg_rom[pc] = 0x10; pc += 1; // BPL
    prg_rom[pc] = ((vblank_wait2 as i8) - (pc as i8) - 1) as u8; pc += 1;
    
    // Load palette
    // Set PPU address to $3F00 (palette RAM)
    prg_rom[pc] = 0xAD; pc += 1; // LDA $2002 (reset address latch)
    prg_rom[pc] = 0x02; pc += 1;
    prg_rom[pc] = 0x20; pc += 1;
    
    prg_rom[pc] = 0xA9; pc += 1; // LDA #$3F
    prg_rom[pc] = 0x3F; pc += 1;
    prg_rom[pc] = 0x8D; pc += 1; // STA $2006
    prg_rom[pc] = 0x06; pc += 1;
    prg_rom[pc] = 0x20; pc += 1;
    
    prg_rom[pc] = 0xA9; pc += 1; // LDA #$00
    prg_rom[pc] = 0x00; pc += 1;
    prg_rom[pc] = 0x8D; pc += 1; // STA $2006
    prg_rom[pc] = 0x06; pc += 1;
    prg_rom[pc] = 0x20; pc += 1;
    
    // Write palette colors (32 bytes)
    // Background palette 0: Black, Dark Blue, Light Blue, White
    let palette_colors = [
        0x0F, 0x01, 0x11, 0x30, // BG palette 0
        0x0F, 0x06, 0x16, 0x26, // BG palette 1 (reds)
        0x0F, 0x09, 0x19, 0x29, // BG palette 2 (greens)
        0x0F, 0x02, 0x12, 0x22, // BG palette 3 (purples)
        0x0F, 0x00, 0x10, 0x30, // Sprite palette 0
        0x0F, 0x00, 0x10, 0x30, // Sprite palette 1
        0x0F, 0x00, 0x10, 0x30, // Sprite palette 2
        0x0F, 0x00, 0x10, 0x30, // Sprite palette 3
    ];
    
    for color in &palette_colors {
        prg_rom[pc] = 0xA9; pc += 1; // LDA #color
        prg_rom[pc] = *color; pc += 1;
        prg_rom[pc] = 0x8D; pc += 1; // STA $2007
        prg_rom[pc] = 0x07; pc += 1;
        prg_rom[pc] = 0x20; pc += 1;
    }
    
    // Fill nametable with pattern
    // Set PPU address to $2000 (nametable 0)
    prg_rom[pc] = 0xAD; pc += 1; // LDA $2002 (reset address latch)
    prg_rom[pc] = 0x02; pc += 1;
    prg_rom[pc] = 0x20; pc += 1;
    
    prg_rom[pc] = 0xA9; pc += 1; // LDA #$20
    prg_rom[pc] = 0x20; pc += 1;
    prg_rom[pc] = 0x8D; pc += 1; // STA $2006
    prg_rom[pc] = 0x06; pc += 1;
    prg_rom[pc] = 0x20; pc += 1;
    
    prg_rom[pc] = 0xA9; pc += 1; // LDA #$00
    prg_rom[pc] = 0x00; pc += 1;
    prg_rom[pc] = 0x8D; pc += 1; // STA $2006
    prg_rom[pc] = 0x06; pc += 1;
    prg_rom[pc] = 0x20; pc += 1;
    
    // Fill with tile pattern (32x30 tiles = 960 tiles)
    // Use X and Y registers as counters
    prg_rom[pc] = 0xA2; pc += 1; // LDX #$00 (outer loop counter)
    prg_rom[pc] = 0x00; pc += 1;
    
    let outer_loop = pc;
    prg_rom[pc] = 0xA0; pc += 1; // LDY #$00 (inner loop counter)
    prg_rom[pc] = 0x00; pc += 1;
    
    let inner_loop = pc;
    // Write tile number based on position
    prg_rom[pc] = 0x8A; pc += 1; // TXA (use X as tile number)
    prg_rom[pc] = 0x29; pc += 1; // AND #$07 (mod 8 for variety)
    prg_rom[pc] = 0x07; pc += 1;
    prg_rom[pc] = 0x8D; pc += 1; // STA $2007
    prg_rom[pc] = 0x07; pc += 1;
    prg_rom[pc] = 0x20; pc += 1;
    
    prg_rom[pc] = 0xC8; pc += 1; // INY
    prg_rom[pc] = 0xC0; pc += 1; // CPY #$20 (32 tiles per row)
    prg_rom[pc] = 0x20; pc += 1;
    prg_rom[pc] = 0xD0; pc += 1; // BNE inner_loop
    prg_rom[pc] = ((inner_loop as i8) - (pc as i8) - 1) as u8; pc += 1;
    
    prg_rom[pc] = 0xE8; pc += 1; // INX
    prg_rom[pc] = 0xE0; pc += 1; // CPX #$1E (30 rows)
    prg_rom[pc] = 0x1E; pc += 1;
    prg_rom[pc] = 0xD0; pc += 1; // BNE outer_loop
    prg_rom[pc] = ((outer_loop as i8) - (pc as i8) - 1) as u8; pc += 1;
    
    // Set attribute table (colors for each 2x2 tile region)
    // Set PPU address to $23C0 (attribute table for nametable 0)
    prg_rom[pc] = 0xAD; pc += 1; // LDA $2002 (reset address latch)
    prg_rom[pc] = 0x02; pc += 1;
    prg_rom[pc] = 0x20; pc += 1;
    
    prg_rom[pc] = 0xA9; pc += 1; // LDA #$23
    prg_rom[pc] = 0x23; pc += 1;
    prg_rom[pc] = 0x8D; pc += 1; // STA $2006
    prg_rom[pc] = 0x06; pc += 1;
    prg_rom[pc] = 0x20; pc += 1;
    
    prg_rom[pc] = 0xA9; pc += 1; // LDA #$C0
    prg_rom[pc] = 0xC0; pc += 1;
    prg_rom[pc] = 0x8D; pc += 1; // STA $2006
    prg_rom[pc] = 0x06; pc += 1;
    prg_rom[pc] = 0x20; pc += 1;
    
    // Fill attribute table with pattern
    prg_rom[pc] = 0xA2; pc += 1; // LDX #$00
    prg_rom[pc] = 0x00; pc += 1;
    
    let attr_loop = pc;
    prg_rom[pc] = 0x8A; pc += 1; // TXA
    prg_rom[pc] = 0x29; pc += 1; // AND #$03 (cycle through palettes)
    prg_rom[pc] = 0x03; pc += 1;
    prg_rom[pc] = 0x8D; pc += 1; // STA $2007
    prg_rom[pc] = 0x07; pc += 1;
    prg_rom[pc] = 0x20; pc += 1;
    
    prg_rom[pc] = 0xE8; pc += 1; // INX
    prg_rom[pc] = 0xE0; pc += 1; // CPX #$40 (64 bytes in attribute table)
    prg_rom[pc] = 0x40; pc += 1;
    prg_rom[pc] = 0xD0; pc += 1; // BNE attr_loop
    prg_rom[pc] = ((attr_loop as i8) - (pc as i8) - 1) as u8; pc += 1;
    
    // Enable rendering
    prg_rom[pc] = 0xA9; pc += 1; // LDA #$90 (NMI enable, use pattern table 0)
    prg_rom[pc] = 0x90; pc += 1;
    prg_rom[pc] = 0x8D; pc += 1; // STA $2000 (PPUCTRL)
    prg_rom[pc] = 0x00; pc += 1;
    prg_rom[pc] = 0x20; pc += 1;
    
    prg_rom[pc] = 0xA9; pc += 1; // LDA #$1E (show background and sprites)
    prg_rom[pc] = 0x1E; pc += 1;
    prg_rom[pc] = 0x8D; pc += 1; // STA $2001 (PPUMASK)
    prg_rom[pc] = 0x01; pc += 1;
    prg_rom[pc] = 0x20; pc += 1;
    
    // Infinite loop
    let infinite_loop = pc;
    prg_rom[pc] = 0x4C; pc += 1; // JMP infinite_loop
    prg_rom[pc] = ((infinite_loop & 0xFF) as u8); pc += 1;
    prg_rom[pc] = ((infinite_loop >> 8) as u8 | 0x80); pc += 1;
    
    // NMI handler (does nothing)
    let nmi_handler = 0x3F00; // Offset in PRG-ROM
    prg_rom[nmi_handler] = 0x40; // RTI
    
    // Set vectors at end of PRG-ROM
    // NMI vector ($FFFA-$FFFB)
    prg_rom[0x3FFA] = 0x00;
    prg_rom[0x3FFB] = 0xBF; // Points to $BF00
    
    // Reset vector ($FFFC-$FFFD)
    prg_rom[0x3FFC] = 0x00;
    prg_rom[0x3FFD] = 0x80; // Points to $8000
    
    // IRQ vector ($FFFE-$FFFF)
    prg_rom[0x3FFE] = 0x00;
    prg_rom[0x3FFF] = 0xBF; // Points to $BF00
}
