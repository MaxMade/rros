//! Supervisor Trap Value Register.
//!
//! #See
//! `4.1.9 Supervisor Trap Value (stval) Register` of `Volume II: RISC-V Privileged Architectures`

use core::arch::asm;

use crate::arch::cpu::CSR;

/// Supervisor Trap Value Register.
///
/// #See
/// `4.1.9 Supervisor Trap Value (stval) Register` of `Volume II: RISC-V Privileged Architectures`
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct STVal(u64);

impl STVal {
    /// Get raw inner value.
    pub const fn raw(self) -> u64 {
        self.0
    }
}

impl CSR for STVal {
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
                "csrw stval, {x}",
                x = in(reg) x,
            );
        }
    }

    fn read(&mut self) {
        let mut x: u64;
        unsafe {
            asm!(
                "csrr {x}, stval",
                x = out(reg) x,
            );
        }
        self.0 = x;
    }

    fn inner(&self) -> u64 {
        self.0
    }
}
