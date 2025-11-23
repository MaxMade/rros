//! Abstraction of general-purpose register

use core::fmt::Display;
use core::ops::Deref;
use core::ops::DerefMut;

/// Abstraction of general-purpose register
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Register(u64);

impl Register {
    /// Create `Register` from raw value.
    pub const fn new(value: u64) -> Self {
        Self { 0: value }
    }

    /// Get raw inner value.
    pub const fn raw(self) -> u64 {
        self.0
    }
}

impl Display for Register {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:#018x}", self.0)
    }
}

impl AsRef<u64> for Register {
    fn as_ref(&self) -> &u64 {
        &self.0
    }
}

impl AsMut<u64> for Register {
    fn as_mut(&mut self) -> &mut u64 {
        &mut self.0
    }
}

impl Deref for Register {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Register {
    fn deref_mut(&mut self) -> &mut u64 {
        &mut self.0
    }
}
