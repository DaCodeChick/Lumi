use anyhow::{Context, Result};
use clap::Parser;
use midly::{Smf, Timing, TrackEventKind, MidiMessage};
use std::fs;
use std::path::PathBuf;

/// Convert MIDI files to NES ROM chiptunes
#[derive(Parser, Debug)]
#[command(name = "midi2nes")]
#[command(about = "Convert MIDI files to NES ROM chiptunes", long_about = None)]
struct Args {
    /// Input MIDI file
    #[arg(value_name = "MIDI_FILE")]
    input: PathBuf,

    /// Output NES ROM file (default: input name with .nes extension)
    #[arg(short, long, value_name = "OUTPUT")]
    output: Option<PathBuf>,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

/// A musical note event
#[derive(Debug, Clone)]
struct NoteEvent {
    /// Time in ticks
    time: u32,
    /// MIDI note number (0-127)
    note: u8,
    /// Velocity (0-127, 0 = note off)
    velocity: u8,
    /// Channel assignment (0-3 for Pulse1, Pulse2, Triangle, Noise)
    channel: u8,
}

/// Music data for embedding in ROM
struct MusicData {
    /// Tempo in microseconds per quarter note
    tempo: u32,
    /// Ticks per quarter note
    ticks_per_quarter: u16,
    /// List of note events (sorted by time)
    events: Vec<NoteEvent>,
}

impl MusicData {
    /// Convert MIDI note number to NES APU timer period
    /// Formula: period = CPU_CLOCK / (16 * frequency) - 1
    /// CPU_CLOCK = 1789773 Hz (NTSC)
    fn midi_note_to_apu_period(note: u8) -> u16 {
        // MIDI note 69 = A4 = 440 Hz
        // frequency = 440 * 2^((note - 69) / 12)
        const CPU_CLOCK: f64 = 1789773.0;
        let note_f64 = note as f64;
        let frequency = 440.0 * 2.0_f64.powf((note_f64 - 69.0) / 12.0);
        let period = (CPU_CLOCK / (16.0 * frequency)) - 1.0;
        
        // Clamp to valid range (0-2047 for 11-bit period)
        period.max(0.0).min(2047.0) as u16
    }
    
