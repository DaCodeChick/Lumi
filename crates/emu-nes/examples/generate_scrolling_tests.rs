/// Generate scrolling test ROMs - both scrolled and non-scrolled versions
/// 
/// Creates two ROMs to demonstrate scrolling:
/// - scrolling_test_scroll.nes: With X=64, Y=32 scroll
/// - scrolling_test_noscroll.nes: With scroll=0
/// 
/// Usage: cargo run --example generate_scrolling_tests -p emu-nes

use std::fs::File;
use std::io::{self, Write};

fn main() -> io::Result<()> {
    // Generate both versions
    generate_rom("scrolling_test_scroll.nes", 64, 32)?;
    generate_rom("scrolling_test_noscroll.nes", 0, 0)?;
    
    println!("\nGenerated both test ROMs!");
    println!("  - scrolling_test_scroll.nes (scroll X=64, Y=32)");
    println!("  - scrolling_test_noscroll.nes (scroll X=0, Y=0)");
    println!("\nRun scrolling_compare demo to see the difference!");
    
    Ok(())
}

fn generate_rom(filename: &str, scroll_x: u8, scroll_y: u8) -> io::Result<()> {
    let mut prg = vec![0xEA; 0x4000]; // Fill with NOPs
    let chr = generate_chr();
    
    let mut pc = 0;
    
    // Skip initial vblank wait - not needed for this test
    
    // Load palette
    prg[pc] = 0x2C; pc += 1; // BIT $2002 (reset address latch)
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
    
    // Write palette: Black, Red, Green, Blue, White, Yellow
    for &color in &[0x0F, 0x16, 0x1A, 0x12, 0x30, 0x28] {
        prg[pc] = 0xA9; pc += 1; // LDA #color
        prg[pc] = color; pc += 1;
        prg[pc] = 0x8D; pc += 1; // STA $2007
        prg[pc] = 0x07; pc += 1;
        prg[pc] = 0x20; pc += 1;
    }
    
    // Write a checkerboard pattern to nametable
    // Set address to $2000
    prg[pc] = 0x2C; pc += 1; // BIT $2002 (reset address latch)
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
    
    // Fill nametable with pattern: creates horizontal bands
    prg[pc] = 0xA2; pc += 1; // LDX #$00 (row counter)
    prg[pc] = 0x00; pc += 1;
    
    let outer_loop = pc;
    // Calculate which tile to use: tile = (row / 4) + 1
    prg[pc] = 0x8A; pc += 1; // TXA (A = row number)
    prg[pc] = 0x4A; pc += 1; // LSR A (divide by 2)
    prg[pc] = 0x4A; pc += 1; // LSR A (divide by 4)
    prg[pc] = 0x18; pc += 1; // CLC
    prg[pc] = 0x69; pc += 1; // ADC #$01
    prg[pc] = 0x01; pc += 1;
    prg[pc] = 0x29; pc += 1; // AND #$03 (wrap to tiles 1-4)
    prg[pc] = 0x03; pc += 1;
    prg[pc] = 0x18; pc += 1; // CLC
    prg[pc] = 0x69; pc += 1; // ADC #$01
    prg[pc] = 0x01; pc += 1;
    
    // Now A has the tile number for this row
    prg[pc] = 0xA0; pc += 1; // LDY #$00 (column counter)
    prg[pc] = 0x00; pc += 1;
    
    let inner_loop = pc;
    prg[pc] = 0x8D; pc += 1; // STA $2007 (write tile)
    prg[pc] = 0x07; pc += 1;
    prg[pc] = 0x20; pc += 1;
    
    prg[pc] = 0xC8; pc += 1; // INY
    prg[pc] = 0xC0; pc += 1; // CPY #32
    prg[pc] = 32; pc += 1;
    prg[pc] = 0xD0; pc += 1; // BNE inner_loop
    let offset = (inner_loop as i16) - (pc as i16) - 1;
    prg[pc] = offset as u8; pc += 1;
    
    prg[pc] = 0xE8; pc += 1; // INX
    prg[pc] = 0xE0; pc += 1; // CPX #30 (30 rows)
    prg[pc] = 30; pc += 1;
    prg[pc] = 0xD0; pc += 1; // BNE outer_loop
    let offset = (outer_loop as i16) - (pc as i16) - 1;
    prg[pc] = offset as u8; pc += 1;
    
    // Set scroll position
    prg[pc] = 0x2C; pc += 1; // BIT $2002 (reset address latch)
    prg[pc] = 0x02; pc += 1;
    prg[pc] = 0x20; pc += 1;
    prg[pc] = 0xA9; pc += 1; // LDA #scroll_x
    prg[pc] = scroll_x; pc += 1;
    prg[pc] = 0x8D; pc += 1; // STA $2005 (X scroll)
    prg[pc] = 0x05; pc += 1;
    prg[pc] = 0x20; pc += 1;
    prg[pc] = 0xA9; pc += 1; // LDA #scroll_y
    prg[pc] = scroll_y; pc += 1;
    prg[pc] = 0x8D; pc += 1; // STA $2005 (Y scroll)
    prg[pc] = 0x05; pc += 1;
    prg[pc] = 0x20; pc += 1;
    
    // Enable rendering
    prg[pc] = 0xA9; pc += 1; // LDA #$80
    prg[pc] = 0x80; pc += 1;
    prg[pc] = 0x8D; pc += 1; // STA $2000 (PPUCTRL)
    prg[pc] = 0x00; pc += 1;
    prg[pc] = 0x20; pc += 1;
    prg[pc] = 0xA9; pc += 1; // LDA #$1E
    prg[pc] = 0x1E; pc += 1;
    prg[pc] = 0x8D; pc += 1; // STA $2001 (PPUMASK - enable BG and sprites)
    prg[pc] = 0x01; pc += 1;
    prg[pc] = 0x20; pc += 1;
    
    // Infinite loop
    let hang = pc;
    prg[pc] = 0x4C; pc += 1; // JMP hang
    prg[pc] = ((hang + 0x8000) & 0xFF) as u8; pc += 1;
    prg[pc] = ((hang + 0x8000) >> 8) as u8;
    
    // Reset vector points to $8000
    prg[0x3FFC] = 0x00;
    prg[0x3FFD] = 0x80;
    
    // Build iNES ROM
    let ines_header = [
        0x4E, 0x45, 0x53, 0x1A, // "NES" + EOF
        0x01,                   // 1x 16KB PRG-ROM
        0x01,                   // 1x 8KB CHR-ROM
        0x00,                   // Mapper 0, horizontal mirroring
        0x00,                   // Mapper 0 upper bits
        0x00, 0x00, 0x00, 0x00, // Reserved
        0x00, 0x00, 0x00, 0x00,
    ];
    
    let mut file = File::create(filename)?;
    file.write_all(&ines_header)?;
    file.write_all(&prg)?;
    file.write_all(&chr)?;
    
    println!("Generated {}", filename);
    
    Ok(())
}

/// Generate CHR-ROM with distinct patterns for tiles 1-4
fn generate_chr() -> Vec<u8> {
    let mut chr = vec![0u8; 0x2000];
    
    // Tile 0: Empty (already all zeros)
    
    // Tile 1: Solid fill
    for i in 0..8 {
        chr[0x10 + i] = 0xFF; // Low bitplane
        chr[0x18 + i] = 0xFF; // High bitplane
    }
    
    // Tile 2: Horizontal stripes
    for i in 0..8 {
        if i % 2 == 0 {
            chr[0x20 + i] = 0xFF;
            chr[0x28 + i] = 0xFF;
        }
    }
    
    // Tile 3: Vertical stripes
    for i in 0..8 {
        chr[0x30 + i] = 0xAA; // 10101010
        chr[0x38 + i] = 0xAA;
    }
    
    // Tile 4: Checkerboard
    for i in 0..8 {
        if i % 2 == 0 {
            chr[0x40 + i] = 0xAA; // 10101010
            chr[0x48 + i] = 0xAA;
        } else {
            chr[0x40 + i] = 0x55; // 01010101
            chr[0x48 + i] = 0x55;
        }
    }
    
    chr
}
