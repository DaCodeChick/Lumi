/// Scrolling Comparison Demo
/// 
/// Loads two ROMs side-by-side to demonstrate scrolling:
/// 1. No scroll (X=0, Y=0)
/// 2. With scroll (X=64, Y=32)
/// 
/// Usage: cargo run --example scrolling_compare -p emu-nes

use emu_nes::NesSystem;
use std::fs::File;
use std::io::Write;

fn main() {
    println!("=== NES Scrolling Comparison Demo ===\n");
    
    // Check if ROMs exist
    if !std::path::Path::new("scrolling_test_noscroll.nes").exists() ||
       !std::path::Path::new("scrolling_test_scroll.nes").exists() {
        println!("ERROR: Test ROMs not found!");
        println!("Please generate them first by running:");
        println!("  cargo run --example generate_scrolling_tests -p emu-nes");
        return;
    }
    
    println!("Loading no-scroll ROM...");
    let mut system_noscroll = NesSystem::load("scrolling_test_noscroll.nes").unwrap();
    
    println!("Loading scrolled ROM...");
    let mut system_scroll = NesSystem::load("scrolling_test_scroll.nes").unwrap();
    
    println!("Running ROMs to set up...\n");
    
    // Run both systems
    for _ in 0..5 {
        system_noscroll.run_frame().unwrap();
        system_scroll.run_frame().unwrap();
    }
    
    // Render one more frame for output
    system_noscroll.run_frame().unwrap();
    system_scroll.run_frame().unwrap();
    
    println!("=== NO SCROLL (X=0, Y=0) ===");
    analyze_framebuffer("noscroll", system_noscroll.framebuffer());
    
    println!("\n=== WITH SCROLL (X=64, Y=32) ===");
    analyze_framebuffer("scroll", system_scroll.framebuffer());
    
    println!("\n=== Comparison ===");
    println!("View the images to see the scrolling effect:");
    println!("  - scrolling_noscroll.ppm: Original position");
    println!("  - scrolling_scroll.ppm: Scrolled by 64 pixels right, 32 pixels down");
    println!("  - The visible area should show different parts of the same nametable");
    
    println!("\n=== Scrolling Comparison Complete ===");
}

fn analyze_framebuffer(name: &str, framebuffer: &[u8]) {
    let filename = format!("scrolling_{}.ppm", name);
    
    // Convert and save
    let rgb = framebuffer_to_rgb(framebuffer);
    let mut file = File::create(&filename).unwrap();
    writeln!(file, "P6").unwrap();
    writeln!(file, "256 240").unwrap();
    writeln!(file, "255").unwrap();
    file.write_all(&rgb).unwrap();
    
    println!("Saved to {}", filename);
    
    // Show top-left corner
    print!("  Top-left corner (first 3 rows): ");
    let mut sample = Vec::new();
    for y in 0..3 {
        for x in 0..8 {
            sample.push(framebuffer[y * 256 + x]);
        }
    }
    
    // Count unique values in sample
    let unique: std::collections::HashSet<_> = sample.iter().collect();
    println!("{} unique values", unique.len());
    
    // Count palette usage
    use std::collections::HashMap;
    let mut counts: HashMap<u8, usize> = HashMap::new();
    for &pixel in framebuffer {
        *counts.entry(pixel).or_insert(0) += 1;
    }
    
    println!("  Palette usage:");
    let mut items: Vec<_> = counts.iter().collect();
    items.sort_by_key(|(k, _)| *k);
    for (idx, count) in items {
        println!("    ${:02X}: {} pixels", idx, count);
    }
}

/// Convert NES palette indices to RGB values
fn framebuffer_to_rgb(framebuffer: &[u8]) -> Vec<u8> {
    // NES palette (simplified - using common RGB values)
    const PALETTE: [(u8, u8, u8); 64] = [
        (84, 84, 84), (0, 30, 116), (8, 16, 144), (48, 0, 136),
        (68, 0, 100), (92, 0, 48), (84, 4, 0), (60, 24, 0),
        (32, 42, 0), (8, 58, 0), (0, 64, 0), (0, 60, 0),
        (0, 50, 60), (0, 0, 0), (0, 0, 0), (0, 0, 0),
        (152, 150, 152), (8, 76, 196), (48, 50, 236), (92, 30, 228),
        (136, 20, 176), (160, 20, 100), (152, 34, 32), (120, 60, 0),
        (84, 90, 0), (40, 114, 0), (8, 124, 0), (0, 118, 40),
        (0, 102, 120), (0, 0, 0), (0, 0, 0), (0, 0, 0),
        (236, 238, 236), (76, 154, 236), (120, 124, 236), (176, 98, 236),
        (228, 84, 236), (236, 88, 180), (236, 106, 100), (212, 136, 32),
        (160, 170, 0), (116, 196, 0), (76, 208, 32), (56, 204, 108),
        (56, 180, 204), (60, 60, 60), (0, 0, 0), (0, 0, 0),
        (236, 238, 236), (168, 204, 236), (188, 188, 236), (212, 178, 236),
        (236, 174, 236), (236, 174, 212), (236, 180, 176), (228, 196, 144),
        (204, 210, 120), (180, 222, 120), (168, 226, 144), (152, 226, 180),
        (160, 214, 228), (160, 162, 160), (0, 0, 0), (0, 0, 0),
    ];
    
    let mut rgb = Vec::with_capacity(framebuffer.len() * 3);
    
    for &palette_index in framebuffer {
        let (r, g, b) = PALETTE[(palette_index & 0x3F) as usize];
        rgb.push(r);
        rgb.push(g);
        rgb.push(b);
    }
    
    rgb
}
