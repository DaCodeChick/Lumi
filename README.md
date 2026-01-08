# LumiEmu

**AI-Powered Multi-System Game Emulator with Memory Discovery**

LumiEmu is a video game emulator that uses reinforcement learning to play games and automatically discover how memory works. As the AI plays, it documents memory patterns, correlates inputs with memory changes, and generates human-readable reports about its discoveries.

## Features

- **NES Emulation**: Accurate Nintendo Entertainment System emulation
  - 6502 CPU emulation
  - PPU (graphics) with background and sprite rendering
  - APU (audio) with all 5 sound channels
  - Mapper support (NROM, with more planned)

- **AI-Driven Memory Analysis**: 
  - Reinforcement learning agent that explores games
  - Automatic discovery of memory patterns (counters, flags, positions)
  - Correlation analysis (inputs → memory → outcomes)
  - Semantic labeling with confidence scores

- **Hybrid ML Backend**:
  - Pure Rust implementation using Burn (GPU-accelerated via Vulkan/WGPU)
  - Python bridge support (planned) for PyTorch-based agents
  - Automatic GPU detection with CPU fallback

- **Automated Documentation**:
  - Generates markdown reports of discoveries
  - Structured data embedded in reports
  - Tracks hypothesis evolution over time
  - Append mode for continuous learning

- **Native UI**: Built with Slint for high performance
  - Real-time emulator display
  - Memory viewer with annotations
  - Training metrics visualization
  - Live feedback document preview

## Project Status

🚧 **Currently in early development** - Phase 1 in progress

See [PLAN.md](PLAN.md) for the complete implementation roadmap.

## Building from Source

### Prerequisites

- Rust (stable, latest version)
- Vulkan drivers (for GPU acceleration)
- NVIDIA RTX GPU recommended (tested on RTX 4070)

### Build

```bash
# Clone the repository
git clone https://github.com/DaCodeChick/Lumi.git
cd Lumi

# Build the project
cargo build --release

# Run the emulator
cargo run --release -- --rom ./roms/your-game.nes
```

## Usage

### Playing Games Manually

```bash
lumiemu --rom ./roms/game.nes --config config/default.yaml
```

### Training AI on a Game

```bash
lumiemu --rom ./roms/game.nes --preset exploration
```

### Configuration

See `config/default.yaml` for all available options. Training presets available:
- `exploration.yaml`: High curiosity, prioritizes discovering memory patterns
- `performance.yaml`: Focuses on maximizing game score
- `hybrid.yaml`: Balanced approach

## Architecture

LumiEmu is built as a Cargo workspace with multiple crates:

- `emu-core`: Core emulator traits and abstractions
- `emu-nes`: NES emulator implementation
- `audio-output`: Audio playback (cpal-based)
- `memory-analyzer`: Memory tracking and pattern detection
- `ai-agent`: Machine learning agent (Burn-based)
- `feedback-writer`: Markdown report generation
- `input-handler`: Keyboard/gamepad input management
- `ui`: Slint-based user interface
- `lumiemu`: Main application binary

See [PLAN.md](PLAN.md) for detailed architecture documentation.

## Example Output

The AI generates reports like this:

```markdown
## Player State (0x0700-0x074F)

### Mario Position
- **Address**: `0x0700` (X position, screen-relative)
- **Data Type**: u8
- **Range Observed**: 0-255
- **Causality**: Pressing RIGHT increments 0x0700, LEFT decrements
- **Confidence**: 0.92 (450 observations)
```

## Roadmap

- [x] Project planning and architecture design
- [ ] Phase 1: Core NES CPU emulation
- [ ] Phase 2: PPU and memory system
- [ ] Phase 3: Input handling and basic UI
- [ ] Phase 4: Audio emulation
- [ ] Phase 5: Memory instrumentation
- [ ] Phase 6: Feedback generation
- [ ] Phase 7-8: RL agent training
- [ ] Phase 9: Semantic memory discovery
- [ ] Phase 10-14: UI enhancements, polish, cross-platform support
- [ ] Future: SNES support, Python ML bridge, corruption testing

See [PLAN.md](PLAN.md) for the complete 14-week implementation plan.

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

This project is in early development. Core features are still being implemented.

## License

LumiEmu is licensed under the [GNU Lesser General Public License v3.0](LICENSE).

## Acknowledgments

- NESDev community for comprehensive NES documentation
- Burn framework for ML infrastructure
- Slint for the UI framework

## Resources

- [NESDev Wiki](https://www.nesdev.org/wiki/Nesdev_Wiki)
- [Burn Framework](https://burn.dev/)
- [Slint UI](https://slint.dev/)

## Contact

GitHub: [@DaCodeChick](https://github.com/DaCodeChick)
