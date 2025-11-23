//! Fine-grained Interrupt Pending Register
//!
//! #See
//! Section `4.1.3 Supervisor Interrupt Registers (sip and sie)` of `Volume II: RISC-V Privileged Architectures`

use core::arch::asm;

use super::cpu::CSR;

/// Fine-grained Interrupt Pending Register
///
/// #See
/// Section `4.1.3 Supervisor Interrupt Registers (sip and sie)` of `Volume II: RISC-V Privileged Architectures`
#[derive(Debug)]
pub struct SIP(u64);

impl SIP {
    /// Create new, initialized `Supervisor Interrupt Pending` register.
    pub fn new() -> Self {
        let mut reg = SIP(0);
        reg.read();
        return reg;
    }
    /// Check if external interrupts are pending.
    pub fn is_external_interrupt_pending(&self) -> bool {
        self.0 & (1 << 9) != 0
    }

    /// Check if timer interrupts are pending.
    pub fn is_timer_interrupt_pending(&self) -> bool {
        self.0 & (1 << 5) != 0
    }

    /// Check if software interrupts are pending.
    pub fn is_software_interrupt_pending(&self) -> bool {
        self.0 & (1 << 1) != 0
    }

    /// Mark external interrupts as enabled.
    pub fn clear_external_interrupt_pending(&mut self) {
        self.0 &= !(1 << 9);
    }

    /// Mark timer interrupts as enabled.
    pub fn clear_timer_interrupt_pending(&mut self) {
        self.0 &= !(1 << 5);
    }

    /// Mark software interrupts as enabled.
    pub fn clear_software_interrupt_pending(&mut self) {
        self.0 &= !(1 << 1);
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

impl CSR for SIP {
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
                "csrw sip, {x}",
                x = in(reg) x,
            );
        }
    }

    fn read(&mut self) {
        let mut x: u64;
        unsafe {
            asm!(
                "csrr {x}, sip",
                x = out(reg) x,
            );
        }
        self.0 = x;
    }

    fn inner(&self) -> u64 {
        self.0
    }
}
