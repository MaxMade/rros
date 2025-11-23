//! Retired-Instruction-Counter Register
//!
//! #See
//! Section `4.1.4 Supervisor Timers and Performance Counters` of `Volume II: RISC-V Privileged Architectures`

use core::arch::asm;

use crate::arch::cpu::CSR;

/// Retired-Instruction-Counter Register
///
/// #See
/// Section `4.1.4 Supervisor Timers and Performance Counters` of `Volume II: RISC-V Privileged Architectures`
#[derive(Debug)]
pub struct InstRet(u64);

impl InstRet {}

impl CSR for InstRet {
    fn new(inner: u64) -> Self {
        Self(inner)
    }

    fn read(&mut self) {
        let mut x: u64;
        unsafe {
            asm!(
                "csrr {x}, instret",
                x = out(reg) x,
            );
        }
        self.0 = x;
    }

    fn write(&self) {
        panic!("INSTRET CSR must not be written!");
    }

    fn inner(&self) -> u64 {
        self.0
    }
}
