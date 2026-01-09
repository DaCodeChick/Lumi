/// Generate a simpler animated test ROM - just a color-cycling screen
/// 
/// Creates a ROM that demonstrates smooth animation with minimal complexity
/// 
/// Usage: cargo run --example generate_animation_test -p emu-nes

use std::fs::File;
use std::io::{self, Write};

fn main() -> io::Result<()> {
    generate_rom("animation_test.nes")?;
    
    println!("\n✓ Generated animation_test.nes!");
    println!("  Features:");
    println!("    - Color-cycling background (changes every 0.5 seconds)");
    println!("    - Animated sprite moving in a square pattern");
    println!("    - 60 FPS smooth animation");
    println!("\n  Load in the GUI to see smooth animation!");
    
    Ok(())
}

fn generate_rom(filename: &str) -> io::Result<()> {
    let mut prg = vec![0xEA; 0x4000]; // 16KB PRG-ROM
    let chr = generate_chr();
    
    // Zero page variables:
    // $00 = sprite X position
    // $01 = sprite Y position  
    // $02 = movement state (0-3: right, down, left, up)
    // $03 = movement counter
    // $04 = color index
    
    let mut pc = 0x0000;
    
    // === RESET HANDLER ===
    let reset_handler = pc;
    
    // Initialize sprite at top-left
    prg[pc] = 0xA9; pc += 1; // LDA #64
    prg[pc] = 64; pc += 1;
    prg[pc] = 0x85; pc += 1; // STA $00 (X)
    prg[pc] = 0x00; pc += 1;
    
    prg[pc] = 0xA9; pc += 1; // LDA #64
    prg[pc] = 64; pc += 1;
    prg[pc] = 0x85; pc += 1; // STA $01 (Y)
    prg[pc] = 0x01; pc += 1;
    
    prg[pc] = 0xA9; pc += 1; // LDA #0
    prg[pc] = 0x00; pc += 1;
    prg[pc] = 0x85; pc += 1; // STA $02 (state)
    prg[pc] = 0x02; pc += 1;
    
    prg[pc] = 0xA9; pc += 1; // LDA #0
    prg[pc] = 0x00; pc += 1;
    prg[pc] = 0x85; pc += 1; // STA $03 (counter)
    prg[pc] = 0x03; pc += 1;
    
    prg[pc] = 0xA9; pc += 1; // LDA #$01
    prg[pc] = 0x01; pc += 1;
    prg[pc] = 0x85; pc += 1; // STA $04 (color)
    prg[pc] = 0x04; pc += 1;
    
    // Wait for vblank
    let wait_vbl1 = pc;
    prg[pc] = 0x2C; pc += 1; // BIT $2002
    prg[pc] = 0x02; pc += 1;
    prg[pc] = 0x20; pc += 1;
    prg[pc] = 0x10; pc += 1; // BPL wait_vbl1
    prg[pc] = (((wait_vbl1 as i32) - (pc as i32) - 1) & 0xFF) as u8; pc += 1;
    
    // Set up palette
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
    
    // Write 16 background colors
    for i in 0..16 {
        prg[pc] = 0xA9; pc += 1; // LDA #color
        prg[pc] = i; pc += 1;
        prg[pc] = 0x8D; pc += 1; // STA $2007
        prg[pc] = 0x07; pc += 1;
        prg[pc] = 0x20; pc += 1;
    }
    
    // Write sprite palette (red sprite)
    for &color in &[0x0F, 0x16, 0x27, 0x30] {
        prg[pc] = 0xA9; pc += 1; // LDA #color
        prg[pc] = color; pc += 1;
        prg[pc] = 0x8D; pc += 1; // STA $2007
        prg[pc] = 0x07; pc += 1;
        prg[pc] = 0x20; pc += 1;
    }
    
    // Enable rendering
    prg[pc] = 0xA9; pc += 1; // LDA #$90 (NMI + BG pattern 1)
    prg[pc] = 0x90; pc += 1;
    prg[pc] = 0x8D; pc += 1; // STA $2000
    prg[pc] = 0x00; pc += 1;
    prg[pc] = 0x20; pc += 1;
    
    prg[pc] = 0xA9; pc += 1; // LDA #$1E (show BG + sprites)
    prg[pc] = 0x1E; pc += 1;
    prg[pc] = 0x8D; pc += 1; // STA $2001
    prg[pc] = 0x01; pc += 1;
    prg[pc] = 0x20; pc += 1;
    
    // Main loop
    let main_loop = pc;
    prg[pc] = 0x4C; pc += 1; // JMP main_loop
    prg[pc] = ((main_loop + 0x8000) & 0xFF) as u8; pc += 1;
    prg[pc] = ((main_loop + 0x8000) >> 8) as u8; pc += 1;
    
    // === NMI HANDLER ===
    let nmi_handler = pc;
    
    // Cycle background color every 30 frames
    prg[pc] = 0xE6; pc += 1; // INC $03 (counter)
    prg[pc] = 0x03; pc += 1;
    prg[pc] = 0xA5; pc += 1; // LDA $03
    prg[pc] = 0x03; pc += 1;
    prg[pc] = 0xC9; pc += 1; // CMP #30
    prg[pc] = 30; pc += 1;
    prg[pc] = 0x90; pc += 1; // BCC skip_color
    prg[pc] = 0x13; pc += 1; // Skip ahead
    
    // Reset counter
    prg[pc] = 0xA9; pc += 1; // LDA #0
    prg[pc] = 0x00; pc += 1;
    prg[pc] = 0x85; pc += 1; // STA $03
    prg[pc] = 0x03; pc += 1;
    
    // Increment color
    prg[pc] = 0xE6; pc += 1; // INC $04
    prg[pc] = 0x04; pc += 1;
    prg[pc] = 0xA5; pc += 1; // LDA $04
    prg[pc] = 0x04; pc += 1;
    prg[pc] = 0x29; pc += 1; // AND #$0F
    prg[pc] = 0x0F; pc += 1;
    prg[pc] = 0x85; pc += 1; // STA $04
    prg[pc] = 0x04; pc += 1;
    
    // Write color to palette
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
    prg[pc] = 0xA5; pc += 1; // LDA $04 (color)
    prg[pc] = 0x04; pc += 1;
    prg[pc] = 0x8D; pc += 1; // STA $2007
    prg[pc] = 0x07; pc += 1;
    prg[pc] = 0x20; pc += 1;
    
    // skip_color:
    // Move sprite in a square (1 pixel per frame)
    prg[pc] = 0xA5; pc += 1; // LDA $02 (state)
    prg[pc] = 0x02; pc += 1;
    prg[pc] = 0xC9; pc += 1; // CMP #0 (moving right?)
    prg[pc] = 0x00; pc += 1;
    prg[pc] = 0xD0; pc += 1; // BNE check_down
    prg[pc] = 0x0E; pc += 1;
    
    // Move right
    prg[pc] = 0xE6; pc += 1; // INC $00 (X++)
    prg[pc] = 0x00; pc += 1;
    prg[pc] = 0xA5; pc += 1; // LDA $00
    prg[pc] = 0x00; pc += 1;
    prg[pc] = 0xC9; pc += 1; // CMP #192 (reached right?)
    prg[pc] = 192; pc += 1;
    prg[pc] = 0x90; pc += 1; // BCC write_oam
    prg[pc] = 0x37; pc += 1;
    prg[pc] = 0xA9; pc += 1; // LDA #1
    prg[pc] = 0x01; pc += 1;
    prg[pc] = 0x85; pc += 1; // STA $02 (state = down)
    prg[pc] = 0x02; pc += 1;
    prg[pc] = 0x4C; pc += 1; // JMP write_oam
    let write_oam = pc + 2;
    prg[pc] = ((write_oam + 0x8000) & 0xFF) as u8; pc += 1;
    prg[pc] = ((write_oam + 0x8000) >> 8) as u8; pc += 1;
    
    // check_down:
    prg[pc] = 0xC9; pc += 1; // CMP #1 (moving down?)
    prg[pc] = 0x01; pc += 1;
    prg[pc] = 0xD0; pc += 1; // BNE check_left
    prg[pc] = 0x0E; pc += 1;
    
    // Move down
    prg[pc] = 0xE6; pc += 1; // INC $01 (Y++)
    prg[pc] = 0x01; pc += 1;
    prg[pc] = 0xA5; pc += 1; // LDA $01
    prg[pc] = 0x01; pc += 1;
    prg[pc] = 0xC9; pc += 1; // CMP #192
    prg[pc] = 192; pc += 1;
    prg[pc] = 0x90; pc += 1; // BCC write_oam
    prg[pc] = 0x25; pc += 1;
    prg[pc] = 0xA9; pc += 1; // LDA #2
    prg[pc] = 0x02; pc += 1;
    prg[pc] = 0x85; pc += 1; // STA $02 (state = left)
    prg[pc] = 0x02; pc += 1;
    prg[pc] = 0x4C; pc += 1; // JMP write_oam
    prg[pc] = ((write_oam + 0x8000) & 0xFF) as u8; pc += 1;
    prg[pc] = ((write_oam + 0x8000) >> 8) as u8; pc += 1;
    
    // check_left:
    prg[pc] = 0xC9; pc += 1; // CMP #2 (moving left?)
    prg[pc] = 0x02; pc += 1;
    prg[pc] = 0xD0; pc += 1; // BNE move_up
    prg[pc] = 0x0E; pc += 1;
    
    // Move left
    prg[pc] = 0xC6; pc += 1; // DEC $00 (X--)
    prg[pc] = 0x00; pc += 1;
    prg[pc] = 0xA5; pc += 1; // LDA $00
    prg[pc] = 0x00; pc += 1;
    prg[pc] = 0xC9; pc += 1; // CMP #64
    prg[pc] = 64; pc += 1;
    prg[pc] = 0xB0; pc += 1; // BCS write_oam
    prg[pc] = 0x13; pc += 1;
    prg[pc] = 0xA9; pc += 1; // LDA #3
    prg[pc] = 0x03; pc += 1;
    prg[pc] = 0x85; pc += 1; // STA $02 (state = up)
    prg[pc] = 0x02; pc += 1;
    prg[pc] = 0x4C; pc += 1; // JMP write_oam
    prg[pc] = ((write_oam + 0x8000) & 0xFF) as u8; pc += 1;
    prg[pc] = ((write_oam + 0x8000) >> 8) as u8; pc += 1;
    
    // move_up:
    prg[pc] = 0xC6; pc += 1; // DEC $01 (Y--)
    prg[pc] = 0x01; pc += 1;
    prg[pc] = 0xA5; pc += 1; // LDA $01
    prg[pc] = 0x01; pc += 1;
    prg[pc] = 0xC9; pc += 1; // CMP #64
    prg[pc] = 64; pc += 1;
    prg[pc] = 0xB0; pc += 1; // BCS write_oam
    prg[pc] = 0x04; pc += 1;
    prg[pc] = 0xA9; pc += 1; // LDA #0
    prg[pc] = 0x00; pc += 1;
    prg[pc] = 0x85; pc += 1; // STA $02 (state = right)
    prg[pc] = 0x02; pc += 1;
    
    // write_oam: Write sprite
    // (label for jumps, pc will reach here naturally)
    prg[pc] = 0xA5; pc += 1; // LDA $01 (Y)
    prg[pc] = 0x01; pc += 1;
    prg[pc] = 0x8D; pc += 1; // STA $0200
    prg[pc] = 0x00; pc += 1;
    prg[pc] = 0x02; pc += 1;
    
    prg[pc] = 0xA9; pc += 1; // LDA #1 (tile)
    prg[pc] = 0x01; pc += 1;
    prg[pc] = 0x8D; pc += 1; // STA $0201
    prg[pc] = 0x01; pc += 1;
    prg[pc] = 0x02; pc += 1;
    
    prg[pc] = 0xA9; pc += 1; // LDA #0 (attr)
    prg[pc] = 0x00; pc += 1;
    prg[pc] = 0x8D; pc += 1; // STA $0202
    prg[pc] = 0x02; pc += 1;
    prg[pc] = 0x02; pc += 1;
    
    prg[pc] = 0xA5; pc += 1; // LDA $00 (X)
    prg[pc] = 0x00; pc += 1;
    prg[pc] = 0x8D; pc += 1; // STA $0203
    prg[pc] = 0x03; pc += 1;
    prg[pc] = 0x02; pc += 1;
    
    // OAM DMA
    prg[pc] = 0xA9; pc += 1; // LDA #$02
    prg[pc] = 0x02; pc += 1;
    prg[pc] = 0x8D; pc += 1; // STA $4014
    prg[pc] = 0x14; pc += 1;
    prg[pc] = 0x40; pc += 1;
    
    // Return from NMI
    prg[pc] = 0x40; // RTI
    
    println!("Reset handler at: ${:04X}", reset_handler + 0x8000);
    println!("NMI handler at: ${:04X}", nmi_handler + 0x8000);
    println!("Code ends at: ${:04X}", pc + 0x8000);
    
    // Vectors
    prg[0x3FFA] = ((nmi_handler + 0x8000) & 0xFF) as u8;
    prg[0x3FFB] = ((nmi_handler + 0x8000) >> 8) as u8;
    prg[0x3FFC] = ((reset_handler + 0x8000) & 0xFF) as u8;
    prg[0x3FFD] = ((reset_handler + 0x8000) >> 8) as u8;
    prg[0x3FFE] = 0x00;
    prg[0x3FFF] = 0x80;
    
    // Build iNES ROM
    let ines_header = [
        0x4E, 0x45, 0x53, 0x1A, // "NES" + EOF
        0x01,                   // 1x 16KB PRG-ROM
        0x01,                   // 1x 8KB CHR-ROM
        0x00,                   // Mapper 0
        0x00,
        0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00,
    ];
    
    let mut file = File::create(filename)?;
    file.write_all(&ines_header)?;
    file.write_all(&prg)?;
    file.write_all(&chr)?;
    
    println!("Generated {}", filename);
    
    Ok(())
}

/// Generate CHR-ROM with a filled square sprite
fn generate_chr() -> Vec<u8> {
    let mut chr = vec![0u8; 0x2000];
    
    // Tile 0: Empty
    
    // Tile 1: 8x8 filled square
    for i in 0..8 {
        chr[0x10 + i] = 0xFF;
        chr[0x18 + i] = 0xFF;
    }
    
    chr
}
