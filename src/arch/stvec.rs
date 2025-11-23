//! Trap-Vector Base-Address Register
//!
//! #See
//! Section `4.1.2 Supervisor Trap Vector Base Address Register (stvec)` of `Volume II: RISC-V Privileged Architectures`

use core::arch::asm;
use core::fmt::Display;

use crate::arch::csr::CSR;

/// Trap-Vector Base-Address Register
///
/// #See
/// Section `4.1.2 Supervisor Trap Vector Base Address Register (stvec)` of `Volume II: RISC-V Privileged Architectures`
#[derive(Debug)]
pub struct STVec(u64);

impl STVec {
    /// Get `Mode`.
    pub fn get_mode(&self) -> STVecMode {
        match self.0 & 0b11 {
            0 => STVecMode::Direct,
            1 => STVecMode::Vectored,
            _ => panic!(),
        }
    }

    /// Set `Mode`.
    pub fn set_mode(&mut self, mode: STVecMode) {
        self.0 &= !(0b11);
        self.0 |= (mode as u64) & 0b11;
    }

    /// Get `Base`.
    pub fn get_base(&self) -> u64 {
        self.0 >> 2
    }

    /// Set `Base`.
    pub fn set_base(&mut self, base: u64) {
        self.0 &= 0b11;
        self.0 |= base << 2;
    }
}

impl CSR for STVec {
    fn new(value: u64) -> Self {
        let reg = STVec(value);
        return reg;
    }

    fn read(&mut self) {
        let mut x: u64;
        unsafe {
            asm!(
                "csrr {x}, stvec",
                x = out(reg) x,
            );
        }
        self.0 = x;
    }

    fn write(&self) {
        let x: u64 = self.0;
        unsafe {
            asm!(
                "csrw stvec, {x}",
                x = in(reg) x,
            );
        }
    }

    fn inner(&self) -> u64 {
        self.0
    }
}

impl Display for STVec {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:#018x}", self.0)
    }
}

/// Mode of vector table.
#[derive(Debug, Eq, PartialEq)]
pub enum STVecMode {
    /// All exceptions set `pc` to `BASE`.
    Direct = 0,
    /// Asynchronous interrupts set `pc` to `BASE+4×cause`.
    Vectored = 1,
}

impl Display for STVecMode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            STVecMode::Direct => write!(f, "Direct"),
            STVecMode::Vectored => write!(f, "Vectored"),
        }
    }
}
