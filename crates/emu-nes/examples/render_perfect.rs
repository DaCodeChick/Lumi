use emu_nes::{framebuffer_to_rgb, NesSystem};
use std::fs::File;
use std::io::{self, Write};

fn main() -> io::Result<()> {
    let mut system = NesSystem::load("perfect_visual.nes")
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;

    println!("Running perfect visual test...\n");
    
    for _ in 0..10 {
        for _ in 0..29780 {
            system.step().map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;
        }
    }

    // Check nametable
    let ppu = system.ppu();
    print!("Nametable first 16 tiles: ");
    for i in 0..16 {
        print!("{} ", ppu.read_nametable_direct(0x2000 + i));
    }
    println!("\n");

    // Render
    let framebuffer = ppu.framebuffer();
    let rgb_data = framebuffer_to_rgb(framebuffer);
    
    write_ppm("perfect_output.ppm", &rgb_data, 256, 240)?;
    
    println!("Rendered to: perfect_output.ppm\n");
    
    // Stats
    let mut counts = std::collections::HashMap::new();
    for &idx in framebuffer {
        *counts.entry(idx).or_insert(0) += 1;
    }
    
    println!("Colors:");
    for (color, count) in counts.iter() {
        let (r, g, b) = emu_nes::palette_to_rgb(*color);
        println!("  ${:02X} RGB({:3},{:3},{:3}): {:6} pixels", color, r, g, b, count);
    }

    Ok(())
}

fn write_ppm(path: &str, rgb_data: &[u8], width: usize, height: usize) -> io::Result<()> {
    let mut file = File::create(path)?;
    writeln!(file, "P6")?;
    writeln!(file, "{} {}", width, height)?;
    writeln!(file, "255")?;
    file.write_all(rgb_data)?;
    Ok(())
}
