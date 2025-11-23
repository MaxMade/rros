//! Supervisor Cause Register
//!
//! #See
//! `4.1.8 Supervisor Cause Register (scause)` of `Volume II: RISC-V Privileged
//! Architectures`

use core::arch::asm;

use crate::arch::csr::CSR;

/// Abstraction of `scause` register.
///
/// #See
/// `4.1.8 Supervisor Cause Register (scause)` of `Volume II: RISC-V Privileged
/// Architectures`
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SCause(u64);

impl SCause {
    /// Create `SCause` from raw value.
    pub const fn new(value: u64) -> Self {
        Self { 0: value }
    }

    /// Get raw inner value.
    pub const fn raw(self) -> u64 {
        self.0
    }
}

impl CSR for SCause {
    fn new(inner: u64) -> Self
    where
        Self: Sized,
    {
        Self(inner)
    }

    fn write(&self) {
        let x: u64 = self.0;
        unsafe {
            asm!(
                "csrw scause, {x}",
                x = in(reg) x,
            );
        }
    }

    fn read(&mut self) {
        let mut x: u64;
        unsafe {
            asm!(
                "csrr {x}, scause",
                x = out(reg) x,
            );
        }
        self.0 = x;
    }

    fn inner(&self) -> u64 {
        self.0
    }
}
