use std::fs::File;
use std::io::Write;

fn main() {
    let mut rom = Vec::new();
    
    // iNES Header
    rom.extend_from_slice(b"NES\x1a");
    rom.push(1);  // 1 PRG ROM bank (16KB)
    rom.push(1);  // 1 CHR ROM bank (8KB)
    rom.push(0);  // Mapper 0, horizontal mirroring
    rom.extend_from_slice(&[0; 9]); // Rest of header
    
    // PRG ROM (16KB at 0xC000-0xFFFF, mirrored to 0x8000-0xBFFF)
    let mut prg = vec![0; 16384];
    let mut pc = 0xC000;
    
    // --- RESET Handler ---
    let reset_start = pc;
    
    // Wait for PPU warmup (2 frames)
    // :vblankwait1
    let vblankwait1 = pc;
    prg[pc - 0xC000] = 0x2C; // BIT $2002
    prg[pc - 0xC000 + 1] = 0x02;
    prg[pc - 0xC000 + 2] = 0x20;
    pc += 3;
    prg[pc - 0xC000] = 0x10; // BPL vblankwait1
    prg[pc - 0xC000 + 1] = ((vblankwait1 as i32 - (pc as i32 + 2)) as i8) as u8;
    pc += 2;
    
    // :vblankwait2
    let vblankwait2 = pc;
    prg[pc - 0xC000] = 0x2C; // BIT $2002
    prg[pc - 0xC000 + 1] = 0x02;
    prg[pc - 0xC000 + 2] = 0x20;
    pc += 3;
    prg[pc - 0xC000] = 0x10; // BPL vblankwait2
    prg[pc - 0xC000 + 1] = ((vblankwait2 as i32 - (pc as i32 + 2)) as i8) as u8;
    pc += 2;
    
    // Clear OAM (sprites) - set all to $FF
    prg[pc - 0xC000] = 0xA9; // LDA #$FF
    prg[pc - 0xC000 + 1] = 0xFF;
    pc += 2;
    prg[pc - 0xC000] = 0xA2; // LDX #$00
    prg[pc - 0xC000 + 1] = 0x00;
    pc += 2;
    let clear_oam_loop = pc;
    prg[pc - 0xC000] = 0x9D; // STA $0200,X
    prg[pc - 0xC000 + 1] = 0x00;
    prg[pc - 0xC000 + 2] = 0x02;
    pc += 3;
    prg[pc - 0xC000] = 0xE8; // INX
    pc += 1;
    prg[pc - 0xC000] = 0xD0; // BNE clear_oam_loop
    prg[pc - 0xC000 + 1] = ((clear_oam_loop as i32 - (pc as i32 + 2)) as i8) as u8;
    pc += 2;
    
    // Initialize sprite at position (100, 100)
    // Sprite format: Y pos, tile #, attributes, X pos
    prg[pc - 0xC000] = 0xA9; // LDA #100
    prg[pc - 0xC000 + 1] = 100;
    pc += 2;
    prg[pc - 0xC000] = 0x85; // STA $00 (X position storage)
    prg[pc - 0xC000 + 1] = 0x00;
    pc += 2;
    
    prg[pc - 0xC000] = 0xA9; // LDA #100
    prg[pc - 0xC000 + 1] = 100;
    pc += 2;
    prg[pc - 0xC000] = 0x8D; // STA $0200 (Y pos)
    prg[pc - 0xC000 + 1] = 0x00;
    prg[pc - 0xC000 + 2] = 0x02;
    pc += 3;
    
    prg[pc - 0xC000] = 0xA9; // LDA #$01 (tile 1)
    prg[pc - 0xC000 + 1] = 0x01;
    pc += 2;
    prg[pc - 0xC000] = 0x8D; // STA $0201 (tile #)
    prg[pc - 0xC000 + 1] = 0x01;
    prg[pc - 0xC000 + 2] = 0x02;
    pc += 3;
    
    prg[pc - 0xC000] = 0xA9; // LDA #$00 (attributes)
    prg[pc - 0xC000 + 1] = 0x00;
    pc += 2;
    prg[pc - 0xC000] = 0x8D; // STA $0202 (attributes)
    prg[pc - 0xC000 + 1] = 0x02;
    prg[pc - 0xC000 + 2] = 0x02;
    pc += 3;
    
    prg[pc - 0xC000] = 0xA5; // LDA $00
    prg[pc - 0xC000 + 1] = 0x00;
    pc += 2;
    prg[pc - 0xC000] = 0x8D; // STA $0203 (X pos)
    prg[pc - 0xC000 + 1] = 0x03;
    prg[pc - 0xC000 + 2] = 0x02;
    pc += 3;
    
    // Enable NMI
    prg[pc - 0xC000] = 0xA9; // LDA #$80
    prg[pc - 0xC000 + 1] = 0x80;
    pc += 2;
    prg[pc - 0xC000] = 0x8D; // STA $2000
    prg[pc - 0xC000 + 1] = 0x00;
    prg[pc - 0xC000 + 2] = 0x20;
    pc += 3;
    
    // Enable sprites
    prg[pc - 0xC000] = 0xA9; // LDA #$10
    prg[pc - 0xC000 + 1] = 0x10;
    pc += 2;
    prg[pc - 0xC000] = 0x8D; // STA $2001
    prg[pc - 0xC000 + 1] = 0x01;
    prg[pc - 0xC000 + 2] = 0x20;
    pc += 3;
    
    // Main loop - just spin forever
    let main_loop = pc;
    prg[pc - 0xC000] = 0x4C; // JMP main_loop
    prg[pc - 0xC000 + 1] = (main_loop & 0xFF) as u8;
    prg[pc - 0xC000 + 2] = ((main_loop >> 8) & 0xFF) as u8;
    pc += 3;
    
    // --- NMI Handler ---
    let nmi_start = pc;
    
    // Save registers
    prg[pc - 0xC000] = 0x48; // PHA
    pc += 1;
    prg[pc - 0xC000] = 0x8A; // TXA
    pc += 1;
    prg[pc - 0xC000] = 0x48; // PHA
    pc += 1;
    prg[pc - 0xC000] = 0x98; // TYA
    pc += 1;
    prg[pc - 0xC000] = 0x48; // PHA
    pc += 1;
    
    // Load X position from $00
    prg[pc - 0xC000] = 0xA5; // LDA $00
    prg[pc - 0xC000 + 1] = 0x00;
    pc += 2;
    
    // Increment X position
    prg[pc - 0xC000] = 0x18; // CLC
    pc += 1;
    prg[pc - 0xC000] = 0x69; // ADC #$01
    prg[pc - 0xC000 + 1] = 0x01;
    pc += 2;
    
    // Check if >= 240 (wrap around)
    prg[pc - 0xC000] = 0xC9; // CMP #240
    prg[pc - 0xC000 + 1] = 240;
    pc += 2;
    prg[pc - 0xC000] = 0x90; // BCC no_wrap (skip next 2 instructions)
    prg[pc - 0xC000 + 1] = 0x02;
    pc += 2;
    prg[pc - 0xC000] = 0xA9; // LDA #0 (reset to 0)
    prg[pc - 0xC000 + 1] = 0x00;
    pc += 2;
    
    // no_wrap:
    // Store back to $00
    prg[pc - 0xC000] = 0x85; // STA $00
    prg[pc - 0xC000 + 1] = 0x00;
    pc += 2;
    
    // Update sprite X position in OAM
    prg[pc - 0xC000] = 0x8D; // STA $0203
    prg[pc - 0xC000 + 1] = 0x03;
    prg[pc - 0xC000 + 2] = 0x02;
    pc += 3;
    
    // DMA sprite data from $0200 to PPU
    prg[pc - 0xC000] = 0xA9; // LDA #$00
    prg[pc - 0xC000 + 1] = 0x00;
    pc += 2;
    prg[pc - 0xC000] = 0x8D; // STA $2003 (OAM address)
    prg[pc - 0xC000 + 1] = 0x03;
    prg[pc - 0xC000 + 2] = 0x20;
    pc += 3;
    prg[pc - 0xC000] = 0xA9; // LDA #$02
    prg[pc - 0xC000 + 1] = 0x02;
    pc += 2;
    prg[pc - 0xC000] = 0x8D; // STA $4014 (OAM DMA)
    prg[pc - 0xC000 + 1] = 0x14;
    prg[pc - 0xC000 + 2] = 0x40;
    pc += 3;
    
    // Restore registers
    prg[pc - 0xC000] = 0x68; // PLA
    pc += 1;
    prg[pc - 0xC000] = 0xA8; // TAY
    pc += 1;
    prg[pc - 0xC000] = 0x68; // PLA
    pc += 1;
    prg[pc - 0xC000] = 0xAA; // TAX
    pc += 1;
    prg[pc - 0xC000] = 0x68; // PLA
    pc += 1;
    
    // Return from interrupt
    prg[pc - 0xC000] = 0x40; // RTI
    pc += 1;
    
    // --- Interrupt Vectors (at 0xFFFA-0xFFFF) ---
    prg[0x3FFA] = (nmi_start & 0xFF) as u8;      // NMI vector low
    prg[0x3FFB] = ((nmi_start >> 8) & 0xFF) as u8; // NMI vector high
    prg[0x3FFC] = (reset_start & 0xFF) as u8;    // RESET vector low
    prg[0x3FFD] = ((reset_start >> 8) & 0xFF) as u8; // RESET vector high
    prg[0x3FFE] = 0x00;                           // IRQ vector (unused)
    prg[0x3FFF] = 0x00;
    
    rom.extend_from_slice(&prg);
    
    // CHR ROM (8KB)
    let mut chr = vec![0; 8192];
    
    // Define a simple filled square sprite (8x8 pixels) at tile $01
    // Each tile is 16 bytes: 8 bytes for plane 0, 8 bytes for plane 1
    let tile_offset = 0x10; // Tile $01 starts at byte 0x10
    for i in 0..8 {
        chr[tile_offset + i] = 0xFF;     // Plane 0: all pixels on
        chr[tile_offset + 8 + i] = 0x00; // Plane 1: color index 1
    }
    
    rom.extend_from_slice(&chr);
    
    // Write to file
    let mut file = File::create("sprite_animation.nes").unwrap();
    file.write_all(&rom).unwrap();
    
    println!("Generated sprite_animation.nes");
    println!("Sprite moves 1 pixel right per frame, wraps at X=240");
}
