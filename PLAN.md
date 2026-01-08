# LumiEmu - Project Implementation Plan

## Project Overview

**LumiEmu** is a multi-system video game emulator with AI-driven memory analysis capabilities. The AI "pseudo-debugs" games by playing them and documenting memory patterns through discovery, generating human-readable feedback documents about how memory is read and written during gameplay.

### Core Concept

The AI plays games like Super Mario Bros and documents its discoveries in a debugging-style manner:
- Identifies sprite data locations
- Tracks memory reads/writes
- Discovers game logic patterns
- Correlates inputs with memory changes
- Generates markdown reports of findings

### Repository Information

- **Location**: `/home/admin/Documents/GitHub/Lumi/`
- **Remote**: `https://github.com/DaCodeChick/Lumi.git`
- **License**: LGPL-3.0
- **Primary Platform**: Linux (Ubuntu/Debian with NVIDIA RTX 4070)
- **Target Platforms**: Linux → Windows → macOS

## Technology Stack

### Core Technologies
- **Language**: Rust (stable, latest)
- **Build System**: Cargo workspace (multi-crate architecture)

### Emulation
- Custom NES emulator (6502 CPU + PPU + APU)
- Initial support: Mapper 0 (NROM) - supports Super Mario Bros
- Future: Mapper 1 (MMC1), Mapper 4 (MMC3)
- Audio output: `cpal` 0.15

### ML/AI (Hybrid Approach)
- **Primary (Phase 1)**: Pure Rust
  - `burn` 0.14 with `burn-wgpu` (Vulkan backend for RTX 4070)
  - Auto-detect GPU, fallback to `burn-ndarray` (CPU)
- **Future (Phase 15)**: Python bridge via PyO3
  - `pyo3` 0.21 for Python agent interface
  - `burn-tch` (CUDA backend) for potential performance gains
  - Support for PyTorch-based agents

### UI
- `slint` 1.5 (native, performant UI framework)
- Real-time emulator display (256x240, scaled)
- Memory viewer with annotations
- Training metrics visualization
- Live feedback document preview

### Input
- `gilrs` 0.10 (gamepad/controller support)
- `winit` 0.29 (keyboard/window events)
- Input recording/playback for testing

### Utilities
- `serde` + `serde_yaml` (configuration management)
- `clap` 4.5 (CLI argument parsing)
- `tracing` + `tracing-subscriber` (logging/diagnostics)
- `anyhow` + `thiserror` (error handling)

## Project Structure

