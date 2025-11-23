//! Convienient helper to access/modify CPU state.

use core::fmt::Display;

/// Abstraction of hard ID.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct HartID(u64);

impl HartID {
    /// Create HartID from raw value.
    pub const fn new(value: u64) -> Self {
        Self { 0: value }
    }
}

impl Display for HartID {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:#018x}", self.0)
    }
}
