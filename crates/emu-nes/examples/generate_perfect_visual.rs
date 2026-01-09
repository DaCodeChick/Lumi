/// Generate a PERFECT visual test ROM - no bugs this time!

use std::fs::File;
use std::io::{self, Write};

fn main() -> io::Result<()> {
    let mut prg = vec![0xEA; 0x4000]; // Fill with NOPs
    let chr = generate_chr();
    
    let mut pc = 0;
    
    // Wait for VBlank
    prg[pc] = 0x2C; pc += 1; // BIT $2002
    prg[pc] = 0x02; pc += 1;
    prg[pc] = 0x20; pc += 1;
    prg[pc] = 0x10; pc += 1; // BPL
    prg[pc] = 0xFB; pc += 1; // -5
    
    // Load palette
    prg[pc] = 0x2C; pc += 1; // BIT $2002
    prg[pc] = 0x02; pc += 1;
    prg[pc] = 0x20; pc += 1;
    prg[pc] = 0xA9; pc += 1; // LDA #$3F
    prg[pc] = 0x3F; pc += 1;
    prg[pc] = 0x8D; pc += 1; // STA $2006
    prg[pc] = 0x06; pc += 1;
    prg[pc] = 0x20; pc += 1;
    prg[pc] = 0xA9; pc += 1; // LDA #$00
    prg[pc] = 0x00; pc += 1;
    prg[pc] = 0x8D; pc += 1; // STA $2006
    prg[pc] = 0x06; pc += 1;
    prg[pc] = 0x20; pc += 1;
    
    // Write palette: Black, Red, Green, Blue
    for &color in &[0x0F, 0x16, 0x1A, 0x12] {
        prg[pc] = 0xA9; pc += 1; // LDA #color
        prg[pc] = color; pc += 1;
        prg[pc] = 0x8D; pc += 1; // STA $2007
        prg[pc] = 0x07; pc += 1;
        prg[pc] = 0x20; pc += 1;
    }
    
    // Write a simple nametable pattern
    // Set address to $2000
    prg[pc] = 0x2C; pc += 1; // BIT $2002
    prg[pc] = 0x02; pc += 1;
    prg[pc] = 0x20; pc += 1;
    prg[pc] = 0xA9; pc += 1; // LDA #$20
    prg[pc] = 0x20; pc += 1;
    prg[pc] = 0x8D; pc += 1; // STA $2006
    prg[pc] = 0x06; pc += 1;
    prg[pc] = 0x20; pc += 1;
    prg[pc] = 0xA9; pc += 1; // LDA #$00
    prg[pc] = 0x00; pc += 1;
    prg[pc] = 0x8D; pc += 1; // STA $2006
    prg[pc] = 0x06; pc += 1;
    prg[pc] = 0x20; pc += 1;
    
    // Fill first 4 rows (128 tiles) with pattern 1,2,3,1,2,3...
    // Using simpler code
    prg[pc] = 0xA9; pc += 1; // LDA #$01
    prg[pc] = 0x01; pc += 1;
    prg[pc] = 0xA2; pc += 1; // LDX #$80 (128 tiles)
    prg[pc] = 0x80; pc += 1;
    
    let loop_addr = pc;
    prg[pc] = 0x8D; pc += 1; // STA $2007 (write A to VRAM)
    prg[pc] = 0x07; pc += 1;
    prg[pc] = 0x20; pc += 1;
    
    prg[pc] = 0xC9; pc += 1; // CMP #$03
    prg[pc] = 0x03; pc += 1;
    prg[pc] = 0xD0; pc += 1; // BNE +3
    prg[pc] = 0x02; pc += 1;
    prg[pc] = 0xA9; pc += 1; // LDA #$00
    prg[pc] = 0x00; pc += 1;
    
    prg[pc] = 0x18; pc += 1; // CLC
    prg[pc] = 0x69; pc += 1; // ADC #$01
    prg[pc] = 0x01; pc += 1;
    
    prg[pc] = 0xCA; pc += 1; // DEX
    prg[pc] = 0xD0; pc += 1; // BNE loop_addr
    let offset = (loop_addr as i16) - (pc as i16) - 1;
    prg[pc] = offset as u8; pc += 1;
    
    // Enable rendering
    prg[pc] = 0xA9; pc += 1; // LDA #$80
    prg[pc] = 0x80; pc += 1;
    prg[pc] = 0x8D; pc += 1; // STA $2000
    prg[pc] = 0x00; pc += 1;
    prg[pc] = 0x20; pc += 1;
    prg[pc] = 0xA9; pc += 1; // LDA #$1E
    prg[pc] = 0x1E; pc += 1;
    prg[pc] = 0x8D; pc += 1; // STA $2001
    prg[pc] = 0x01; pc += 1;
    prg[pc] = 0x20; pc += 1;
    
    // Infinite loop
    let hang = pc;
    prg[pc] = 0x4C; pc += 1;
    prg[pc] = ((hang + 0x8000) & 0xFF) as u8; pc += 1;
    prg[pc] = ((hang + 0x8000) >> 8) as u8; pc += 1;
    
    // Vectors
    prg[0x3FFC] = 0x00;
    prg[0x3FFD] = 0x80;
    
    // Write ROM
    let header = vec![0x4E, 0x45, 0x53, 0x1A, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let mut file = File::create("perfect_visual.nes")?;
    file.write_all(&header)?;
    file.write_all(&prg)?;
    file.write_all(&chr)?;
    
    println!("Generated: perfect_visual.nes");
    Ok(())
}

fn generate_chr() -> Vec<u8> {
    let mut chr = vec![0; 0x2000];
    
    // Tile 1: Solid (all pixel_value = 3)
    for i in 0..16 {
        chr[16 + i] = 0xFF;
    }
    
    // Tile 2: Checkerboard
    for i in 0..8 {
        chr[32 + i] = 0xAA;
        chr[32 + i + 8] = 0x55;
    }
    
    // Tile 3: Horizontal stripes
    for i in 0..8 {
        if i % 2 == 0 {
            chr[48 + i] = 0xFF;
            chr[48 + i + 8] = 0xFF;
        }
    }
    
    chr
}
