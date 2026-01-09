//! Common types for emulators

use bitflags::bitflags;

bitflags! {
    /// NES controller button flags
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct Button: u8 {
        const A      = 0b0000_0001;
        const B      = 0b0000_0010;
        const SELECT = 0b0000_0100;
        const START  = 0b0000_1000;
        const UP     = 0b0001_0000;
        const DOWN   = 0b0010_0000;
        const LEFT   = 0b0100_0000;
        const RIGHT  = 0b1000_0000;
    }
}

/// Controller state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ControllerState {
    pub buttons: Button,
}

impl ControllerState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_pressed(&self, button: Button) -> bool {
        self.buttons.contains(button)
    }

    pub fn press(&mut self, button: Button) {
        self.buttons.insert(button);
    }

    pub fn release(&mut self, button: Button) {
        self.buttons.remove(button);
    }

    pub fn set(&mut self, button: Button, pressed: bool) {
        if pressed {
            self.press(button);
        } else {
            self.release(button);
        }
    }
}

/// NES controller hardware (handles shift register)
#[derive(Debug, Clone)]
pub struct Controller {
    /// Current button state
    state: ControllerState,
    /// Shift register (buttons are read one at a time)
    shift_register: u8,
    /// Strobe mode (if true, continuously reload shift register)
    strobe: bool,
}

impl Controller {
    pub fn new() -> Self {
        Self {
            state: ControllerState::new(),
            shift_register: 0,
            strobe: false,
        }
    }

    /// Write to $4016 (strobe)
    pub fn write(&mut self, value: u8) {
        let new_strobe = (value & 1) != 0;
        
        // Strobe falling edge: latch button states into shift register
        if self.strobe && !new_strobe {
            self.shift_register = self.state.buttons.bits();
        }
        
        self.strobe = new_strobe;
    }

    /// Read from $4016 (shift out one button state)
    pub fn read(&mut self) -> u8 {
        if self.strobe {
            // While strobing, always return A button state
            self.state.buttons.bits() & 1
        } else {
            // Return lowest bit and shift right
            let result = self.shift_register & 1;
            self.shift_register >>= 1;
            // Set bit 7 after shifting (open bus behavior)
            self.shift_register |= 0x80;
            result
        }
    }

    /// Get current controller state (for external modification)
    pub fn state(&mut self) -> &mut ControllerState {
        &mut self.state
    }

    /// Get immutable controller state
    pub fn state_ref(&self) -> &ControllerState {
        &self.state
    }
}

impl Default for Controller {
    fn default() -> Self {
        Self::new()
    }
}
