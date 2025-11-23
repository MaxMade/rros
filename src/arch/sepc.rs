//! Supervisor Exception Program Counter Register.
//!
//! #See
//! `4.1.7 Supervisor Exception Program Counter (sepc)` of `Volume II: RISC-V Privileged
//! Architectures`

use core::arch::asm;

use crate::arch::csr::CSR;

/// Abstraction of `sepc` register.
///
/// #See
/// `4.1.7 Supervisor Exception Program Counter (sepc)` of `Volume II: RISC-V Privileged
/// Architectures`
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SEPC(u64);

impl SEPC {}

impl CSR for SEPC {
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
                "csrw sepc, {x}",
                x = in(reg) x,
            );
        }
    }

    fn read(&mut self) {
        let mut x: u64;
        unsafe {
            asm!(
                "csrr {x}, sepc",
                x = out(reg) x,
            );
        }
        self.0 = x;
    }

    fn inner(&self) -> u64 {
        self.0
    }
}
