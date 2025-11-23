//! Fine-grained Interrupt Enable Register
//!
//! #See
//! Section `4.1.3 Supervisor Interrupt Registers (sip and sie)` of `Volume II: RISC-V Privileged Architectures`

use core::arch::asm;

use crate::arch::csr::CSR;

/// Fine-grained Interrupt Enable Register
///
/// #See
/// Section `4.1.3 Supervisor Interrupt Registers (sip and sie)` of `Volume II: RISC-V Privileged Architectures`
#[derive(Debug)]
pub struct SIE(u64);

impl SIE {
    /// Check if external interrupts are enabled.
    pub fn is_external_interrupt_enabled(&self) -> bool {
        self.0 & (1 << 9) != 0
    }

    /// Check if timer interrupts are enabled.
    pub fn is_timer_interrupt_enabled(&self) -> bool {
        self.0 & (1 << 5) != 0
    }

    /// Check if software interrupts are enabled.
    pub fn is_software_interrupt_enabled(&self) -> bool {
        self.0 & (1 << 1) != 0
    }

    /// Mark external interrupts as enabled.
    pub fn mark_external_interrupt_enabled(&mut self, enabled: bool) {
        match enabled {
            true => self.0 |= 1 << 9,
            false => self.0 &= !(1 << 9),
        };
    }

    /// Mark timer interrupts as enabled.
    pub fn mark_timer_interrupt_enabled(&mut self, enabled: bool) {
        match enabled {
            true => self.0 |= 1 << 5,
            false => self.0 &= !(1 << 5),
        };
    }

    /// Mark software interrupts as enabled.
    pub fn mark_software_interrupt_enabled(&mut self, enabled: bool) {
        match enabled {
            true => self.0 |= 1 << 1,
            false => self.0 &= !(1 << 1),
        };
    }

    /// Set all enable-bits for interrupt and write updated value back to register.
    pub fn enable_all_interrupts(&mut self) {
        self.0 = u64::MAX;
    }

    /// Clear all enable-bits for interrupt and write updated value back to register.
    pub fn disable_all_interrupts(&mut self) {
        self.0 = 0u64;
    }
}

impl CSR for SIE {
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
                "csrw sie, {x}",
                x = in(reg) x,
            );
        }
    }

    fn read(&mut self) {
        let mut x: u64;
        unsafe {
            asm!(
                "csrr {x}, sie",
                x = out(reg) x,
            );
        }
        self.0 = x;
    }

    fn inner(&self) -> u64 {
        self.0
    }
}