```
/home/admin/Documents/GitHub/Lumi/
├── .git/
├── .gitignore
├── Cargo.toml                      # Workspace root
├── rust-toolchain.toml             # Pin Rust version
├── LICENSE                         # LGPL-3.0
├── README.md                       # Project overview
├── PLAN.md                         # This file
├── ARCHITECTURE.md                 # Technical architecture details
│
├── config/                         # Configuration files
│   ├── default.yaml                # Default settings
│   └── training_presets/
│       ├── exploration.yaml        # Curiosity-driven learning
│       ├── performance.yaml        # Score-maximizing
│       └── hybrid.yaml             # Balanced approach
│
├── crates/                         # Workspace crates
│   ├── emu-core/                   # Core emulator abstractions
│   │   └── src/
│   │       ├── traits.rs           # Emulator, Memory, CPU, PPU traits
│   │       ├── memory_bus.rs       # Memory access with instrumentation hooks
│   │       ├── input.rs            # Input abstraction
│   │       └── types.rs            # Common types (Button, Color, etc.)
│   │
│   ├── emu-nes/                    # NES emulator implementation
│   │   └── src/
│   │       ├── nes.rs              # Main NES system struct
│   │       ├── cpu.rs              # 6502 CPU emulation
│   │       ├── ppu.rs              # Picture Processing Unit
│   │       ├── apu.rs              # Audio Processing Unit
│   │       ├── memory.rs           # NES memory map
│   │       ├── cartridge.rs        # ROM loading (iNES format)
│   │       ├── mappers/            # Cartridge mapper implementations
│   │       │   ├── mapper.rs       # Mapper trait
│   │       │   ├── mapper000.rs    # NROM (Super Mario Bros)
│   │       │   └── mapper001.rs    # MMC1 (Zelda, Metroid)
│   │       └── instructions/       # CPU instruction implementations
│   │
│   ├── audio-output/               # Audio playback
│   │   └── src/
│   │       ├── mixer.rs            # Mix audio channels
│   │       ├── output.rs           # cpal output stream
│   │       ├── resampler.rs        # Sample rate conversion
│   │       └── buffer.rs           # Ring buffer for audio
│   │
│   ├── memory-analyzer/            # Memory tracking & analysis
│   │   └── src/
│   │       ├── tracker.rs          # Track all memory accesses
│   │       ├── differ.rs           # Snapshot comparison
│   │       ├── pattern.rs          # Detect patterns (counters, flags)
│   │       ├── correlator.rs       # Input→Memory correlation
│   │       ├── semantic.rs         # Build semantic understanding
│   │       └── hypothesis.rs       # Generate/test hypotheses
│   │
│   ├── ai-agent/                   # Hybrid ML agent system
│   │   ├── python/                 # Python ML code (Phase 15+)
│   │   │   ├── requirements.txt    # PyTorch, numpy, etc.
│   │   │   ├── agent.py            # Base agent interface
│   │   │   └── dqn_agent.py        # Example DQN implementation
│   │   └── src/
│   │       ├── trait.rs            # GameAgent trait
│   │       ├── backend.rs          # Backend selection logic
│   │       ├── rust/               # Pure Rust implementation (Phase 7-8)
│   │       │   ├── agent.rs        # RustAgent
│   │       │   ├── dqn.rs          # DQN network in Burn
│   │       │   ├── trainer.rs      # Training loop
│   │       │   ├── replay_buffer.rs # Experience replay
│   │       │   └── curiosity.rs    # Curiosity module
│   │       └── python/             # Python bridge (Phase 15+)
│   │           ├── agent.rs        # PythonAgent
│   │           ├── bridge.rs       # PyO3 bridge
│   │           └── converter.rs    # State conversion
│   │
│   ├── feedback-writer/            # Documentation generation
│   │   └── src/
│   │       ├── markdown.rs         # Markdown generation
│   │       ├── template.rs         # Report templates
│   │       ├── append.rs           # Incremental updates
│   │       ├── formatter.rs        # Format discoveries
│   │       └── serializer.rs       # JSON/YAML embedding
│   │
│   ├── input-handler/              # Input management
│   │   └── src/
│   │       ├── keyboard.rs         # Keyboard mapping
│   │       ├── gamepad.rs          # Controller (gilrs)
│   │       ├── source.rs           # Input source switching
│   │       ├── recorder.rs         # Record/replay inputs
│   │       └── mapper.rs           # Key/button → NES button mapping
│   │
│   └── ui/                         # Slint UI
│       ├── build.rs
│       ├── ui/                     # Slint files
│       │   ├── app.slint           # Main window
│       │   └── components/
│       │       ├── emulator_display.slint
│       │       ├── ai_status.slint
│       │       ├── memory_viewer.slint
│       │       └── control_panel.slint
│       └── src/
│           ├── bridge.rs           # Rust ↔ Slint data bridge
│           ├── renderer.rs         # Convert PPU buffer to Slint image
│           └── callbacks.rs        # UI event handlers
│
├── lumiemu/                        # Main application binary
│   └── src/
│       ├── main.rs
│       ├── app.rs                  # Application state
│       ├── orchestrator.rs         # Coordinates all components
│       ├── config.rs               # Config loading/management
│       └── cli.rs                  # Command-line arguments
│
├── docs/                           # Documentation
│   ├── architecture.md
│   ├── nes-implementation.md
│   ├── ai-approach.md
│   ├── memory-analysis.md
│   ├── user-guide.md
│   └── extending.md                # How to add new systems
│
├── examples/                       # Example programs
│   ├── manual_play.rs              # Play games manually
│   ├── ai_training.rs              # Train AI on a game
│   └── memory_exploration.rs       # Explore memory without ML
│
├── tests/                          # Integration tests
│   ├── nes_cpu_tests.rs            # CPU instruction tests
│   ├── ppu_tests.rs
│   └── integration_tests.rs
│
└── roms/                           # ROM files (gitignored)
    ├── .gitkeep
    └── Super Mario Bros (World).nes  # Mario/Duck Hunt NTSC ROM
```

