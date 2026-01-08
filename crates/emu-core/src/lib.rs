//! Core emulator traits and types for LumiEmu
//!
//! This crate provides the fundamental abstractions for building emulators,
//! including memory bus interfaces, CPU traits, and instrumentation hooks
//! for AI-driven memory analysis.

pub mod error;
pub mod memory_bus;
pub mod traits;
pub mod types;

pub use error::{EmulatorError, Result};
pub use memory_bus::{MemoryBus, MemoryObserver, MemoryAccess, AccessType, EmulatorContext};
pub use traits::{Cpu, Emulator};
pub use types::{Button, ControllerState};
