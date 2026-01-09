# NES Emulator Examples

This directory contains example programs demonstrating the LumiEmu NES emulator capabilities.

## Running Examples

```bash
# From the project root
cargo run --example <example_name> -p emu-nes
```

## Available Examples

### CPU Examples

#### `cpu_demo.rs`
Demonstrates basic 6502 CPU functionality with a simple program that performs arithmetic and memory operations.

```bash
cargo run --example cpu_demo -p emu-nes
```

Shows:
- Loading a program into memory
- Step-by-step CPU execution
- Register state inspection
- Memory read/write operations

---

### System Integration Examples

#### `nes_system_demo.rs`
Demonstrates the complete NES system including CPU, PPU, and memory working together.

```bash
cargo run --example nes_system_demo -p emu-nes
```

Shows:
- Creating a complete NES system
- Loading programs
- Running the emulator
- CPU-PPU synchronization

---

### Testing Examples

#### `generate_test_rom.rs`
Generates a comprehensive CPU test ROM that validates the 6502 implementation.

```bash
cargo run --example generate_test_rom -p emu-nes
```

Creates `test.nes` - a ROM that tests:
- Arithmetic operations (ADC, SBC)
- Logical operations (AND, OR, EOR)
- Loops and branches
- Stack operations
- Subroutines (JSR/RTS)

#### `run_test_rom.rs`
Runs the CPU test ROM and validates all operations completed successfully.

```bash
# First generate the test ROM
cargo run --example generate_test_rom -p emu-nes

# Then run it
cargo run --example run_test_rom -p emu-nes
```

Validates that the CPU implementation is correct by checking memory values after execution.

---

### Graphics/Rendering Examples

#### `generate_perfect_visual.rs`
Generates a visual test ROM that demonstrates PPU rendering with multiple tiles and colors.

```bash
cargo run --example generate_perfect_visual -p emu-nes
```

Creates `perfect_visual.nes` - a ROM that:
- Loads a color palette (Black, Red, Green, Blue)
- Fills the nametable with tile patterns
- Displays CHR-ROM tiles:
  - Tile 1: Solid pattern
  - Tile 2: Checkerboard pattern
  - Tile 3: Horizontal stripes
- Enables background rendering

#### `render_perfect.rs`
Runs the visual test ROM and outputs a rendered frame as a PPM image.

```bash
# First generate the visual ROM
cargo run --example generate_perfect_visual -p emu-nes

# Then render it
cargo run --example render_perfect -p emu-nes
```

Creates `perfect_output.ppm` - a 256×240 image showing:
- Multiple colors from the NES palette
- Different tile patterns rendered correctly
- Background rendering in action

You can view the PPM file with:
- Image viewers: GIMP, Photoshop, any viewer supporting PPM
- Convert to PNG: `convert perfect_output.ppm perfect_output.png`
- Terminal viewers: `feh perfect_output.ppm`

---

### Controller/Input Examples

#### `generate_controller_test.rs`
Generates a controller test ROM that reads button input and stores states in memory.

```bash
cargo run --example generate_controller_test -p emu-nes
```

Creates `controller_test.nes` - a ROM that:
- Strobes the controller ($4016)
- Reads 8 button states sequentially
- Stores each button in memory ($00-$07):
  - $00 = A button
  - $01 = B button
  - $02 = Select
  - $03 = Start
  - $04 = Up
  - $05 = Down
  - $06 = Left
  - $07 = Right
- Sets success flag at $10

#### `controller_demo.rs`
Demonstrates controller input functionality by pressing buttons and verifying the ROM reads them correctly.

```bash
# First generate the controller test ROM
cargo run --example generate_controller_test -p emu-nes

# Then run the demo
cargo run --example controller_demo -p emu-nes
```

Shows:
- Setting button states using the API
- Running a ROM that reads controller input
- Verifying correct button detection
- Controller shift register behavior

---

## Example Output

### CPU Test ROM
```
Arithmetic Test: PASS
Loop Test: PASS
Stack Test: PASS
Subroutine Test: PASS
All tests passed!
```

### Visual Test ROM
```
Running perfect visual test...

Nametable first 16 tiles: 1 2 3 1 2 3 1 2 3 1 2 3 1 2 3 1 

Rendered to: perfect_output.ppm

Colors:
  $16 RGB(152, 34, 32):   4096 pixels  (Red)
  $12 RGB( 48, 50,236):   4096 pixels  (Blue)
  $1A RGB(  8,124,  0):   4096 pixels  (Green)
  $0F RGB(  0,  0,  0):  49152 pixels  (Black)
```

### Controller Demo
```
=== NES Controller Demo ===

Loading controller_test.nes...
Setting button states:
  - Pressing A button
  - Pressing Start button
  - Pressing Up button

Running ROM to read controller input...
Success flag detected after 55 instructions

Button states read from memory:
  $00 (     A) = 1 [pressed] ✓
  $01 (     B) = 0 [released] ✓
  $02 (Select) = 0 [released] ✓
  $03 ( Start) = 1 [pressed] ✓
  $04 (    Up) = 1 [pressed] ✓
  $05 (  Down) = 0 [released] ✓
  $06 (  Left) = 0 [released] ✓
  $07 ( Right) = 0 [released] ✓

SUCCESS: All button states are correct!
```

## Development Workflow

1. **Test CPU functionality**: Run `cpu_demo` and `run_test_rom`
2. **Test controller input**: Run `generate_controller_test` and `controller_demo`
3. **Generate visual content**: Run `generate_perfect_visual`
4. **Verify rendering**: Run `render_perfect` and inspect the output image
5. **Iterate**: Modify examples or add new ones to test specific features

## Notes

- All examples use the `emu-nes` crate's public API
- ROM files are generated in the project root directory
- Output images (`.ppm` files) are also created in the project root
- Examples demonstrate best practices for using the emulator
