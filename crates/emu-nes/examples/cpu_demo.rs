//! Simple demonstration of the 6502 CPU executing a small program
//!
//! This example shows the CPU executing a simple program that:
//! 1. Loads values into registers
//! 2. Performs arithmetic operations
//! 3. Stores results in memory
//! 4. Uses branches and loops

use emu_core::Cpu;
use emu_nes::cpu::{Cpu6502, CpuMemory};

/// Simple memory implementation for testing
struct SimpleMemory {
    ram: Vec<u8>,
}

impl SimpleMemory {
    fn new() -> Self {
        Self {
            ram: vec![0; 0x10000], // 64KB
        }
    }

    fn load_program(&mut self, start_addr: u16, program: &[u8]) {
        for (i, &byte) in program.iter().enumerate() {
            self.ram[start_addr as usize + i] = byte;
        }
    }
}

impl CpuMemory for SimpleMemory {
    fn read(&mut self, addr: u16) -> u8 {
        self.ram[addr as usize]
    }

    fn write(&mut self, addr: u16, value: u8) {
        self.ram[addr as usize] = value;
    }
}

fn main() {
    println!("=== LumiEmu 6502 CPU Demo ===\n");

    // Create memory and CPU
    let mut memory = SimpleMemory::new();
    
    // Simple program that calculates: result = (10 + 5) * 2
    // Using the 6502's limited instruction set
    #[rustfmt::skip]
    let program = vec![
        // Start at $0200
        0xA9, 0x0A,        // LDA #$0A      ; A = 10
        0x69, 0x05,        // ADC #$05      ; A = 10 + 5 = 15
        0x85, 0x20,        // STA $20       ; Store 15 in memory
        0x0A,              // ASL A         ; A = 15 << 1 = 30 (multiply by 2)
        0x85, 0x21,        // STA $21       ; Store result in memory
        
        // Demonstrate a simple loop: count from 0 to 5
        0xA2, 0x00,        // LDX #$00      ; X = 0 (counter)
        // Loop start at $020B:
        0xE8,              // INX           ; X++
        0xE0, 0x05,        // CPX #$05      ; Compare X with 5
        0xD0, 0xFB,        // BNE -5        ; If X != 5, branch back to INX ($020B)
        0x8E, 0x22, 0x00,  // STX $0022     ; Store final counter value
        
        // Demonstrate subroutine call
        0x20, 0x1B, 0x02,  // JSR $021B     ; Call subroutine
        0x85, 0x23,        // STA $23       ; Store returned value
        
        0x4C, 0x18, 0x02,  // JMP $0218     ; Infinite loop (program end)
        
        // Subroutine at $021B: returns A = 42
        0xA9, 0x2A,        // LDA #$2A      ; A = 42
        0x60,              // RTS           ; Return
    ];

    // Load program at $0200
    memory.load_program(0x0200, &program);
    
    // Set reset vector to point to our program
    memory.ram[0xFFFC] = 0x00;
    memory.ram[0xFFFD] = 0x02;
    
    // Create CPU and reset
    let mut cpu = Cpu6502::new(memory);
    cpu.reset();
    
    println!("Initial state:");
    println!("  PC: ${:04X}", cpu.pc());
    println!("  A: ${:02X}, X: ${:02X}, Y: ${:02X}", cpu.a(), cpu.x(), cpu.y());
    println!("  SP: ${:02X}", cpu.sp());
    println!();

    // Execute the program step by step
    let mut instruction_count = 0;
    let max_instructions = 100;

    println!("Executing program...\n");

    while instruction_count < max_instructions {
        let pc_before = cpu.pc();
        
        match cpu.step() {
            Ok(cycles) => {
                instruction_count += 1;
                
                // Print first 10 instructions, then interesting milestones
                if instruction_count <= 10 || instruction_count % 5 == 0 || instruction_count >= 25 {
                    println!("#{:2} PC=${:04X} A=${:02X} X=${:02X} Y=${:02X} SP=${:02X} Cycles={}",
                        instruction_count, pc_before, cpu.a(), cpu.x(), cpu.y(), cpu.sp(), cycles);
                }

                // Check if we've reached the infinite loop (program completed successfully)
                // After JSR/RTS/STA, we enter an infinite loop at $0218: JMP $0218
                if cpu.pc() == 0x0218 && instruction_count > 20 {
                    println!("\nProgram completed successfully!");
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
    println!("  PC: ${:04X}", cpu.pc());
    println!("  A: ${:02X}, X: ${:02X}, Y: ${:02X}", cpu.a(), cpu.x(), cpu.y());
    println!("  SP: ${:02X}", cpu.sp());
    println!("  Total cycles: {}", cpu.cycles);
    println!();
    
    // Read results from memory
    println!("Results in memory:");
    println!("  $0020: ${:02X} (should be 15 = 10 + 5)", cpu.memory().read(0x20));
    println!("  $0021: ${:02X} (should be 30 = 15 * 2)", cpu.memory().read(0x21));
    println!("  $0022: ${:02X} (should be 5 = loop counter)", cpu.memory().read(0x22));
    println!("  $0023: ${:02X} (should be 42 = returned from subroutine)", cpu.memory().read(0x23));
    
    println!("\n=== Demo Complete ===");
}
