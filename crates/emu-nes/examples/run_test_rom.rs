/// Run the test ROM and verify it executes correctly
/// 
/// This demonstrates loading a real iNES ROM file and running it on the emulator.

use emu_nes::NesSystem;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== LumiEmu NES Test ROM Runner ===\n");
    
    // Load the test ROM
    let rom_path = Path::new("test.nes");
    
    if !rom_path.exists() {
        eprintln!("Error: test.nes not found!");
        eprintln!("Run 'cargo run --example generate_test_rom' first to create it.");
        return Ok(());
    }
    
    println!("Loading ROM: {}", rom_path.display());
    let mut nes = NesSystem::new(rom_path)?;
    
    println!("ROM loaded successfully!");
    println!("Initial state:");
    println!("  PC: ${:04X}", nes.cpu().pc);
    println!("  A: ${:02X}, X: ${:02X}, Y: ${:02X}", nes.cpu().a, nes.cpu().x, nes.cpu().y);
    println!("  SP: ${:02X}", nes.cpu().sp);
    println!();
    
    // Run the test program
    println!("Executing test program...\n");
    
    let max_instructions = 200;
    let mut instruction_count = 0;
    let infinite_loop_pc = 0x803D; // Where our test program loops
    
    loop {
        let pc_before = nes.cpu().pc;
        
        // Check if we've reached the infinite loop
        if pc_before == infinite_loop_pc && instruction_count > 50 {
            println!("\nProgram completed (reached infinite loop at ${:04X})", infinite_loop_pc);
            break;
        }
        
        if instruction_count >= max_instructions {
            println!("\nMax instructions reached");
            break;
        }
        
        match nes.step() {
            Ok(cycles) => {
                instruction_count += 1;
                
                // Print first 10 instructions and then occasional updates
                if instruction_count <= 10 || instruction_count % 20 == 0 {
                    println!("#{:3} PC=${:04X} A=${:02X} X=${:02X} Y=${:02X} SP=${:02X} Cycles={}",
                        instruction_count, pc_before, nes.cpu().a, nes.cpu().x, nes.cpu().y, 
                        nes.cpu().sp, cycles);
                }
            }
            Err(e) => {
                println!("\nError at PC=${:04X}: {:?}", pc_before, e);
                break;
            }
        }
    }
    
    println!("\nFinal state:");
    println!("  PC: ${:04X}", nes.cpu().pc);
    println!("  A: ${:02X}, X: ${:02X}, Y: ${:02X}", nes.cpu().a, nes.cpu().x, nes.cpu().y);
    println!("  SP: ${:02X}", nes.cpu().sp);
    println!("  Total cycles: {}", nes.cpu().cycles);
    println!("  Total instructions: {}", instruction_count);
    println!();
    
    // Verify test results
    println!("Verifying test results:");
    println!("======================");
    
    let mut all_passed = true;
    
    // Test 1: Basic arithmetic (10 + 5 = 15)
    let result_01 = nes.read_memory(0x01);
    let expected_01 = 0x0F;
    let pass_01 = result_01 == expected_01;
    println!("Test 1 - Arithmetic:     $01 = ${:02X} (expected ${:02X}) {}", 
        result_01, expected_01, if pass_01 { "✓ PASS" } else { "✗ FAIL" });
    all_passed &= pass_01;
    
    // Test 2: Loop counter (10 iterations)
    let result_02 = nes.read_memory(0x02);
    let expected_02 = 0x0A;
    let pass_02 = result_02 == expected_02;
    println!("Test 2 - Loop counter:   $02 = ${:02X} (expected ${:02X}) {}", 
        result_02, expected_02, if pass_02 { "✓ PASS" } else { "✗ FAIL" });
    all_passed &= pass_02;
    
    // Test 3: Memory operations
    let result_11 = nes.read_memory(0x11);
    let expected_11 = 0x42;
    let pass_11 = result_11 == expected_11;
    println!("Test 3 - Memory ops:     $11 = ${:02X} (expected ${:02X}) {}", 
        result_11, expected_11, if pass_11 { "✓ PASS" } else { "✗ FAIL" });
    all_passed &= pass_11;
    
    // Test 4: Stack operations
    let result_20 = nes.read_memory(0x20);
    let expected_20 = 0x99;
    let pass_20 = result_20 == expected_20;
    println!("Test 4 - Stack ops:      $20 = ${:02X} (expected ${:02X}) {}", 
        result_20, expected_20, if pass_20 { "✓ PASS" } else { "✗ FAIL" });
    all_passed &= pass_20;
    
    // Test 5: Subroutine call
    let result_30 = nes.read_memory(0x30);
    let expected_30 = 0x55;
    let pass_30 = result_30 == expected_30;
    println!("Test 5 - Subroutine:     $30 = ${:02X} (expected ${:02X}) {}", 
        result_30, expected_30, if pass_30 { "✓ PASS" } else { "✗ FAIL" });
    all_passed &= pass_30;
    
    // Success flag
    let result_40 = nes.read_memory(0x40);
    let expected_40 = 0xFF;
    let pass_40 = result_40 == expected_40;
    println!("Test 6 - Success flag:   $40 = ${:02X} (expected ${:02X}) {}", 
        result_40, expected_40, if pass_40 { "✓ PASS" } else { "✗ FAIL" });
    all_passed &= pass_40;
    
    println!();
    if all_passed {
        println!("🎉 All tests PASSED! The NES emulator is working correctly!");
    } else {
        println!("❌ Some tests FAILED. Check the implementation.");
    }
    
    println!("\n=== Test Complete ===");
    
    Ok(())
}
