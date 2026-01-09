use emu_nes::{framebuffer_to_rgb, NesSystem};
use std::fs::File;
use std::io::{self, Write};

fn main() -> io::Result<()> {
    println!("LumiEmu - NES Visual Rendering Test");
    println!("====================================\n");

    // Load the visual test ROM
    let rom_path = "visual_test.nes";
    println!("Loading ROM: {}", rom_path);

    let mut system = match NesSystem::load(rom_path) {
        Ok(sys) => sys,
        Err(e) => {
            eprintln!("Error loading ROM: {:?}", e);
            eprintln!("\nMake sure to generate the visual test ROM first:");
            eprintln!("  cargo run --example generate_visual_rom -p emu-nes");
            return Err(io::Error::new(io::ErrorKind::Other, format!("{:?}", e)));
        }
    };

    println!("ROM loaded successfully!");
    println!("Running emulator...\n");

    // Run for multiple frames to let the program initialize
    // The test ROM needs to:
    // 1. Wait for 2 VBlanks (PPU warm-up)
    // 2. Load palette
    // 3. Fill nametable with tiles
    // 4. Set attribute table
    // 5. Enable rendering
    
    println!("Running initialization code...");
    for frame_num in 0..5 {
        println!("  Frame {}/5", frame_num + 1);
        for _ in 0..29780 {
            if let Err(e) = system.step() {
                eprintln!("Error during emulation: {:?}", e);
                return Err(io::Error::new(io::ErrorKind::Other, format!("{:?}", e)));
            }
        }
    }

    println!("\nRendering complete!");
    println!("Extracting framebuffer...\n");

    // Get the framebuffer from the PPU
    let framebuffer = system.ppu().framebuffer();
    
    // Convert to RGB
    let rgb_data = framebuffer_to_rgb(framebuffer);

    // Write as PPM file (simple image format)
    let output_path = "visual_output.ppm";
    write_ppm(output_path, &rgb_data, 256, 240)?;

    println!("Rendered frame saved to: {}", output_path);
    println!("\nYou can view the PPM file with:");
    println!("  - GIMP, Photoshop, or any image viewer that supports PPM");
    println!("  - Convert to PNG: convert visual_output.ppm visual_output.png");
    println!("  - View in terminal: feh visual_output.ppm");

    // Print some statistics
    println!("\nFramebuffer statistics:");
    let mut color_counts = std::collections::HashMap::new();
    for &palette_idx in framebuffer.iter() {
        *color_counts.entry(palette_idx).or_insert(0) += 1;
    }
    
    println!("  Unique colors used: {}", color_counts.len());
    println!("  Total pixels: {}", framebuffer.len());
    
    if color_counts.len() <= 16 {
        println!("\n  Color distribution:");
        let mut sorted: Vec<_> = color_counts.iter().collect();
        sorted.sort_by_key(|&(_, count)| std::cmp::Reverse(*count));
        for (color, count) in sorted.iter() {
            let (r, g, b) = emu_nes::palette_to_rgb(**color);
            println!("    Palette ${:02X} (RGB: {:3},{:3},{:3}): {:6} pixels ({:.1}%)", 
                     color, r, g, b, count, 
                     (**count as f32 / framebuffer.len() as f32) * 100.0);
        }
    }

    println!("\n✓ Visual test complete! Check the output image to see the rendered graphics.");

    Ok(())
}

/// Write a PPM (Portable Pixmap) image file
/// PPM is a simple uncompressed RGB format that's easy to generate
fn write_ppm(path: &str, rgb_data: &[u8], width: usize, height: usize) -> io::Result<()> {
    let mut file = File::create(path)?;
    
    // PPM header
    writeln!(file, "P6")?;
    writeln!(file, "{} {}", width, height)?;
    writeln!(file, "255")?;
    
    // RGB pixel data (already in the right format)
    file.write_all(rgb_data)?;
    
    Ok(())
}
