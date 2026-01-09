/// Generate a controller test ROM
/// 
/// This ROM reads controller input and stores button states in memory:
/// - Addresses $00-$07: Button states (A, B, Select, Start, Up, Down, Left, Right)
/// - Address $10: Success flag ($FF when complete)
/// 
/// Usage: cargo run --example generate_controller_test -p emu-nes

use std::fs::File;
use std::io::{self, Write};

fn main() -> io::Result<()> {
    let mut prg = vec![0xEA; 0x4000]; // Fill with NOPs
    
    let mut pc = 0;
    
    // Start strobe (write 1 to $4016)
    prg[pc] = 0xA9; pc += 1; // LDA #$01
    prg[pc] = 0x01; pc += 1;
    prg[pc] = 0x8D; pc += 1; // STA $4016
    prg[pc] = 0x16; pc += 1;
    prg[pc] = 0x40; pc += 1;
    
    // End strobe (write 0 to $4016 to latch button states)
    prg[pc] = 0xA9; pc += 1; // LDA #$00
    prg[pc] = 0x00; pc += 1;
    prg[pc] = 0x8D; pc += 1; // STA $4016
    prg[pc] = 0x16; pc += 1;
    prg[pc] = 0x40; pc += 1;
    
    // Initialize loop counter (8 buttons)
    prg[pc] = 0xA2; pc += 1; // LDX #$00
    prg[pc] = 0x00; pc += 1;
    
    // Loop to read 8 buttons
    let loop_start = pc;
    prg[pc] = 0xAD; pc += 1; // LDA $4016 (read one button bit)
    prg[pc] = 0x16; pc += 1;
    prg[pc] = 0x40; pc += 1;
    
    prg[pc] = 0x29; pc += 1; // AND #$01 (mask to bit 0)
    prg[pc] = 0x01; pc += 1;
    
    prg[pc] = 0x95; pc += 1; // STA $00,X (store in $00-$07)
    prg[pc] = 0x00; pc += 1;
    
    prg[pc] = 0xE8; pc += 1; // INX (increment counter)
    
    prg[pc] = 0xE0; pc += 1; // CPX #$08 (compare with 8)
    prg[pc] = 0x08; pc += 1;
    
    prg[pc] = 0xD0; pc += 1; // BNE loop_start
    let offset = (loop_start as i16) - (pc as i16) - 1;
    prg[pc] = offset as u8; pc += 1;
    
    // Set success flag at $10
    prg[pc] = 0xA9; pc += 1; // LDA #$FF
    prg[pc] = 0xFF; pc += 1;
    prg[pc] = 0x85; pc += 1; // STA $10
    prg[pc] = 0x10; pc += 1;
    
    // Infinite loop
    let hang = pc;
    prg[pc] = 0x4C; pc += 1; // JMP hang
    prg[pc] = ((hang + 0x8000) & 0xFF) as u8; pc += 1;
    prg[pc] = ((hang + 0x8000) >> 8) as u8; pc += 1;
    
    // Reset vector points to $8000
    prg[0x3FFC] = 0x00;
    prg[0x3FFD] = 0x80;
    
    // Create minimal CHR-ROM (8KB, all zeros - we're not using graphics)
    let chr = vec![0u8; 0x2000];
    
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
    
    let mut file = File::create("controller_test.nes")?;
    file.write_all(&ines_header)?;
    file.write_all(&prg)?;
    file.write_all(&chr)?;
    
    println!("Generated controller_test.nes");
    println!();
    println!("This ROM reads controller input and stores button states in memory:");
    println!("  $00 = A button");
    println!("  $01 = B button");
    println!("  $02 = Select button");
    println!("  $03 = Start button");
    println!("  $04 = Up button");
    println!("  $05 = Down button");
    println!("  $06 = Left button");
    println!("  $07 = Right button");
    println!("  $10 = Success flag ($FF when complete)");
    
    Ok(())
}
