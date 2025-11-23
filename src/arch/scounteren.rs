//! Counter-Enable Register
//!
//! #See
//! Section `4.1.5 Counter-Enable Register (scounteren)` of `Volume II: RISC-V Privileged Architectures`

use core::arch::asm;

use crate::arch::csr::CSR;

/// Counter-Enable Register
///
/// #See
/// Section `4.1.5 Counter-Enable Register (scounteren)` of `Volume II: RISC-V Privileged Architectures`
#[derive(Debug)]
pub struct SCounterEn(u64);

impl SCounterEn {
    /// Check if [`Cycle`](crate::arch::cycle::Cycle) register is enabled.
    pub fn is_cycle_enabled(&self) -> bool {
        (self.0 & (1 << 0)) != 0
    }

    /// Check if [`Time`](crate::arch::time::Time) register is enabled.
    pub fn is_time_enabled(&self) -> bool {
        (self.0 & (1 << 1)) != 0
    }

    /// Check if [`InstRet`](crate::arch::inst_ret::InstRet) register is enabled.
    pub fn is_instret_enabled(&self) -> bool {
        (self.0 & (1 << 2)) != 0
    }

    /// Get raw inner value.
    pub const fn raw(self) -> u64 {
        self.0
    }

    /// Enable/disable [`Cycle`](crate::arch::cycle::Cycle) register.
    pub fn set_cycle_enabled(&mut self, enabled: bool) {
        match enabled {
            true => self.0 |= 1 << 0,
            false => self.0 &= !(1 << 0),
        };
    }

    /// Enable/disable [`Time`](crate::arch::time::Time) register.
    pub fn set_time_enabled(&mut self, enabled: bool) {
        match enabled {
            true => self.0 |= 1 << 1,
            false => self.0 &= !(1 << 1),
        };
    }

    /// Enable/disable [`InstRet`](crate::arch::inst_ret::InstRet) register.
    pub fn set_instret_enabled(&mut self, enabled: bool) {
        match enabled {
            true => self.0 |= 1 << 2,
            false => self.0 &= !(1 << 2),
        };
    }
}

impl CSR for SCounterEn {
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
                "csrw scounteren, {x}",
                x = in(reg) x,
            );
        }
    }

    fn read(&mut self) {
        let mut x: u64;
        unsafe {
            asm!(
                "csrr {x}, scounteren",
                x = out(reg) x,
            );
        }
        self.0 = x;
    }

    fn inner(&self) -> u64 {
        self.0
    }
}
