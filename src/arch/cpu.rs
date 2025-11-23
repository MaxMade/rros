//! Convienient helper to access/modify CPU state.

use core::arch::asm;
use core::fmt::Display;

/// Get default page size (`4096` bytes)
pub const fn page_size() -> usize {
    4096
}

/// Generic abstraction of a `Control and Status Register`.
pub trait CSR {
    /// Create a new [`CSR`] from fixed the fixed value `inner`.
    fn new(inner: u64) -> Self
    where
        Self: Sized;

    /// Write current `inner` value back to register.
    fn write(&self);

    /// Read current register value and store it within [`CSR`].
    fn read(&mut self);

    /// Get `inner` value of [`CSR`].
    fn inner(&self) -> u64;
}

impl Display for dyn CSR {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:#018x}", self.inner())
    }
}

/// Abstraction of `tp` (thread pointer) register.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TP(u64);

impl TP {
    /// Create zeroed abstraction of `tp` register.
    pub fn new(value: u64) -> Self {
        Self { 0: value }
    }

    /// Load current value from `tp` register.
    pub fn read(&mut self) {
        let mut x: u64;
        unsafe {
            asm!(
                "mv {x}, tp",
                x = out(reg) x,
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
