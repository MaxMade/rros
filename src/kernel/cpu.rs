//! Convienient helper to access/modify CPU state.

use core::arch::asm;
use core::fmt::Display;

/// Abstraction of hard ID.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct HartID(u64);

impl HartID {
    /// Create HartID from raw value.
    pub const fn new(value: u64) -> Self {
        Self { 0: value }
    }

    /// Get raw inner value.
    pub const fn raw(self) -> u64 {
        self.0
    }
}

impl Display for HartID {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:#018x}", self.0)
    }
}

/// Abstraction of `tp` (thread pointer) register.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ThreadPointer(u64);

impl ThreadPointer {
    // Create zeroed abstraction of `tp` register.
    pub fn new(value: u64) -> Self {
        Self { 0: value }
    }

    /// Load current value from `tp` register.
    pub fn read(&mut self) {
        let mut x: u64 = 0;
        unsafe {
            asm!(
                "mv {x}, tp",
                x = out(reg) _,
            );
        }
        self.0 = x;
    }

    /// Store current value to `tp` register.
    pub fn write(&self) {
        let x: u64 = self.0;
        unsafe {
            asm!(
                "mv tp, {x}",
                x = in(reg) x,
            );
        }
    }

    /// Get raw inner value.
    pub const fn raw(self) -> u64 {
        self.0
    }
}
