/// Demonstration of CPU + NES Memory System working together
/// 
/// This shows the 6502 CPU running with the full NES memory map,
/// including RAM mirroring and cartridge ROM access.

use emu_core::Cpu;
use emu_nes::{Cpu6502, NesMemory};
use emu_nes::cpu::CpuMemory;

fn main() {
    println!("=== LumiEmu NES System Demo ===\n");
    
    // Create NES memory system
    let mut memory = NesMemory::new();
    
    // Create a simple program in PRG-ROM that:
    // 1. Copies data from ROM to RAM
    // 2. Manipulates the data
    // 3. Tests RAM mirroring
    #[rustfmt::skip]
    let program = vec![
        // Program starts at $8000 (in ROM)
        // Copy a value from ROM ($8020) to RAM ($0000)
        0xAD, 0x20, 0x80,  // LDA $8020     ; Load from ROM
        0x85, 0x00,        // STA $00       ; Store to RAM
        0x85, 0x10,        // STA $10       ; Store to RAM again
        
        // Test RAM mirroring - write to $0000, read from mirror
        0xA9, 0x42,        // LDA #$42
        0x85, 0x00,        // STA $00       ; Write to base RAM
        0xAD, 0x00, 0x08,  // LDA $0800     ; Read from mirror (should be $42)
        0x85, 0x01,        // STA $01       ; Store result
        
        // Test ROM mirroring (16KB ROM mirrors in $C000-$FFFF)
        0xAD, 0x20, 0xC0,  // LDA $C020     ; Read from ROM mirror
        0x85, 0x02,        // STA $02       ; Store result
        
        // Done - infinite loop
        0x4C, 0x15, 0x80,  // JMP $8015 (jump to self)
    ];
    
    // Create 16KB ROM and place program + data
    let mut rom = vec![0xEA; 0x4000]; // Fill with NOPs
    rom[..program.len()].copy_from_slice(&program);
    rom[0x20] = 0x99; // Data byte at offset $20
    memory.load_prg_rom(rom);
    
    // Set reset vector to $8000
    // Note: In a real NES, the reset vector is at $FFFC-$FFFD
    // For this demo, we'll manually set the PC
    let mut cpu = Cpu6502::new(memory);
    cpu.reset();
    
    // Manually set PC to $8000 (our program start)
    // In a real NES, this would be read from the reset vector
    cpu.pc = 0x8000;
    
    println!("Initial state:");
    println!("  PC: ${:04X}", cpu.pc);
    println!("  A: ${:02X}, X: ${:02X}, Y: ${:02X}", cpu.a, cpu.x, cpu.y);
    println!();
    
    // Execute program
    println!("Executing program...\n");
    
    for step in 1..=15 {
        let pc_before = cpu.pc;
        
        match cpu.step() {
            Ok(cycles) => {
                println!("#{:2} PC=${:04X} A=${:02X} X=${:02X} Y=${:02X} Cycles={}",
                    step, pc_before, cpu.a, cpu.x, cpu.y, cycles);
                
                // Stop at infinite loop
                if cpu.pc == 0x8015 && step > 10 {
                    println!("\nProgram completed (reached infinite loop)");
                    break;
                }
            }
            Err(e) => {
                println!("Error at PC=${:04X}: {:?}", pc_before, e);
                break;
            }
        }
    }
    
    println!("\nFinal state:");
    println!("  PC: ${:04X}", cpu.pc);
    println!("  A: ${:02X}, X: ${:02X}, Y: ${:02X}", cpu.a, cpu.x, cpu.y);
    println!();
    
    // Check results in RAM
    println!("RAM contents:");
    println!("  $0000: ${:02X} (should be $42 - wrote via base address)", cpu.memory().read(0x0000));
    println!("  $0001: ${:02X} (should be $42 - read via $0800 mirror)", cpu.memory().read(0x0001));
    println!("  $0002: ${:02X} (should be $99 - read from ROM mirror $C020)", cpu.memory().read(0x0002));
    println!("  $0010: ${:02X} (should be $99 - copied from ROM)", cpu.memory().read(0x0010));
    
    // Verify mirroring works
    println!("\nVerifying RAM mirroring:");
    println!("  $0000 = ${:02X}", cpu.memory().read(0x0000));
    println!("  $0800 = ${:02X} (mirror of $0000)", cpu.memory().read(0x0800));
    println!("  $1000 = ${:02X} (mirror of $0000)", cpu.memory().read(0x1000));
    println!("  $1800 = ${:02X} (mirror of $0000)", cpu.memory().read(0x1800));
    
    println!("\n=== Demo Complete ===");
}
