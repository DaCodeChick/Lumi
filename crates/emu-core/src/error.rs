//! Error types for emulator core

use thiserror::Error;

/// Result type for emulator operations
pub type Result<T> = std::result::Result<T, EmulatorError>;

/// Errors that can occur during emulation
#[derive(Error, Debug)]
pub enum EmulatorError {
    #[error("Invalid memory address: 0x{0:04X}")]
    InvalidAddress(u16),

    #[error("Invalid opcode: 0x{0:02X}")]
    InvalidOpcode(u8),

    #[error("ROM loading error: {0}")]
    RomLoadError(String),

    #[error("Unsupported mapper: {0}")]
    UnsupportedMapper(u8),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Emulation error: {0}")]
    Other(String),
}
