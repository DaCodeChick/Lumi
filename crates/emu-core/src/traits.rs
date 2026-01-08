//! Core traits for emulators

use crate::Result;

/// Core CPU trait
pub trait Cpu {
    /// Reset the CPU to its initial state
    fn reset(&mut self);

    /// Execute a single instruction
    /// Returns the number of cycles consumed
    fn step(&mut self) -> Result<u8>;

    /// Get the program counter
    fn pc(&self) -> u16;

    /// Get the stack pointer
    fn sp(&self) -> u8;

    /// Get the accumulator
    fn a(&self) -> u8;

    /// Get the X register
    fn x(&self) -> u8;

    /// Get the Y register
    fn y(&self) -> u8;

    /// Get the status flags
    fn status(&self) -> u8;
}

/// Core emulator trait
pub trait Emulator {
    /// Reset the emulator to its initial state
    fn reset(&mut self);

    /// Run one frame of emulation
    /// Returns the number of cycles executed
    fn run_frame(&mut self) -> Result<usize>;

    /// Check if the emulator is paused
    fn is_paused(&self) -> bool;

    /// Pause or unpause the emulator
    fn set_paused(&mut self, paused: bool);
}
