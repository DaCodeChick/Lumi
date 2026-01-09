use std::fs::File;
use std::io::Write;

fn main() {
    let mut rom = Vec::new();
    
    // iNES Header
    rom.extend_from_slice(b"NES\x1a");
    rom.push(1);  // 1 PRG ROM bank (16KB)
    rom.push(0);  // 0 CHR ROM banks (using CHR-RAM)
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
    
    // Configure Pulse 1 for 440Hz tone (A4)
    // Duty cycle: 50% (duty = 2)
    // Length counter halt: enabled
    // Constant volume: enabled
    // Volume: 15 (max)
    prg[pc - 0xC000] = 0xA9; // LDA #$BF (duty=2, length_halt=1, const_vol=1, vol=15)
    prg[pc - 0xC000 + 1] = 0xBF;
    pc += 2;
    prg[pc - 0xC000] = 0x8D; // STA $4000
    prg[pc - 0xC000 + 1] = 0x00;
    prg[pc - 0xC000 + 2] = 0x40;
    pc += 3;
    
    // No sweep
    prg[pc - 0xC000] = 0xA9; // LDA #$00
    prg[pc - 0xC000 + 1] = 0x00;
    pc += 2;
    prg[pc - 0xC000] = 0x8D; // STA $4001
    prg[pc - 0xC000 + 1] = 0x01;
    prg[pc - 0xC000 + 2] = 0x40;
    pc += 3;
    
    // Timer period for 440Hz
    // CPU clock: 1.789773 MHz
    // APU clock: CPU / 2 = 894,886.5 Hz
    // Period = (APU_CLOCK / (16 * frequency)) - 1
    // Period = (894886.5 / (16 * 440)) - 1 = 126.7 ≈ 127
    // Timer low byte
    prg[pc - 0xC000] = 0xA9; // LDA #$7F (127 & 0xFF)
    prg[pc - 0xC000 + 1] = 0x7F;
    pc += 2;
    prg[pc - 0xC000] = 0x8D; // STA $4002
    prg[pc - 0xC000 + 1] = 0x02;
    prg[pc - 0xC000 + 2] = 0x40;
    pc += 3;
    
    // Enable Pulse 1 FIRST (before writing $4003)
    prg[pc - 0xC000] = 0xA9; // LDA #$01
    prg[pc - 0xC000 + 1] = 0x01;
    pc += 2;
    prg[pc - 0xC000] = 0x8D; // STA $4015
    prg[pc - 0xC000 + 1] = 0x15;
    prg[pc - 0xC000 + 2] = 0x40;
    pc += 3;
    
    // Timer high byte + length counter (AFTER enabling channel)
    prg[pc - 0xC000] = 0xA9; // LDA #$00 (timer_high=0, length=10)
    prg[pc - 0xC000 + 1] = 0x00;
    pc += 2;
    prg[pc - 0xC000] = 0x8D; // STA $4003
    prg[pc - 0xC000 + 1] = 0x03;
    prg[pc - 0xC000 + 2] = 0x40;
    pc += 3;
    
    // Enable NMI
    prg[pc - 0xC000] = 0xA9; // LDA #$80
    prg[pc - 0xC000 + 1] = 0x80;
    pc += 2;
    prg[pc - 0xC000] = 0x8D; // STA $2000
    prg[pc - 0xC000 + 1] = 0x00;
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
    
    // Just return (keep sound playing)
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
    
    // No CHR ROM (using CHR-RAM)
    
    // Write to file
    let mut file = File::create("audio_test_440hz.nes").unwrap();
    file.write_all(&rom).unwrap();
    
    println!("Generated audio_test_440hz.nes");
    println!("Plays a 440Hz tone (musical note A4)");
}
