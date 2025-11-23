//! Time Register.
//!
//! #See
//! Section `4.1.4 Supervisor Timers and Performance Counters` of `Volume II: RISC-V Privileged Architectures`

use core::arch::asm;

use crate::arch::cpu::CSR;

/// Time Register.
///
/// #See
/// Section `4.1.4 Supervisor Timers and Performance Counters` of `Volume II: RISC-V Privileged Architectures`
#[derive(Debug)]
pub struct Time(u64);

impl Time {}

impl CSR for Time {
    fn new(inner: u64) -> Self
    where
        Self: Sized,
    {
        Self(inner)
    }

    fn write(&self) {
        panic!("TIME CSR must not be written!");
    }

    fn read(&mut self) {
        let mut x: u64;
        unsafe {
            asm!(
                "csrr {x}, time",
                x = out(reg) x,
            );
        }
        self.0 = x;
    }

    fn inner(&self) -> u64 {
        self.0
    }
}
