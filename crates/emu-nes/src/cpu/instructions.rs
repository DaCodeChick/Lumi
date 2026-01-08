//! 6502 instruction implementations

use super::{Cpu6502, CpuMemory, StatusFlags};
use emu_core::Result;

impl<M: CpuMemory> Cpu6502<M> {
    /// Execute an instruction given its opcode
    /// Returns the number of cycles consumed
    pub(super) fn execute(&mut self, opcode: u8) -> Result<u8> {
        // This will be filled in with actual instruction implementations
        // For now, return an error for all opcodes
        Err(emu_core::EmulatorError::InvalidOpcode(opcode))
    }
}
