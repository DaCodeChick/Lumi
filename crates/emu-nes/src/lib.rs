//! NES Emulator Implementation
//!
//! This crate implements a Nintendo Entertainment System emulator,
//! including the 6502 CPU, PPU, APU, and memory system.

pub mod cpu;
pub mod memory;

pub use cpu::Cpu6502;
pub use memory::NesMemory;
