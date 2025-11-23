//! Supervisor Address Translation and Protection Register.
//!
//! #See
//! `4.1.11 Supervisor Address Translation and Protection (satp) Register` of `Volume II: RISC-V Privileged Architectures`

use core::arch::asm;

use crate::arch::cpu::page_size;
use crate::arch::cpu::CSR;
use crate::kernel::address::Address;
use crate::kernel::address::PhysicalAddress;
use crate::mm::pte::PageTableEntry;

/// Supervisor Address Translation and Protection Register.
///
/// #See
/// `4.1.11 Supervisor Address Translation and Protection (satp) Register` of `Volume II: RISC-V Privileged Architectures`
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SATP(u64);

impl SATP {
    /// Get address of root page table
    pub const fn get_root_page_table(&self) -> PhysicalAddress<PageTableEntry> {
        let ppn = (self.0 & 0xFFF_FFFF_FFFF) as u64;
        PhysicalAddress::new((ppn * page_size() as u64) as *mut _)
    }

    /// Set address of root page table
    pub fn set_root_page_table(&mut self, phys_addr: PhysicalAddress<PageTableEntry>) {
        let ppn = phys_addr.addr() / page_size();
        self.0 &= !0xFFF_FFFF_FFFF;
        self.0 |= ppn as u64;
    }
}

impl CSR for SATP {
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
                "csrw satp, {x}",
                x = in(reg) x,
            );
        }
    }

    fn read(&mut self) {
        let mut x: u64;
        unsafe {
            asm!(
                "csrr {x}, satp",
                x = out(reg) x,
            );
        }
        self.0 = x;
    }

    fn inner(&self) -> u64 {
        self.0
    }
}
