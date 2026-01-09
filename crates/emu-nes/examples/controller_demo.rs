/// Controller Demo
/// 
/// Demonstrates controller input functionality by:
/// 1. Loading the controller test ROM
/// 2. Setting various button states
/// 3. Running the ROM to read controller input
/// 4. Verifying button states were correctly read
/// 
/// Usage: cargo run --example controller_demo -p emu-nes

use emu_core::Button;
use emu_nes::NesSystem;

fn main() {
    println!("=== NES Controller Demo ===\n");
    
    // Check if ROM exists, if not provide instructions
    if !std::path::Path::new("controller_test.nes").exists() {
        println!("ERROR: controller_test.nes not found!");
        println!("Please generate it first by running:");
        println!("  cargo run --example generate_controller_test -p emu-nes");
        return;
    }
    
    println!("Loading controller_test.nes...");
    let mut system = NesSystem::load("controller_test.nes").unwrap();
    
    println!("Setting button states:");
    println!("  - Pressing A button");
    println!("  - Pressing Start button");
    println!("  - Pressing Up button");
    
    // Press some buttons
    system.press_button(Button::A);
    system.press_button(Button::START);
    system.press_button(Button::UP);
    
    println!("\nRunning ROM to read controller input...");
    
    // Run until success flag is set
    let mut cycles = 0;
    let max_cycles = 10000;
    
    while cycles < max_cycles {
        system.step().unwrap();
        cycles += 1;
        
        // Check if success flag is set ($10 = $FF)
        if system.read_memory(0x10) == 0xFF {
            println!("Success flag detected after {} instructions\n", cycles);
            break;
        }
    }
    
    if cycles >= max_cycles {
        println!("WARNING: Timeout - success flag not set\n");
    }
    
    // Read button states from memory
    println!("Button states read from memory:");
    
    let button_names = [
        "A", "B", "Select", "Start", 
        "Up", "Down", "Left", "Right"
    ];
    
    let expected_states = [
        true,  // A - pressed
        false, // B
        false, // Select
        true,  // Start - pressed
        true,  // Up - pressed
        false, // Down
        false, // Left
        false, // Right
    ];
    
    let mut all_correct = true;
    
    for i in 0..8 {
        let addr = i as u16;
        let value = system.read_memory(addr);
        let pressed = value != 0;
        let expected = expected_states[i];
        
        let status = if pressed == expected {
            "✓"
        } else {
            all_correct = false;
            "✗"
        };
        
        println!("  ${:02X} ({:>6}) = {} [{}] {}", 
            addr, 
            button_names[i], 
            value,
            if pressed { "pressed" } else { "released" },
            status
        );
    }
    
    println!();
    
    if all_correct {
        println!("SUCCESS: All button states are correct!");
    } else {
        println!("FAILURE: Some button states are incorrect!");
    }
    
    println!("\n=== Controller Demo Complete ===");
}
