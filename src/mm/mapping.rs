//! Kernel APIs to create/update/revoke mappings.

use core::ffi::c_void;

use crate::kernel::address::{Address, PhysicalAddress, VirtualAddress};
use crate::kernel::cpu;
use crate::mm::error::MemoryError;
use crate::mm::page_allocator::PAGE_FRAME_ALLOCATOR;
use crate::sync::level::{LevelMapping, LevelPaging};
use crate::sync::ticketlock::TicketlockMapping;

use super::page_allocator::PageFrameAllocator;
use super::pte::PageTableEntry;

/// Protection bits.
///
/// See `4.3.1 Addressing and Memory Protection` of `Volume II: RISC-V Privileged Architectures`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protection {
    /// Read-only mapping.
    R,
    /// Readable/Writable mapping.
    RW,
    /// Execute-only mapping.
    X,
    /// Readable/Executable mapping.
    RX,
    /// Readable/Writable/Executable mapping.
    RWX,
}

impl Protection {
    /// Check if [`Protection`] is readable.
    pub fn is_readable(self) -> bool {
        match self {
            Protection::R => true,
            Protection::RW => true,
            Protection::X => false,
            Protection::RX => true,
            Protection::RWX => true,
        }
    }

    /// Check if [`Protection`] is writable.
    pub fn is_writable(self) -> bool {
        match self {
            Protection::R => false,
            Protection::RW => true,
            Protection::X => false,
            Protection::RX => false,
            Protection::RWX => true,
        }
    }

    /// Check if [`Protection`] is executable.
    pub fn is_executable(self) -> bool {
        match self {
            Protection::R => false,
            Protection::RW => false,
            Protection::X => true,
            Protection::RX => true,
            Protection::RWX => true,
        }
    }
}

/// Mode of mapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Kernel mapping.
    Kernel,
    /// User mapping.
    User,
}

/// Abstraction of `Sv39: Page-Based 39-bit Virtual-Memory System`.
pub struct VirtualMemorySystem {
    root: TicketlockMapping<PhysicalAddress<PageTableEntry>>,
}

impl VirtualMemorySystem {
    /// Create a new [`VirtualMemorySystem`] consisting of the root page table.
    pub fn new(token: LevelMapping) -> Result<(Self, LevelMapping), (MemoryError, LevelMapping)> {
        todo!();
    }

    /// Create a new mapping from `virt_addr` to `phys_addr` with specified `protection`/`mode`.
    pub fn create(
        &self,
        phys_addr: PhysicalAddress<c_void>,
        virt_addr: VirtualAddress<c_void>,
        protection: Protection,
        mode: Mode,
        token: LevelMapping,
    ) -> Result<LevelMapping, (MemoryError, LevelMapping)> {
        // Get first (root) page table
        let (p_pt_0, token) = self.root.lock(token);
        let v_pt_0 = PageFrameAllocator::phys_to_virt(*p_pt_0);

        // Check first page table
        let vpn_0 = Self::offset(virt_addr, 0);
        let pte_0 = unsafe { v_pt_0.add(vpn_0).as_mut_ptr().as_mut().unwrap() };

        // Check second page table
        let (p_pt_1, token) = match pte_0.is_valid() {
            true => {
                // Check entry
                assert!(pte_0.is_inner_page_table());
                assert!(pte_0.is_user_accessible() == false);

                (pte_0.get_physical_page(), token)
            }
            false => {
                // Allocate a fresh page table entry
                let (p_pt_1, token): (PhysicalAddress<PageTableEntry>, _) =
                    match PAGE_FRAME_ALLOCATOR.allocate(token) {
                        Ok((p_pt_1, token)) => unsafe { (p_pt_1.cast(), token) },
                        Err((err, token)) => {
                            return Err((err, p_pt_0.unlock(token)));
                        }
                    };

                // Update previous page table
                pte_0.set_physical_page(p_pt_1);
                pte_0.mark_as_valid(true);

                (p_pt_1, token)
            }
        };
        let v_pt_1 = PageFrameAllocator::phys_to_virt(p_pt_1);
        let vpn_1 = Self::offset(virt_addr, 1);
        let pte_1 = unsafe { v_pt_1.add(vpn_1).as_mut_ptr().as_mut().unwrap() };

        // Check third page table
        let (p_pt_2, token) = match pte_1.is_valid() {
            true => {
                // Check entry
                assert!(pte_1.is_inner_page_table());
                assert!(pte_1.is_user_accessible() == false);

                (pte_1.get_physical_page(), token)
            }
            false => {
                // Allocate a fresh page table entry
                let (p_pt_2, token): (PhysicalAddress<PageTableEntry>, _) =
                    match PAGE_FRAME_ALLOCATOR.allocate(token) {
                        Ok((p_pt_2, token)) => unsafe { (p_pt_2.cast(), token) },
                        Err((err, token)) => {
                            return Err((err, p_pt_0.unlock(token)));
                        }
                    };

                // Update previous page table
                pte_1.set_physical_page(p_pt_2);
                pte_1.mark_as_valid(true);

                (p_pt_2, token)
            }
        };
        let v_pt_2 = PageFrameAllocator::phys_to_virt(p_pt_2);
        let vpn_2 = Self::offset(virt_addr, 2);
        let pte_2 = unsafe { v_pt_2.add(vpn_2).as_mut_ptr().as_mut().unwrap() };

        // Try to create mapping
        match pte_2.is_valid() {
            true => {
                // Mapping for given virtual address already exists
                return Err((MemoryError::AddressAlreadyInUse, p_pt_0.unlock(token)));
            }
            false => {
                // Update mapping
                pte_2.set_physical_page(phys_addr);
                pte_2.mark_as_readable(protection.is_readable());
                pte_2.mark_as_writable(protection.is_writable());
                pte_2.mark_as_executable(protection.is_executable());
                pte_2.mark_as_user_accessible(mode == Mode::User);
                pte_2.mark_as_valid(true);
            }
        }

        // Unlock mapping
        let token = p_pt_0.unlock(token);
        Ok(token)
    }

