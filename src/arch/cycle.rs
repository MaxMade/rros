//! Cycle-Counter Register
//!
//! #See
//! Section `4.1.4 Supervisor Timers and Performance Counters` of `Volume II: RISC-V Privileged Architectures`

use core::arch::asm;

use crate::arch::cpu::CSR;

/// Cycle-Counter Register
///
/// #See
/// Section `4.1.4 Supervisor Timers and Performance Counters` of `Volume II: RISC-V Privileged Architectures`
#[derive(Debug)]
pub struct Cycle(u64);

impl Cycle {}

impl CSR for Cycle {
    /// Create new, initialized [`Cycle`].
    fn new(inner: u64) -> Self {
        Self(inner)
    }

    fn read(&mut self) {
        let mut x: u64;
        unsafe {
            asm!(
                "csrr {x}, cycle",
                x = out(reg) x,
            );
        }
        self.0 = x;
    }

    /// Get raw inner value.
    fn inner(&self) -> u64 {
        self.0
    }

    fn write(&self) {
        panic!("CYCLE CSR must not be written!");
    }
}
