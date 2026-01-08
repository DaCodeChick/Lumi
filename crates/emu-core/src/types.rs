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