    /// Update `protection`/`mode` of a given `virt_addr`.
    pub fn update(
        &self,
        virt_addr: VirtualAddress<c_void>,
        protection: Protection,
        mode: Mode,
        token: LevelMapping,
    ) -> Result<LevelMapping, (MemoryError, LevelMapping)> {
        todo!();
    }

    /// Revoke a new mapping targeting `virt_addr`.
    pub fn remove(
        &self,
        virt_addr: VirtualAddress<c_void>,
        token: LevelMapping,
    ) -> Result<LevelMapping, (MemoryError, LevelMapping)> {
        todo!();
    }

    /// Perform a software-based page table lookup.
    pub fn lookup(
        &self,
        virt_addr: VirtualAddress<c_void>,
        token: LevelMapping,
    ) -> Result<
        (PhysicalAddress<c_void>, Protection, Mode, LevelMapping),
        (MemoryError, LevelMapping),
    > {
        // Get first (root) page table
        let (p_pt_0, token) = self.root.lock(token);
        let v_pt_0 = PageFrameAllocator::phys_to_virt(*p_pt_0);

        // Check first page table
        let vpn_0 = Self::offset(virt_addr, 0);
        let pte_0 = unsafe { v_pt_0.add(vpn_0).as_mut_ptr().as_mut().unwrap() };

        // Check second page table
        let (p_pt_1, token): (PhysicalAddress<PageTableEntry>, LevelPaging) = match pte_0.is_valid()
        {
            true => {
                // Check entry
                assert!(pte_0.is_inner_page_table());
                assert!(pte_0.is_user_accessible() == false);

                (pte_0.get_physical_page(), token)
            }
            false => {
                let token = p_pt_0.unlock(token);
                return Err((MemoryError::NoSuchAddress, token));
            }
        };
        let v_pt_1 = PageFrameAllocator::phys_to_virt(p_pt_1);
        let vpn_1 = Self::offset(virt_addr, 1);
        let pte_1 = unsafe { v_pt_1.add(vpn_1).as_mut_ptr().as_mut().unwrap() };

        // Check third page table
        let (p_pt_2, token): (PhysicalAddress<PageTableEntry>, LevelPaging) = match pte_1.is_valid()
        {
            true => {
                // Check entry
                assert!(pte_1.is_inner_page_table());
                assert!(pte_1.is_user_accessible() == false);

                (pte_1.get_physical_page(), token)
            }
            false => {
                let token = p_pt_0.unlock(token);
                return Err((MemoryError::NoSuchAddress, token));
            }
        };
        let v_pt_2 = PageFrameAllocator::phys_to_virt(p_pt_2);
        let vpn_2 = Self::offset(virt_addr, 2);
        let pte_2 = unsafe { v_pt_2.add(vpn_2).as_mut_ptr().as_mut().unwrap() };

        // Try to create mapping
        if !pte_2.is_valid() {
            let token = p_pt_0.unlock(token);
            return Err((MemoryError::NoSuchAddress, token));
        }

        let phys_addr = pte_2.get_physical_page();
        let protection = match (
            pte_2.is_readable(),
            pte_2.is_writable(),
            pte_2.is_executable(),
        ) {
            (true, true, true) => Protection::RWX,
            (true, true, false) => Protection::RW,
            (true, false, true) => Protection::RX,
            (true, false, false) => Protection::R,
            (false, false, true) => Protection::X,
            (readable, writable, executable) => panic!(
                "Invalid memory protection: Readable? {} Writable? {} Executable? {}",
                readable, writable, executable
            ),
        };
        let mode = match pte_2.is_user_accessible() {
            true => Mode::User,
            false => Mode::Kernel,
        };

        // Unlock mapping
        let token = p_pt_0.unlock(token);
        Ok((phys_addr, protection, mode, token))
    }