    /// Encode music data as bytes for ROM
    fn encode(&self) -> Vec<u8> {
        let mut data = Vec::new();
        
        // Header: tempo (4 bytes), ticks_per_quarter (2 bytes), event_count (2 bytes)
        data.extend_from_slice(&self.tempo.to_le_bytes());
        data.extend_from_slice(&self.ticks_per_quarter.to_le_bytes());
        data.extend_from_slice(&(self.events.len() as u16).to_le_bytes());
        
        // Events: each event is 8 bytes
        // [time: 4 bytes][note: 1 byte][velocity: 1 byte][channel: 1 byte][period: 2 bytes]
        for event in &self.events {
            let period = Self::midi_note_to_apu_period(event.note);
            data.extend_from_slice(&event.time.to_le_bytes());
            data.push(event.note);
            data.push(event.velocity);
            data.push(event.channel);
            data.extend_from_slice(&period.to_le_bytes());
        }
        
        data
    }
}

fn parse_midi(path: &PathBuf, verbose: bool) -> Result<MusicData> {
    let data = fs::read(path)?;
    let smf = Smf::parse(&data)?;
    
    if verbose {
        println!("MIDI format: {:?}", smf.header.format);
        println!("Number of tracks: {}", smf.tracks.len());
    }
    
    // Get timing information
    let ticks_per_quarter = match smf.header.timing {
        Timing::Metrical(tpq) => tpq.as_int(),
        Timing::Timecode(_, _) => {
            anyhow::bail!("Timecode-based MIDI files are not supported");
        }
    };
    
    if verbose {
        println!("Ticks per quarter note: {}", ticks_per_quarter);
    }
    
    // Default tempo: 120 BPM = 500000 microseconds per quarter note
    let mut tempo = 500000u32;
    let mut events = Vec::new();
    
    // Track current time and active notes per channel
    let mut current_time = 0u32;
    let mut channel_notes: [Option<u8>; 4] = [None; 4]; // Track which note is playing on each NES channel
    
    // Simple channel allocation: distribute MIDI channels to NES channels
    // Pulse1: MIDI channels 0-3
    // Pulse2: MIDI channels 4-7
    // Triangle: MIDI channels 8-11
    // Noise: MIDI channel 9 (standard percussion channel)
    
    // Process all tracks
    for (track_idx, track) in smf.tracks.iter().enumerate() {
        current_time = 0;
        
        for event in track {
            current_time += event.delta.as_int();
            
            match event.kind {
                TrackEventKind::Meta(meta) => {
                    if let midly::MetaMessage::Tempo(new_tempo) = meta {
                        tempo = new_tempo.as_int();
                        if verbose {
                            println!("Tempo change: {} μs/quarter note", tempo);
                        }
                    }
                }
                TrackEventKind::Midi { channel, message } => {
                    let midi_channel = channel.as_int();
                    
                    // Map MIDI channel to NES channel
                    let nes_channel = if midi_channel == 9 {
                        3 // Percussion -> Noise channel
                    } else if midi_channel < 4 {
                        0 // Pulse 1
                    } else if midi_channel < 8 {
                        1 // Pulse 2
                    } else {
                        2 // Triangle
                    };
                    
                    match message {
                        MidiMessage::NoteOn { key, vel } => {
                            let note = key.as_int();
                            let velocity = vel.as_int();
                            
                            if velocity > 0 {
                                // Note on
                                events.push(NoteEvent {
                                    time: current_time,
                                    note,
                                    velocity,
                                    channel: nes_channel,
                                });
                                channel_notes[nes_channel as usize] = Some(note);
                            } else {
                                // Note off (velocity = 0)
                                if channel_notes[nes_channel as usize].is_some() {
                                    events.push(NoteEvent {
                                        time: current_time,
                                        note,
                                        velocity: 0,
                                        channel: nes_channel,
                                    });
                                    channel_notes[nes_channel as usize] = None;
                                }
                            }
                        }
                        MidiMessage::NoteOff { key, .. } => {
                            let note = key.as_int();
                            
                            if channel_notes[nes_channel as usize].is_some() {
                                events.push(NoteEvent {
                                    time: current_time,
                                    note,
                                    velocity: 0,
                                    channel: nes_channel,
                                });
                                channel_notes[nes_channel as usize] = None;
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }
    
    // Sort events by time
    events.sort_by_key(|e| e.time);
    
    if verbose {
        println!("Total events: {}", events.len());
        println!("Duration: {} ticks", events.last().map(|e| e.time).unwrap_or(0));
    }
    
    Ok(MusicData {
        tempo,
        ticks_per_quarter,
        events,
    })
}

fn generate_rom(music: &MusicData, output: &PathBuf, verbose: bool) -> Result<()> {
    if verbose {
        println!("Generating NES ROM...");
    }
    
    let music_data = music.encode();
    
    if verbose {
        println!("Music data size: {} bytes", music_data.len());
    }
    
    // Build ROM structure
    let mut rom = Vec::new();
    
    // iNES Header
    rom.extend_from_slice(b"NES\x1a");
    rom.push(2);  // 2 PRG ROM banks (32KB) - need space for music data
    rom.push(0);  // 0 CHR ROM banks
    rom.push(0);  // Mapper 0, horizontal mirroring
    rom.extend_from_slice(&[0; 9]);
    
    // PRG ROM (32KB)
    let mut prg = vec![0u8; 32768];
    
    // Music player code starts at $8000
    let code_start = 0x8000;
    let mut pc = code_start;
    
    // Music data will be placed at $C000 (second PRG bank)
    let music_data_addr = 0xC000u16;
    
    // === RESET Handler ===
    let reset_addr = 0x8000;
    pc = reset_addr;
    
    // Wait for PPU warmup (simplified)
    for _ in 0..2 {
        let wait_loop = pc;
        prg[(pc - code_start) as usize] = 0x2C; // BIT $2002
        prg[(pc - code_start + 1) as usize] = 0x02;
        prg[(pc - code_start + 2) as usize] = 0x20;
        pc += 3;
        prg[(pc - code_start) as usize] = 0x10; // BPL wait_loop
        prg[(pc - code_start + 1) as usize] = ((wait_loop as i32 - (pc as i32 + 2)) as i8) as u8;
        pc += 2;
    }
    
    // Initialize APU
    // Enable all channels
    prg[(pc - code_start) as usize] = 0xA9; // LDA #$0F
    prg[(pc - code_start + 1) as usize] = 0x0F;
    pc += 2;
    prg[(pc - code_start) as usize] = 0x8D; // STA $4015
    prg[(pc - code_start + 1) as usize] = 0x15;
    prg[(pc - code_start + 2) as usize] = 0x40;
    pc += 3;
    
    // Set up pulse channels with 50% duty, constant volume
    for channel_base in [0x4000u16, 0x4004u16] {
        prg[(pc - code_start) as usize] = 0xA9; // LDA #$BF (50% duty, length halt, const vol=15)
        prg[(pc - code_start + 1) as usize] = 0xBF;
        pc += 2;
        prg[(pc - code_start) as usize] = 0x8D; // STA $400x
        prg[(pc - code_start + 1) as usize] = (channel_base & 0xFF) as u8;
        prg[(pc - code_start + 2) as usize] = ((channel_base >> 8) & 0xFF) as u8;
        pc += 3;
    }
    
    // Initialize triangle channel
    prg[(pc - code_start) as usize] = 0xA9; // LDA #$FF (length halt, linear counter load)
    prg[(pc - code_start + 1) as usize] = 0xFF;
    pc += 2;
    prg[(pc - code_start) as usize] = 0x8D; // STA $4008
    prg[(pc - code_start + 1) as usize] = 0x08;
    prg[(pc - code_start + 2) as usize] = 0x40;
    pc += 3;
    
    // Initialize noise channel
    prg[(pc - code_start) as usize] = 0xA9; // LDA #$3F (length halt, const vol=15)
    prg[(pc - code_start + 1) as usize] = 0x3F;
    pc += 2;
    prg[(pc - code_start) as usize] = 0x8D; // STA $400C
    prg[(pc - code_start + 1) as usize] = 0x0C;
    prg[(pc - code_start + 2) as usize] = 0x40;
    pc += 3;
    
    // Initialize music playback variables
    // $00-$01: Current event index (16-bit)
    // $02-$05: Current time counter (32-bit)
    prg[(pc - code_start) as usize] = 0xA9; // LDA #$00
    prg[(pc - code_start + 1) as usize] = 0x00;
    pc += 2;
    for addr in 0x00..=0x05 {
        prg[(pc - code_start) as usize] = 0x85; // STA $addr
        prg[(pc - code_start + 1) as usize] = addr;
        pc += 2;
    }
    
    // Enable NMI
    prg[(pc - code_start) as usize] = 0xA9; // LDA #$80
    prg[(pc - code_start + 1) as usize] = 0x80;
    pc += 2;
    prg[(pc - code_start) as usize] = 0x8D; // STA $2000
    prg[(pc - code_start + 1) as usize] = 0x00;
    prg[(pc - code_start + 2) as usize] = 0x20;
    pc += 3;
    
    // Main loop - infinite loop
    let main_loop = pc;
    prg[(pc - code_start) as usize] = 0x4C; // JMP main_loop
    prg[(pc - code_start + 1) as usize] = (main_loop & 0xFF) as u8;
    prg[(pc - code_start + 2) as usize] = ((main_loop >> 8) & 0xFF) as u8;
    pc += 3;
    
    // === NMI Handler (called 60 times per second) ===
    let nmi_addr = pc;
    
    // For now, simple handler that just returns
    // TODO: Implement music playback logic that reads music data and updates APU
    prg[(pc - code_start) as usize] = 0x40; // RTI
    pc += 1;
    
    // Place music data at $C000 (second bank, offset 0x4000 in PRG)
    let music_data_offset = 0x4000;
    if music_data_offset + music_data.len() > prg.len() {
        anyhow::bail!("Music data too large to fit in ROM");
    }
    prg[music_data_offset..music_data_offset + music_data.len()].copy_from_slice(&music_data);
    
    // Set interrupt vectors at $FFFA-$FFFF
    let vector_offset = 0xFFFA - code_start;
    prg[vector_offset as usize] = (nmi_addr & 0xFF) as u8;      // NMI low
    prg[vector_offset as usize + 1] = ((nmi_addr >> 8) & 0xFF) as u8; // NMI high
    prg[vector_offset as usize + 2] = (reset_addr & 0xFF) as u8;    // RESET low
    prg[vector_offset as usize + 3] = ((reset_addr >> 8) & 0xFF) as u8; // RESET high
    prg[vector_offset as usize + 4] = 0x00; // IRQ low
    prg[vector_offset as usize + 5] = 0x00; // IRQ high
    
    rom.extend_from_slice(&prg);
    
    // Write ROM to file
    fs::write(output, rom)?;
    
    if verbose {
        println!("ROM written to: {}", output.display());
    }
    
    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    // Determine output filename
    let output = args.output.unwrap_or_else(|| {
        let mut path = args.input.clone();
        path.set_extension("nes");
        path
    });
    
    if args.verbose {
        println!("Input: {}", args.input.display());
        println!("Output: {}", output.display());
        println!();
    }
    
    // Parse MIDI file
    let music = parse_midi(&args.input, args.verbose)
        .context("Failed to parse MIDI file")?;
    
    if args.verbose {
        println!();
    }
    
    // Generate ROM
    generate_rom(&music, &output, args.verbose)
        .context("Failed to generate ROM")?;
    
    println!("✓ Successfully converted {} to {}", 
             args.input.display(), 
             output.display());
    
    Ok(())
}