## NES Architecture Reference

### Memory Map
- `0x0000-0x07FF`: 2KB internal RAM
- `0x0800-0x1FFF`: Mirrors of RAM
- `0x2000-0x2007`: PPU registers (mirrored to 0x3FFF)
- `0x4000-0x4017`: APU & I/O registers
- `0x4020-0xFFFF`: Cartridge space (ROM, RAM, mapper registers)

### CPU: MOS 6502
- 8-bit processor, ~1.79 MHz (NTSC)
- Registers: A, X, Y, SP, PC, Status
- ~56 official opcodes
- Cycle-accurate timing required

### PPU: Picture Processing Unit
- Generates NTSC video signal
- 256x240 resolution
- 341 PPU cycles per scanline
- 262 scanlines per frame
- 60 Hz refresh rate

### APU: Audio Processing Unit
- 2 pulse wave channels (square waves)
- 1 triangle wave channel
- 1 noise channel
- 1 DMC channel (delta modulation)
- Sample rate: 44.1kHz or 48kHz

### Controllers
- 8-bit shift register
- Buttons: A, B, Select, Start, Up, Down, Left, Right

## ML/AI Architecture

### Agent Goal: Pseudo-Debugging

The AI's primary objective is **exploration and discovery**, not necessarily winning:
- Play the game and observe memory changes
- Correlate inputs with memory modifications
- Identify patterns (counters, flags, positions, etc.)
- Generate hypotheses about memory semantics
- Document findings in human-readable reports

### State Representation

The AI observes:
1. **Visual state**: PPU frame buffer (256x240x3 RGB)
   - Processed by CNN (convolutional neural network)
   - Can be downsampled for efficiency
2. **Memory state**: Full RAM (2KB for NES)
   - Processed by MLP (multi-layer perceptron)
3. **Memory deltas**: Changes since last frame
   - Highlights what's actively changing
4. **Discovered patterns**: Previously identified addresses
   - Provides context for learning

### Action Space

NES controller: 8 buttons = 2^8 = 256 possible combinations
- In practice, many combinations don't make sense (e.g., Up+Down)
- Can be simplified or left as-is for exploration

### Reward Function (Dual Objectives)

1. **Intrinsic (Curiosity)**: Reward for discovering new memory patterns
   - Novel memory states
   - New correlations found
   - Hypotheses validated
   - Weight: 0.7 (configurable)

2. **Extrinsic (Performance)**: Reward for game progress
   - Score increases
   - Level completion
   - Survival time
   - Weight: 0.3 (configurable)

### DQN Architecture (Deep Q-Network)

```
Input State
    ↓
┌─────────────────┐  ┌──────────────────┐
│ Visual Encoder  │  │ Memory Encoder   │
│ (CNN)           │  │ (MLP)            │
│ Conv2D x3       │  │ Dense x2         │
│ + Pooling       │  │ + Normalization  │
└─────────────────┘  └──────────────────┘
         ↓                    ↓
         └────────┬───────────┘
                  ↓
          ┌──────────────┐
          │ Fusion Layer │
          │ (Dense)      │
          └──────────────┘
                  ↓
          ┌──────────────┐
          │ Output Layer │
          │ Q-values x256│
          └──────────────┘
                  ↓
         Action Selection
       (ε-greedy or softmax)
```

### Curiosity Module

Predict memory changes to encourage exploration:
- Forward model: `predict_memory(state, action) → next_memory`
- Prediction error = curiosity reward
- Higher error = more novel = higher reward

### Training Loop

1. **Online Training** (default):
   - Train while playing
   - Update network every N frames
   - Continuous learning

2. **Offline Training** (alternative):
   - Collect data first (exploration phase)
   - Train on collected data (training phase)
   - Separate exploration and learning

### Hybrid Backend Design

