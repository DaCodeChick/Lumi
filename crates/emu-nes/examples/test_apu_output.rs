use emu_nes::Apu;

fn main() {
    let mut apu = Apu::new();
    
    println!("Testing APU audio output...\n");
    
    // Configure Pulse 1 like our audio_test_440hz.nes ROM does
    println!("Configuring Pulse 1:");
    println!("  $4000 = $BF (duty=2/50%, length_halt=1, const_vol=1, vol=15)");
    apu.write_register(0x4000, 0xBF);
    
    println!("  $4001 = $00 (no sweep)");
    apu.write_register(0x4001, 0x00);
    
    println!("  $4002 = $7F (timer low = 127)");
    apu.write_register(0x4002, 0x7F);
    
    println!("  $4015 = $01 (enable Pulse 1) - MUST BE BEFORE $4003!");
    apu.write_register(0x4015, 0x01);
    
    println!("  $4003 = $00 (timer high = 0, length = 10)");
    apu.write_register(0x4003, 0x00);
    
    println!("\nGenerating 100 samples:");
    let mut sample_counts = [0; 16];
    for i in 0..100 {
        // Clock the APU a few times between samples
        for _ in 0..40 {
            apu.clock();
        }
        
        let sample = apu.output();
        let pulse1_out = apu.pulse1.output();
        
        sample_counts[pulse1_out as usize] += 1;
        
        if i < 20 {
            println!("  Sample {}: pulse1={}, mixed={:.4}", i, pulse1_out, sample);
        }
    }
    
    println!("\nPulse1 output value distribution:");
    for (value, count) in sample_counts.iter().enumerate() {
        if *count > 0 {
            println!("  Value {}: {} times", value, count);
        }
    }
}
