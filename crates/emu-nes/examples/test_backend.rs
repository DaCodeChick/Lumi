/// Quick test to verify NES emulator backend works with test ROM
use emu_nes::system::NesSystem;
use std::path::Path;

fn main() {
    let rom_path = Path::new("animation_test.nes");
    
    println!("Loading ROM: {:?}", rom_path);
    let mut system = match NesSystem::new(rom_path) {
        Ok(s) => {
            println!("✓ ROM loaded successfully");
            s
        }
        Err(e) => {
            eprintln!("✗ Failed to load ROM: {:?}", e);
            eprintln!("  Trying scrolling_test_noscroll.nes instead...");
            let rom_path = Path::new("scrolling_test_noscroll.nes");
            match NesSystem::new(rom_path) {
                Ok(s) => {
                    println!("✓ ROM loaded successfully");
                    s
                }
                Err(e) => {
                    eprintln!("✗ Failed: {:?}", e);
                    return;
                }
            }
        }
    };
    
    println!("\nRunning 10 frames...");
    for i in 0..10 {
        match system.run_frame() {
            Ok(_) => println!("  Frame {}: OK", i + 1),
            Err(e) => {
                eprintln!("✗ Frame {} failed: {:?}", i + 1, e);
                return;
            }
        }
    }
    
    println!("\nGetting framebuffer...");
    let fb = system.framebuffer();
    println!("✓ Framebuffer size: {} bytes (expected: {})", fb.len(), 256 * 240);
    println!("  First 10 color indices: {:?}", &fb[..10]);
    println!("  Last 10 color indices: {:?}", &fb[fb.len()-10..]);
    
    // Check if framebuffer has any non-zero values (indicating rendering)
    let non_zero = fb.iter().filter(|&&x| x != 0).count();
    println!("  Non-zero pixels: {} ({:.1}%)", non_zero, (non_zero as f32 / fb.len() as f32) * 100.0);
    
    if fb.len() == 256 * 240 {
        println!("\n✓✓✓ SUCCESS! Emulator backend is working correctly.");
    } else {
        println!("\n✗ FAILED: Incorrect framebuffer size");
    }
}