```rust
pub trait GameAgent: Send + Sync {
    fn select_action(&mut self, state: &GameState) -> Action;
    fn observe(&mut self, transition: Transition);
    fn train(&mut self) -> TrainingMetrics;
    fn save(&self, path: &Path) -> Result<()>;
    fn load(&mut self, path: &Path) -> Result<()>;
}

// Implementation 1: Pure Rust (Burn)
pub struct RustAgent<B: Backend> { ... }

// Implementation 2: Python bridge (PyO3)
pub struct PythonAgent { ... }
```

Both implementations share the same interface, selectable via configuration.

## Memory Analysis System

### Tracking Phase

Every memory access is recorded with context:
```rust
struct MemoryAccess {
    address: u16,
    value: u8,
    access_type: Read | Write,
    frame: u64,
    cycle: u64,
    pc: u16,           // Program counter
    last_input: u8,    // Controller state
}
```

### Pattern Detection

Automatic detection of:
- **Counters**: Values that increment/decrement consistently
- **Flags**: Binary toggles (0/1)
- **Bounded values**: Health bars, timers (stay within range)
- **Constants**: ROM reads (never change)
- **Pointers**: Values that look like addresses

### Correlation Analysis

1. **Input → Memory**: Which buttons affect which addresses?
   ```
   Pressing RIGHT → 0x0700 increments (player X position)
   Pressing A → 0x0704 changes (jump state)
   ```

2. **Memory → Outcome**: Which addresses predict events?
   ```
   0x075A == 0 → Game Over
   0x07E0 increases → Score increased
   ```

### Semantic Labeling

Generate human-readable names for addresses:
```rust
struct SemanticLabel {
    address: u16,
    label: String,        // e.g., "player_x_position"
    confidence: f32,      // 0.0 to 1.0
    observations: u32,    // Evidence count
    hypothesis: String,   // Human-readable description
}
```

### Hypothesis Generation

Example discoveries for Super Mario Bros:
- `0x0700`: "Player X position (screen-relative)" - confidence: 0.95
- `0x0710`: "Player Y position" - confidence: 0.93
- `0x075A`: "Lives remaining" - confidence: 0.88
- `0x07E0-0x07E5`: "Score (BCD encoded)" - confidence: 0.92

## Feedback Document Format

### Structure: Markdown + Embedded JSON

```markdown
---
session_id: "nes-mario-001"
game: "Super Mario Bros"
rom_hash: "sha256:..."
training_iteration: 142
timestamp: "2026-01-08T10:30:00Z"
---

# Memory Discovery Report - Iteration 142

## Player State (0x0700-0x074F)

### Mario Position
- **Address**: `0x0700` (X position, screen-relative)
- **Address**: `0x0710` (Y position)
- **Data Type**: u8
- **Range Observed**: 0-255
- **Causality**: Pressing RIGHT increments 0x0700, LEFT decrements

**Structured Data:**
\```json
{
  "address": "0x0700",
  "semantic_name": "mario_x_position_screen",
  "confidence": 0.92,
  "observations": 450,
  "correlations": {
    "input_right": {"effect": "increment", "strength": 0.98},
    "input_left": {"effect": "decrement", "strength": 0.98}
  }
}
\```

## Hypothesis: Collision Detection

Observed pattern at `0x0726`:
- Value changes from 0x00 to 0x01 when Mario Y position ≈ 0xA0 
  AND sprite at 0x0200 Y position matches
- **Hypothesis**: Collision flag
- **Confidence**: 0.67 (needs more testing)
- **Next Steps**: Trigger collision scenarios deliberately
```

### Append Mode

- Reports are updated incrementally
- New discoveries are appended
- Confidence scores are updated as evidence grows
- Hypothesis status tracked over time

## Configuration System

### Configuration File Format (YAML)

