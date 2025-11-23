//! Convienient helper to access/modify CPU state.

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
