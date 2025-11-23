//! Supervisor Scratch Register.
//!
//! #See
//! `4.1.6 Supervisor Scratch Register (sscratch)` of `Volume II: RISC-V Privileged Architectures`

use core::arch::asm;

use crate::arch::cpu::CSR;

/// Abstraction of `sscratch` register.
///
/// #See
/// `4.1.6 Supervisor Scratch Register (sscratch)` of `Volume II: RISC-V Privileged Architectures`
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SScratch(u64);

impl SScratch {
    /// Create `SScratch` from raw value.
    pub const fn new(value: u64) -> Self {
        Self { 0: value }
    }

    /// Get raw inner value.
    pub const fn raw(self) -> u64 {
        self.0
    }
}

impl CSR for SScratch {
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
                "csrw sscratch, {x}",
                x = in(reg) x,
            );
        }
    }

    fn read(&mut self) {
        let mut x: u64;
        unsafe {
            asm!(
                "csrr {x}, sscratch",
                x = out(reg) x,
            );
        }
        self.0 = x;
    }

    fn inner(&self) -> u64 {
        self.0
    }
}
