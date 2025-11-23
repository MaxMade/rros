//! Supervisor time-compare register
//!
//! #See
//! Section `1.1. Supervisor Timer Register (stimecmp)` of `RISC-V "stimecmp / vstimecmp" Extension`

use core::arch::asm;

use crate::arch::csr::CSR;

/// Supervisor time-compare register
///
/// #See
/// Section `1.1. Supervisor Timer Register (stimecmp)` of `RISC-V "stimecmp / vstimecmp" Extension`
#[derive(Debug)]
pub struct STimeCmp(u64);

impl STimeCmp {}

impl CSR for STimeCmp {
    fn new(inner: u64) -> Self {
        Self(inner)
    }

    fn read(&mut self) {
        let mut x: u64;
        unsafe {
            asm!(
                "csrr {x}, stimecmp",
                x = out(reg) x,
            );
        }
        self.0 = x;
    }

    fn write(&self) {
        let x: u64 = self.0;
        unsafe {
            asm!(
                "csrw stimecmp, {x}",
                x = in(reg) x,
            );
        }
    }

    fn inner(&self) -> u64 {
        self.0
    }
}