    /// Check if `virt_addr` is readable for kernel-space.
    pub fn is_kernel_readable(
        &self,
        virt_addr: VirtualAddress<c_void>,
        token: LevelMapping,
    ) -> (bool, LevelMapping) {
        match self.lookup(virt_addr, token) {
            Ok((_, protection, mode, token)) => {
                return (mode == Mode::Kernel && protection.is_readable(), token);
            }
            Err((_, token)) => {
                return (false, token);
            }
        }
    }

    /// Check if `virt_addr` is writable for kernel-space.
    pub fn is_kernel_writable(
        &self,
        virt_addr: VirtualAddress<c_void>,
        token: LevelMapping,
    ) -> (bool, LevelMapping) {
        match self.lookup(virt_addr, token) {
            Ok((_, protection, mode, token)) => {
                return (mode == Mode::Kernel && protection.is_writable(), token);
            }
            Err((_, token)) => {
                return (false, token);
            }
        }
    }

    /// Check if `virt_addr` is executable for kernel-space.
    pub fn is_kernel_executable(
        &self,
        virt_addr: VirtualAddress<c_void>,
        token: LevelMapping,
    ) -> (bool, LevelMapping) {
        match self.lookup(virt_addr, token) {
            Ok((_, protection, mode, token)) => {
                return (mode == Mode::Kernel && protection.is_executable(), token);
            }
            Err((_, token)) => {
                return (false, token);
            }
        }
    }

    /// Check if `virt_addr` is readable for user-space.
    pub fn is_user_readable(
        &self,
        virt_addr: VirtualAddress<c_void>,
        token: LevelMapping,
    ) -> (bool, LevelMapping) {
        match self.lookup(virt_addr, token) {
            Ok((_, protection, mode, token)) => {
                return (mode == Mode::User && protection.is_readable(), token);
            }
            Err((_, token)) => {
                return (false, token);
            }
        }
    }

    /// Check if `virt_addr` is writable for user-space.
    pub fn is_user_writable(
        &self,
        virt_addr: VirtualAddress<c_void>,
        token: LevelMapping,
    ) -> (bool, LevelMapping) {
        match self.lookup(virt_addr, token) {
            Ok((_, protection, mode, token)) => {
                return (mode == Mode::User && protection.is_writable(), token);
            }
            Err((_, token)) => {
                return (false, token);
            }
        }
    }

    /// Check if `virt_addr` is executable for user-space.
    pub fn is_user_executable(
        &self,
        virt_addr: VirtualAddress<c_void>,
        token: LevelMapping,
    ) -> (bool, LevelMapping) {
        match self.lookup(virt_addr, token) {
            Ok((_, protection, mode, token)) => {
                return (mode == Mode::User && protection.is_executable(), token);
            }
            Err((_, token)) => {
                return (false, token);
            }
        }
    }

    /// Get offset first, second and third page table (respective `level`s: `0`, `1` and `2`).
    pub fn offset<T>(virt_addr: VirtualAddress<T>, level: usize) -> usize {
        let result = match level {
            0 => virt_addr.addr() >> 30,
            1 => virt_addr.addr() >> 21,
            2 => virt_addr.addr() >> 12,
            _ => panic!("Unsupported level {} for 39bit paging", level),
        };

        result & 0x1ff
    }
}