```yaml
emulator:
  system: "nes"
  speed_multiplier: 1         # 1x = real-time, 10x = fast, 100x = very fast
  headless: false             # Show UI while training
  audio_enabled: true

ai:
  backend: "rust"             # "rust" or "python"
  
  rust:
    device: "wgpu"            # "ndarray" (CPU) or "wgpu" (GPU)
    model_path: null          # Load pretrained model
    
  python:
    module: "dqn_agent"       # Python module name
    class: "DQNAgent"         # Python class name
    model_path: null
    
  training:
    mode: "online"            # "online" or "offline"
    exploration_weight: 0.7   # Balance curiosity vs performance
    epsilon_start: 1.0
    epsilon_end: 0.1
    epsilon_decay_steps: 100000
    batch_size: 32
    learning_rate: 0.0001
    gamma: 0.99
    replay_buffer_size: 100000
    target_update_frequency: 1000
    
  observation:
    include_visual: true
    visual_downsample: 2      # Downsample to 120x128
    include_memory: true
    include_memory_delta: true
    frame_stack: 4            # Stack last N frames

memory_analysis:
  enabled: true
  snapshot_frequency: 1       # Every N frames
  correlation_threshold: 0.8  # Min confidence for discoveries
  pattern_detection: true
  hypothesis_generation: true

feedback:
  enabled: true
  format: "markdown"          # "markdown" or "json"
  save_interval: 100          # Episodes between saves
  output_dir: "./discoveries"
  append_mode: true
  include_structured_data: true

input:
  source: "ai"                # "human", "ai", or "playback"
  recording_enabled: false
  recording_path: "./recordings"

rom:
  path: "./roms/Super Mario Bros (World).nes"
  auto_detect_mapper: true

logging:
  level: "info"               # "trace", "debug", "info", "warn", "error"
  file: "./logs/training.log"
```

### Training Presets

**Exploration** (`config/training_presets/exploration.yaml`):
- High curiosity weight (0.9)
- High epsilon (slow decay)
- Prioritizes discovering new memory patterns

**Performance** (`config/training_presets/performance.yaml`):
- Low curiosity weight (0.1)
- Fast epsilon decay
- Prioritizes maximizing game score

**Hybrid** (`config/training_presets/hybrid.yaml`):
- Balanced (0.5/0.5)
- Moderate epsilon decay
- Balances discovery and performance

## Implementation Phases

### Phase 1: Core NES CPU (Week 1)
**Goal**: Working 6502 CPU emulator

**Tasks**:
1. Workspace setup (Cargo.toml, LICENSE, README)
2. emu-core crate (traits and interfaces)
3. emu-nes CPU implementation (6502 instruction set)
4. Unit tests (Klaus Dormann test suite)

**Deliverable**: 6502 CPU that passes all instruction tests

---

### Phase 2: NES Memory & PPU Basics (Week 2)
**Goal**: Complete NES system with video output

**Tasks**:
1. NES memory map implementation
2. Cartridge loading (iNES format parser)
3. Mapper 0 (NROM) implementation
4. PPU rendering (background + sprites)
5. Frame buffer generation

**Deliverable**: Can load and display Super Mario Bros (headless)

---

### Phase 3: Input Handling & Basic UI (Week 3)
**Goal**: Playable emulator with GUI

**Tasks**:
1. input-handler crate (keyboard + gamepad)
2. ui crate (Slint UI)
3. lumiemu binary (main application)
4. Controller input integration
5. 60 FPS game loop

**Deliverable**: Playable NES emulator with GUI

---

### Phase 4: Audio Emulation (Week 4)
**Goal**: Full audio output

**Tasks**:
1. APU register handling
2. Audio channel implementations (pulse, triangle, noise, DMC)
3. audio-output crate (cpal integration)
4. Audio/video synchronization

**Deliverable**: Full audio/video emulation

---

### Phase 5: Memory Instrumentation (Week 5)
**Goal**: Track and analyze memory access

**Tasks**:
1. Memory observation hooks
2. memory-analyzer crate (tracking, diffing, patterns)
3. Integration with emulator
4. Export observations to JSON

**Deliverable**: Memory access tracking functional

---

### Phase 6: Feedback System (Week 6)
**Goal**: Generate markdown reports

**Tasks**:
1. feedback-writer crate (markdown generation)
2. Report structure and templates
3. Structured data embedding
4. Append mode implementation

**Deliverable**: Automated markdown report generation

---

### Phase 7: Rust ML Agent - Foundation (Week 7)
**Goal**: Basic DQN agent using Burn

**Tasks**:
1. Burn setup (wgpu + ndarray backends)
2. State representation (frame + memory tensors)
3. DQN network architecture
4. Training infrastructure (replay buffer, target network)
5. GameAgent trait

**Deliverable**: DQN architecture ready for training

