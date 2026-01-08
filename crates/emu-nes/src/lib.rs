//! NES Emulator Implementation
//!
//! This crate implements a Nintendo Entertainment System emulator,
//! including the 6502 CPU, PPU, APU, and memory system.

pub mod cpu;

pub use cpu::Cpu6502;
