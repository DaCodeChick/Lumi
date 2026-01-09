//! NES Emulator Implementation
//!
//! This crate implements a Nintendo Entertainment System emulator,
//! including the 6502 CPU, PPU, APU, and memory system.

pub mod cartridge;
pub mod cpu;
pub mod memory;
pub mod palette;
pub mod ppu;
pub mod system;

pub use cartridge::Cartridge;
pub use cpu::Cpu6502;
pub use memory::NesMemory;
pub use palette::{framebuffer_to_rgb, palette_to_rgb, NES_PALETTE};
pub use ppu::Ppu;
pub use system::NesSystem;