---

### Phase 8: Rust ML Agent - Training Loop (Week 8)
**Goal**: Train agent on NES games

**Tasks**:
1. Reward function (curiosity + performance)
2. Curiosity module
3. Training loop implementation
4. Metrics tracking
5. Integration with emulator
6. Configuration system

**Deliverable**: Working RL training pipeline

---

### Phase 9: Memory Correlation & Semantic Discovery (Week 9)
**Goal**: AI discovers memory semantics

**Tasks**:
1. Input correlation analysis
2. Outcome correlation analysis
3. Semantic labeling with confidence scores
4. Hypothesis generation and testing
5. Integration with feedback writer

**Deliverable**: AI-generated semantic memory maps

---

### Phase 10: UI Enhancements (Week 10)
**Goal**: Feature-rich monitoring UI

**Tasks**:
1. AI status panel
2. Memory viewer with annotations
3. Training charts
4. Feedback document preview
5. Advanced controls

**Deliverable**: Comprehensive training monitoring UI

---

### Phase 11: Gamepad Support & Input Recording (Week 11)
**Goal**: Enhanced input handling

**Tasks**:
1. Gamepad support (gilrs integration)
2. Input recording to file
3. Input playback
4. Controller mapping configuration

**Deliverable**: Full gamepad support and recording/playback

---

### Phase 12: Configuration & Presets (Week 12)
**Goal**: User-friendly configuration

**Tasks**:
1. Configuration files and presets
2. Config validation
3. CLI enhancements
4. Documentation

**Deliverable**: Flexible configuration system

---

### Phase 13: Polish & Testing (Week 13)
**Goal**: Production-ready stability

**Tasks**:
1. Performance profiling and optimization
2. Memory leak detection
3. Error handling
4. Integration testing
5. Documentation (README, architecture, user guide)
6. Optional: CI/CD setup

**Deliverable**: Stable, documented LumiEmu

---

### Phase 14: Windows Support (Week 14)
**Goal**: Cross-platform support

**Tasks**:
1. Windows testing
2. Platform-specific fixes
3. Windows packaging
4. Full feature testing on Windows

**Deliverable**: LumiEmu working on Windows

---

### Future Phases (Post-MVP)

**Phase 15**: Python ML Bridge
- PyO3 integration
- Python agent interface
- PyTorch example agent

**Phase 16**: Advanced RL Algorithms
- PPO implementation
- A3C for parallel training

**Phase 17**: SNES Emulation
- emu-snes crate
- 65816 CPU
- SNES PPU

**Phase 18**: Corruption Testing (RTC-style)
- Memory corruption tools
- Audio/visual effects tracking

**Phase 19**: Additional NES Mappers
- Mapper 1 (MMC1)
- Mapper 4 (MMC3)

**Phase 20**: macOS Support
- Testing on Intel and Apple Silicon
- Metal backend

**Phase 21**: GPU Capability Detection
- Auto-detect GPU capabilities
- Automatic backend selection
- Fallback strategies

## Key Milestones

| Phase | Week | Milestone |
|-------|------|-----------|
| 1-2 | 1-2 | Working NES emulator (CPU + PPU + Memory) |
| 3 | 3 | Playable with GUI |
| 4 | 4 | Audio working |
| 5-6 | 5-6 | Memory tracking + Feedback generation |
| 7-8 | 7-8 | RL agent training on NES games |
| 9 | 9 | Semantic memory discovery |
| 10-11 | 10-11 | Feature-complete UI + Input enhancements |
| 12-13 | 12-13 | Production-ready (Linux) |
| 14 | 14 | Windows support |

**Total**: ~14 weeks (~3.5 months) for production-ready system

## Development Guidelines

### Commit Strategy
- Frequent commits with working increments (agile approach)
- Clear, descriptive commit messages
- Each phase milestone gets tagged

### Code Quality
- Comprehensive unit tests for all components
- Integration tests for emulator accuracy
- Use `clippy` for linting
- Use `rustfmt` for consistent formatting
- Document public APIs with rustdoc

### Performance
- Profile regularly (CPU, memory, GPU)
- Optimize hotspaths in emulator core
- Efficient tensor operations in ML code
- Minimize allocations in game loop

