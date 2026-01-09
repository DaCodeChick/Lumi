// Simple MIDI file generator for testing midi2nes
// Generates "Mary Had a Little Lamb" melody

use std::fs::File;
use std::io::Write;

fn main() {
    let mut midi = Vec::new();
    
    // MIDI Header
    // "MThd" chunk
    midi.extend_from_slice(b"MThd");
    midi.extend_from_slice(&6u32.to_be_bytes()); // Header length
    midi.extend_from_slice(&0u16.to_be_bytes()); // Format 0 (single track)
    midi.extend_from_slice(&1u16.to_be_bytes()); // 1 track
    midi.extend_from_slice(&480u16.to_be_bytes()); // 480 ticks per quarter note
    
    // Track chunk
    midi.extend_from_slice(b"MTrk");
    
    let mut track = Vec::new();
    
    // Set tempo: 120 BPM = 500000 microseconds per quarter note
    track.push(0x00); // Delta time = 0
    track.extend_from_slice(&[0xFF, 0x51, 0x03]); // Meta event: Set Tempo
    track.extend_from_slice(&500000u32.to_be_bytes()[1..]); // 3 bytes
    
    // Mary Had a Little Lamb melody
    // E D C D E E E (rest) D D D (rest) E G G
    // MIDI notes: E4=64, D4=62, C4=60, G4=67
    let melody = [
        (64, 480),  // E
        (62, 480),  // D
        (60, 480),  // C
        (62, 480),  // D
        (64, 480),  // E
        (64, 480),  // E
        (64, 960),  // E (longer)
        (62, 480),  // D
        (62, 480),  // D
        (62, 960),  // D (longer)
        (64, 480),  // E
        (67, 480),  // G
        (67, 960),  // G (longer)
    ];
    
    for (note, duration) in melody {
        // Note On
        track.push(0x00); // Delta time
        track.push(0x90); // Note On, channel 0
        track.push(note); // Note
        track.push(0x64); // Velocity (100)
        
        // Note Off (after duration)
        write_var_len(&mut track, duration);
        track.push(0x80); // Note Off, channel 0
        track.push(note); // Note
        track.push(0x40); // Velocity
    }
    
    // End of track
    track.push(0x00); // Delta time
    track.extend_from_slice(&[0xFF, 0x2F, 0x00]); // Meta event: End of Track
    
    // Write track length
    midi.extend_from_slice(&(track.len() as u32).to_be_bytes());
    midi.extend_from_slice(&track);
    
    // Write to file
    let mut file = File::create("test_mary.mid").unwrap();
    file.write_all(&midi).unwrap();
    
    println!("Generated test_mary.mid");
}

fn write_var_len(buf: &mut Vec<u8>, mut value: u32) {
    let mut bytes = Vec::new();
    bytes.push((value & 0x7F) as u8);
    value >>= 7;
    
    while value > 0 {
        bytes.push(((value & 0x7F) | 0x80) as u8);
        value >>= 7;
    }
    
    bytes.reverse();
    buf.extend_from_slice(&bytes);
}
