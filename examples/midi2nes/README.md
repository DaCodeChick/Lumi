# midi2nes - MIDI to NES ROM Converter

Convert MIDI files into NES ROM chiptunes that play on the NES APU.

## Features

- ✅ Parse MIDI files (Format 0 and 1)
- ✅ Convert MIDI notes to NES APU timer periods
- ✅ Map MIDI channels to NES channels:
  - MIDI channels 0-3 → Pulse 1 (square wave)
  - MIDI channels 4-7 → Pulse 2 (square wave)
  - MIDI channels 8-11 → Triangle (bass)
  - MIDI channel 9 → Noise (percussion)
- ✅ Generate valid NES ROMs with embedded music data
- ⚠️ Playback engine (TODO - needs 6502 assembly implementation)

## Usage

```bash
# Convert MIDI to NES ROM
cargo run -p midi2nes -- input.mid

# With verbose output
cargo run -p midi2nes -- input.mid -v

# Specify output filename
cargo run -p midi2nes -- input.mid -o output.nes
```

## Example

```bash
# Generate test MIDI file (Mary Had a Little Lamb)
cd examples/midi2nes
rustc gen_test_midi.rs -o gen_test_midi
./gen_test_midi

# Convert to NES ROM
cargo run -p midi2nes -- test_mary.mid -v
```

## How It Works

### 1. MIDI Parsing
- Uses the `midly` crate to parse MIDI files
- Extracts note on/off events, timing, and tempo
- Converts tick-based timing to frame-based timing

### 2. Note Conversion
MIDI notes are converted to NES APU timer periods using:
```
frequency = 440 * 2^((note - 69) / 12)
period = (CPU_CLOCK / (16 * frequency)) - 1
```

Where CPU_CLOCK = 1,789,773 Hz (NTSC)

### 3. Channel Mapping
- **Pulse channels**: 50% duty cycle, constant volume
- **Triangle channel**: Full linear counter for sustained bass
- **Noise channel**: Mode 0 for standard percussion

### 4. ROM Structure
```
$8000-$BFFF: Code (reset handler, APU init, playback engine)
$C000-$FFFF: Music data
  Header: [tempo:4][ticks_per_quarter:2][event_count:2]
  Events: [time:4][note:1][velocity:1][channel:1][period:2] (8 bytes each)
```

## Current Limitations

1. **No playback yet**: ROM generates but doesn't play music
   - Needs 6502 assembly music player implementation
   - Requires frame counter and timing logic
   
2. **Limited polyphony**: Only 4 channels (NES hardware limit)
   - Multiple notes on same MIDI channel will conflict
   - No voice stealing or note priority

3. **No dynamics**: Velocity is encoded but not used yet

4. **No tempo changes**: Only initial tempo is used

5. **No effects**: No vibrato, slides, or other modulation

## Future Enhancements

- [ ] Implement 6502 playback engine in NES ROM
- [ ] Support tempo changes during playback
- [ ] Add velocity-based volume control
- [ ] Implement note priority/voice stealing
- [ ] Add vibrato and pitch bend support
- [ ] Support longer songs with bank switching
- [ ] Add visual feedback (display notes on screen)
- [ ] Optimize for smaller ROM sizes

## Technical Details

### Music Data Format

Each event is 8 bytes:
- **time** (4 bytes): Event timestamp in MIDI ticks
- **note** (1 byte): MIDI note number (0-127)
- **velocity** (1 byte): Note velocity (0=off, 1-127=on)
- **channel** (1 byte): NES channel (0=Pulse1, 1=Pulse2, 2=Triangle, 3=Noise)
- **period** (2 bytes): Pre-calculated APU timer period

### APU Initialization

The ROM initializes the APU with:
- All channels enabled ($4015 = $0F)
- Pulse 1 & 2: 50% duty, length counter halt, constant volume 15
- Triangle: Full linear counter ($FF)
- Noise: Length counter halt, constant volume 15

## License

Same as LumiEmu project: LGPL-3.0
