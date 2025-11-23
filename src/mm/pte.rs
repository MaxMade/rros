//! Abstraction of a *P*age-*T*able-*E*ntry.
//!
//! For more details, see Section `4.4.1 Addressing and Memory Protection` of `Volume II: RISC-V Privileged Architectures`

use crate::kernel::address::Address;
use crate::kernel::address::PhysicalAddress;

const PHYSICAL_PAGE_NUMBER_SIZE: u64 = 1 << 44;

#[derive(Debug)]
enum Offset {
    V = 0,
    R = 1,
    W = 2,
    X = 3,
    U = 4,
    G = 5,
    A = 6,
    D = 7,
    PPN = 10,
}

/// Abstraction of a page table entry.
#[derive(Debug)]
pub struct PageTableEntry(u64);

impl PageTableEntry {
    /// Check if page-table entry is valid (`V` bit).
    pub const fn is_valid(&self) -> bool {
        (self.0 & (1 << Offset::V as u64)) != 0
    }

    /// Mark page as (in)valid (`V` bit).
    pub fn mark_as_valid(&mut self, valid: bool) {
        match valid {
            true => self.0 |= 1 << Offset::V as u64,
            false => self.0 &= !(1 << Offset::V as u64),
        };
    }

    /// Check if page-table entry is readable (`R` bit).
    pub const fn is_readable(&self) -> bool {
        (self.0 & (1 << Offset::R as u64)) != 0
    }

    /// Mark page-table entry as (non-)readable (`R` bit).
    pub fn mark_as_readable(&mut self, readable: bool) {
        match readable {
            true => self.0 |= 1 << Offset::R as u64,
            false => self.0 &= !(1 << Offset::R as u64),
        };
    }

    /// Check if page-table entry is writable (`W` bit).
    pub const fn is_writable(&self) -> bool {
        (self.0 & (1 << Offset::W as u64)) != 0
    }

    /// Mark page-table entry as (non-)writable (`W` bit).
    pub fn mark_as_writable(&mut self, writable: bool) {
        match writable {
            true => self.0 |= 1 << Offset::W as u64,
            false => self.0 &= !(1 << Offset::W as u64),
        };
    }

    /// Check if page-table entry is executable (`X` bit).
    pub const fn is_executable(&self) -> bool {
        (self.0 & (1 << Offset::X as u64)) != 0
    }

    /// Mark page-table entry as (non-)executable (`X` bit).
    pub fn mark_as_executable(&mut self, executable: bool) {
        match executable {
            true => self.0 |= 1 << Offset::X as u64,
            false => self.0 &= !(1 << Offset::X as u64),
        };
    }

    /// Check if entry contains pointer to the next table level (**inner page table**) or if its
    /// contains the target physical address (**leaf page table**) (`R`/`W`/`X` bits).
    pub const fn is_inner_page_table(&self) -> bool {
        !self.is_readable() && !self.is_writable() && !self.is_executable()
    }

    /// Mark entry as **inner page table**  (`R`/`W`/`X` bits).
    ///
    /// # Caution
    /// This function will clear the respective  `R`/`W`/`X` bits.
    pub fn mark_as_inner_page_table(&mut self) {
        self.mark_as_readable(false);
        self.mark_as_writable(false);
        self.mark_as_executable(false);
    }

    /// Check if page-table entry is user-accessible (`U` bit).
    pub const fn is_user_accessible(&self) -> bool {
        (self.0 & (1 << Offset::U as u64)) != 0
    }

    /// Mark page-table entry as (non-)user-accessible (`U` bit).
    pub fn mark_as_user_accessible(&mut self, user_accessible: bool) {
        match user_accessible {
            true => self.0 |= 1 << Offset::U as u64,
            false => self.0 &= !(1 << Offset::U as u64),
        };
    }

    /// Check if page-table entry is global (`G` bit).
    pub const fn is_global(&self) -> bool {
        (self.0 & (1 << Offset::G as u64)) != 0
    }

    /// Mark page-table entry as (non-)global (`G` bit).
    pub fn mark_as_global(&mut self, global: bool) {
        match global {
            true => self.0 |= 1 << Offset::G as u64,
            false => self.0 &= !(1 << Offset::G as u64),
        };
    }

    /// Check if page-table entry is accessed (`A` bit).
    pub const fn is_accessed(&self) -> bool {
        (self.0 & (1 << Offset::A as u64)) != 0
    }

    /// Clear access flag for page-table entry (`A` bit).
    pub fn clear_access_flag(&mut self) {
        self.0 &= !(1 << Offset::A as u64);
    }

    /// Check if page-table entry is dirty (`D` bit).
    pub const fn is_dirty(&self) -> bool {
        (self.0 & (1 << Offset::D as u64)) != 0
    }

    /// Clear dirty flag for page-table entry (`D` bit).
    pub fn clear_dirty_flag(&mut self) {
        self.0 &= !(1 << Offset::D as u64);
    }

    /// Get physical page number of page-table entry (`PPN` bits)
    pub fn get_physical_page_number<T>(&self) -> PhysicalAddress<T> {
        let raw_addr = (self.0 >> Offset::PPN as u64) & (PHYSICAL_PAGE_NUMBER_SIZE - 1);
        PhysicalAddress::new(raw_addr as *mut T)
    }

    /// Get physical page number of page-table entry (`PPN` bits)
    pub fn set_physical_page_number<T>(&mut self, phys_addr: PhysicalAddress<T>) {
        let raw_addr = phys_addr.as_ptr() as u64;
        if raw_addr >= PHYSICAL_PAGE_NUMBER_SIZE {
            panic!("Only 44-bits physical addresses are supported!");
        }

        self.0 &= !(PHYSICAL_PAGE_NUMBER_SIZE - 1 << Offset::PPN as u64);
        self.0 |= raw_addr << Offset::PPN as u64;
    }
}
