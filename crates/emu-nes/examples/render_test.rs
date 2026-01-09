use emu_nes::{framebuffer_to_rgb, NesSystem};
use std::fs::File;
use std::io::{self, Write};

fn main() -> io::Result<()> {
    println!("LumiEmu - NES Rendering Test");
    println!("============================\n");

    // Load the test ROM
    let rom_path = "test.nes";
    println!("Loading ROM: {}", rom_path);

    let mut system = match NesSystem::load(rom_path) {
        Ok(sys) => sys,
        Err(e) => {
            eprintln!("Error loading ROM: {:?}", e);
            return Err(io::Error::new(io::ErrorKind::Other, format!("{:?}", e)));
        }
    };

    println!("ROM loaded successfully!");
    println!("Running emulator for 1 frame (29780 cycles)...\n");

    // Run for one full frame (1 frame = ~29780 CPU cycles)
    // This includes VBlank, so the PPU will render a complete frame
    let cycles_per_frame = 29780;
    
    for _ in 0..cycles_per_frame {
        if let Err(e) = system.step() {
            eprintln!("Error during emulation: {:?}", e);
            return Err(io::Error::new(io::ErrorKind::Other, format!("{:?}", e)));
        }
    }

    println!("Frame rendered!");
    println!("Extracting framebuffer...\n");

    // Get the framebuffer from the PPU
    let framebuffer = system.ppu().framebuffer();
    
    // Convert to RGB
    let rgb_data = framebuffer_to_rgb(framebuffer);

    // Write as PPM file (simple image format)
    let output_path = "output.ppm";
    write_ppm(output_path, &rgb_data, 256, 240)?;

    println!("Rendered frame saved to: {}", output_path);
    println!("\nYou can view the PPM file with:");
    println!("  - GIMP, Photoshop, or any image viewer that supports PPM");
    println!("  - Convert to PNG: convert output.ppm output.png");
    println!("  - View in terminal: feh output.ppm");

    // Print some statistics
    println!("\nFramebuffer statistics:");
    let mut color_counts = std::collections::HashMap::new();
    for &palette_idx in framebuffer.iter() {
        *color_counts.entry(palette_idx).or_insert(0) += 1;
    }
    
    println!("  Unique colors used: {}", color_counts.len());
    println!("  Total pixels: {}", framebuffer.len());
    
    if color_counts.len() <= 10 {
        println!("\n  Color distribution:");
        let mut sorted: Vec<_> = color_counts.iter().collect();
        sorted.sort_by_key(|&(_, count)| std::cmp::Reverse(*count));
        for (color, count) in sorted.iter().take(10) {
            let (r, g, b) = emu_nes::palette_to_rgb(**color);
            println!("    Palette ${:02X} (RGB: {:3},{:3},{:3}): {:6} pixels", 
                     color, r, g, b, count);
        }
    }

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