### Error Handling
- Use `Result<T, E>` for fallible operations
- Use `anyhow` for application-level errors
- Use `thiserror` for library-level errors
- Provide helpful error messages

## Testing Strategy

### Unit Tests
- Every instruction in CPU
- Memory access patterns
- PPU rendering logic
- APU sound generation
- ML network forward/backward passes

### Integration Tests
- Load and run test ROMs
- Klaus Dormann 6502 functional tests
- NEStress test ROM
- PPU test ROMs (sprite hit, scrolling, etc.)

### End-to-End Tests
- Play through first level of Super Mario Bros
- Verify determinism (same inputs = same outputs)
- Record/replay testing

### Performance Benchmarks
- CPU emulation speed (cycles per second)
- PPU rendering speed (frames per second)
- ML training throughput (steps per second)
- Memory usage over time

## Dependencies Summary

### Workspace Dependencies

```toml
[workspace.dependencies]
# Emulation
bitflags = "2.4"
byteorder = "1.5"

# Audio
cpal = "0.15"

# UI
slint = "1.5"

# Input
gilrs = "0.10"
winit = "0.29"

# ML - Rust
burn = "0.14"
burn-ndarray = "0.14"      # CPU backend
burn-wgpu = "0.14"         # GPU backend (Vulkan)
burn-tch = "0.14"          # Optional: CUDA backend
burn-autodiff = "0.14"

# ML - Python bridge (Phase 15+)
pyo3 = { version = "0.21", features = ["auto-initialize"], optional = true }
numpy = { version = "0.21", optional = true }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
serde_yaml = "0.9"

# Error handling
anyhow = "1.0"
thiserror = "1.0"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# CLI
clap = { version = "4.5", features = ["derive"] }

# Utilities
crossbeam = "0.8"
parking_lot = "0.12"
rand = "0.8"

# Testing
criterion = "0.5"
```

### Python Requirements (Phase 15+)

```txt
torch>=2.0.0
numpy>=1.24.0
gymnasium>=0.29.0
stable-baselines3>=2.0.0
tensorboard>=2.14.0
```

## Resources & References

### NES Development
- [NESDev Wiki](https://www.nesdev.org/wiki/Nesdev_Wiki)
- [6502 Instruction Reference](https://www.masswerk.at/6502/6502_instruction_set.html)
- [NES PPU Documentation](https://www.nesdev.org/wiki/PPU)
- [iNES ROM Format](https://www.nesdev.org/wiki/INES)

### Test ROMs
- Klaus Dormann 6502 Functional Test
- NEStress
- blargg's test ROMs

### Reinforcement Learning
- [Deep Q-Network (DQN) Paper](https://arxiv.org/abs/1312.5602)
- [Curiosity-driven Exploration](https://arxiv.org/abs/1705.05363)
- [Playing Atari with Deep RL](https://arxiv.org/abs/1312.5602)

### Burn Framework
- [Burn Book](https://burn.dev/)
- [Burn GitHub](https://github.com/tracel-ai/burn)
- [Burn Examples](https://github.com/tracel-ai/burn/tree/main/examples)

## Notes

- **ROM**: Mario/Duck Hunt NTSC ROM available at project start
- **GPU**: NVIDIA RTX 4070 with Vulkan support
- **Mapper Priority**: Mapper 0 (NROM) first, then Mapper 1 (MMC1) if time permits
- **Development Flow**: Agile with frequent commits
- **GPU Detection**: Auto-detect, fallback to CPU if unavailable
- **Python Bridge**: Defer to Phase 15 (post-MVP)
- **Corruption Testing**: Defer to Phase 18 (post-MVP)

## Questions / Clarifications

All major architectural decisions have been finalized:
- ✅ Project name: LumiEmu
- ✅ License: LGPL-3.0
- ✅ ML approach: Hybrid (Rust first, Python later)
- ✅ Burn backend: wgpu (Vulkan) with CPU fallback
- ✅ Test ROM: Super Mario Bros (Mario/Duck Hunt NTSC)
- ✅ Platform priority: Linux → Windows → macOS
- ✅ Development approach: Agile, frequent commits

Ready to begin Phase 1 implementation!
