//! Memory bus with instrumentation hooks for AI observation

/// Type of memory access
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessType {
    Read,
    Write,
}

/// Context information for a memory access
#[derive(Debug, Clone, Copy)]
pub struct EmulatorContext {
    /// Current frame number
    pub frame: u64,
    /// Current cycle count
    pub cycle: u64,
    /// Program counter at time of access
    pub pc: u16,
    /// Last controller input state
    pub last_input: u8,
}

/// Information about a memory access
#[derive(Debug, Clone, Copy)]
pub struct MemoryAccess {
    /// Memory address accessed
    pub address: u16,
    /// Value read or written
    pub value: u8,
    /// Type of access
    pub access_type: AccessType,
    /// Emulator context at time of access
    pub context: EmulatorContext,
    /// For writes: the old value before writing
    pub old_value: Option<u8>,
}

/// Observer trait for monitoring memory accesses
///
/// This allows the AI/memory analyzer to observe every memory
/// read and write for pattern detection and semantic discovery
pub trait MemoryObserver: Send + Sync {
    /// Called when memory is read
    fn on_read(&mut self, address: u16, value: u8, context: &EmulatorContext);

    /// Called when memory is written
    fn on_write(&mut self, address: u16, old_value: u8, new_value: u8, context: &EmulatorContext);

    /// Called at the end of each frame
    fn on_frame_end(&mut self, frame: u64) {
        let _ = frame; // Default implementation does nothing
    }
}

/// Memory bus trait with observer support
pub trait MemoryBus {
    /// Read a byte from memory
    fn read(&mut self, addr: u16) -> u8;

    /// Write a byte to memory
    fn write(&mut self, addr: u16, value: u8);

    /// Read a 16-bit word (little-endian)
    fn read_word(&mut self, addr: u16) -> u16 {
        let lo = self.read(addr) as u16;
        let hi = self.read(addr.wrapping_add(1)) as u16;
        (hi << 8) | lo
    }

    /// Write a 16-bit word (little-endian)
    fn write_word(&mut self, addr: u16, value: u16) {
        let lo = (value & 0xFF) as u8;
        let hi = ((value >> 8) & 0xFF) as u8;
        self.write(addr, lo);
        self.write(addr.wrapping_add(1), hi);
    }

    /// Attach a memory observer
    fn attach_observer(&mut self, observer: Box<dyn MemoryObserver>);

    /// Remove all observers
    fn clear_observers(&mut self);

    /// Get the current emulator context (for observers)
    fn context(&self) -> EmulatorContext;

    /// Update the emulator context
    fn update_context(&mut self, context: EmulatorContext);
}

/// A simple no-op observer for testing
pub struct NoOpObserver;

impl MemoryObserver for NoOpObserver {
    fn on_read(&mut self, _address: u16, _value: u8, _context: &EmulatorContext) {}
    fn on_write(&mut self, _address: u16, _old_value: u8, _new_value: u8, _context: &EmulatorContext) {}
}
