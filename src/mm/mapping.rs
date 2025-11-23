//! Kernel APIs to create/update/revoke mappings.

use core::ffi::c_void;

use crate::arch::cpu;
use crate::arch::cpu::SATP;
use crate::kernel::address::{Address, PhysicalAddress, VirtualAddress};
use crate::kernel::compiler;
use crate::mm::error::MemoryError;
use crate::mm::page_allocator::PageFrameAllocator;
use crate::mm::page_allocator::PAGE_FRAME_ALLOCATOR;
use crate::mm::pte::PageTableEntry;
use crate::sync::const_cell::ConstCell;
use crate::sync::init_cell::InitCell;
use crate::sync::level::{LevelInitialization, LevelMapping, LevelPaging};
use crate::sync::ticketlock::TicketlockMapping;

/// Virtual memory system containing only kernel-addresses (upper `4GiB`).
pub static KERNEL_VIRTUAL_MEMORY_SYSTEM: InitCell<VirtualMemorySystem> = InitCell::new();

static KERNEL_PTS_1: InitCell<TicketlockMapping<PageTableSubspace>> = InitCell::new();

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

/// Page table entries at level 1 for either kernel space (upper `4GiB`) or user space (lower `4GiB`).
struct PageTableSubspace([PhysicalAddress<PageTableEntry>; 4]);
unsafe impl Send for PageTableSubspace {}
unsafe impl Sync for PageTableSubspace {}

/// 39bit page-based Virtual Memory System.
///
/// For more details, see `4.4 Sv39: Page-Based 39-bit Virtual-Memory System` of `Volume II: RISC-V Privileged Architectures`.
pub struct VirtualMemorySystem {
    root: ConstCell<PhysicalAddress<PageTableEntry>>,
    user_pts_1: TicketlockMapping<PageTableSubspace>,
    kernel_pts_1: &'static TicketlockMapping<PageTableSubspace>,
}
unsafe impl Send for VirtualMemorySystem {}
unsafe impl Sync for VirtualMemorySystem {}

impl VirtualMemorySystem {
    /// Apply current mapping by writing [`SATP`] register.
    pub fn load(&self) {
        let mut satp = SATP::new();
        satp.set_root_page_table(*self.root);
        satp.write();
    }

    /// Create initial [`VirtualMemorySystem`] for kernel-space only.
    pub fn initalize(token: LevelInitialization) -> LevelInitialization {
        // Initialize kernel mapping (PT 0 & PT 1s)
        let (p_pt_0, token) = PAGE_FRAME_ALLOCATOR.early_allocate(token).unwrap();
        let p_pt_0: PhysicalAddress<PageTableEntry> = unsafe { p_pt_0.cast() };

        let (p_pt_1_511, token) = PAGE_FRAME_ALLOCATOR.early_allocate(token).unwrap();
        let p_pt_1_511: PhysicalAddress<PageTableEntry> = unsafe { p_pt_1_511.cast() };

        let (p_pt_1_510, token) = PAGE_FRAME_ALLOCATOR.early_allocate(token).unwrap();
        let p_pt_1_510: PhysicalAddress<PageTableEntry> = unsafe { p_pt_1_510.cast() };

        let (p_pt_1_509, token) = PAGE_FRAME_ALLOCATOR.early_allocate(token).unwrap();
        let p_pt_1_509: PhysicalAddress<PageTableEntry> = unsafe { p_pt_1_509.cast() };

        let (p_pt_1_508, token) = PAGE_FRAME_ALLOCATOR.early_allocate(token).unwrap();
        let p_pt_1_508: PhysicalAddress<PageTableEntry> = unsafe { p_pt_1_508.cast() };

        let v_pt_0: VirtualAddress<PageTableEntry> = PageFrameAllocator::phys_to_virt(p_pt_0);

        let v_pte_0 = unsafe { v_pt_0.add(511).as_mut_ptr().as_mut().unwrap() };
        v_pte_0.set_physical_page(p_pt_1_511);
        v_pte_0.mark_as_valid(true);

        let v_pte_0 = unsafe { v_pt_0.add(510).as_mut_ptr().as_mut().unwrap() };
        v_pte_0.set_physical_page(p_pt_1_510);
        v_pte_0.mark_as_valid(true);

        let v_pte_0 = unsafe { v_pt_0.add(509).as_mut_ptr().as_mut().unwrap() };
        v_pte_0.set_physical_page(p_pt_1_509);
        v_pte_0.mark_as_valid(true);

        let v_pte_0 = unsafe { v_pt_0.add(508).as_mut_ptr().as_mut().unwrap() };
        v_pte_0.set_physical_page(p_pt_1_508);
        v_pte_0.mark_as_valid(true);

        // Initialize kernel page tables for level 1
        let mut kernel_pts = KERNEL_PTS_1.get_mut(token);
        kernel_pts.get_mut().0[0] = p_pt_1_508;
        kernel_pts.get_mut().0[1] = p_pt_1_509;
        kernel_pts.get_mut().0[2] = p_pt_1_510;
        kernel_pts.get_mut().0[3] = p_pt_1_511;
        let token = kernel_pts.destroy();
        let token = unsafe { KERNEL_PTS_1.finanlize(token) };

        // Initialize kernel-only
        let vms = Self {
            root: ConstCell::new(p_pt_0),
            user_pts_1: TicketlockMapping::new(PageTableSubspace([PhysicalAddress::null(); 4])),
            kernel_pts_1: KERNEL_PTS_1.as_ref(),
        };

        let mut kernel_virtual_memory_system = KERNEL_VIRTUAL_MEMORY_SYSTEM.get_mut(token);
        *kernel_virtual_memory_system.as_mut() = vms;
        let token = kernel_virtual_memory_system.destroy();
        let token = unsafe { KERNEL_VIRTUAL_MEMORY_SYSTEM.finanlize(token) };

        let mut token = Some(token);

        // Map .text segment
        let text_segment_size = compiler::text_segment_size();
        assert!(text_segment_size % cpu::page_size() == 0);
        assert!(compiler::text_segment_phys_start().addr() % cpu::page_size() == 0);
        assert!(compiler::text_segment_phys_end().addr() % cpu::page_size() == 0);
        assert!(compiler::text_segment_virt_start().addr() % cpu::page_size() == 0);
        assert!(compiler::text_segment_virt_end().addr() % cpu::page_size() == 0);
        for i in 0..text_segment_size / cpu::page_size() {
            let phys_addr = compiler::text_segment_phys_start().add(cpu::page_size() * i);
            let virt_addr = compiler::text_segment_virt_start().add(cpu::page_size() * i);
            token = Some(
                KERNEL_VIRTUAL_MEMORY_SYSTEM
                    .as_ref()
                    .early_create(
                        phys_addr,
                        virt_addr,
                        Protection::RX,
                        Mode::Kernel,
                        token.unwrap(),
                    )
                    .unwrap(),
            );
        }

        // Map .rodata segment
        let rodata_segment_size = compiler::rodata_segment_size();
        assert!(rodata_segment_size % cpu::page_size() == 0);
        assert!(compiler::rodata_segment_phys_start().addr() % cpu::page_size() == 0);
        assert!(compiler::rodata_segment_phys_end().addr() % cpu::page_size() == 0);
        assert!(compiler::rodata_segment_virt_start().addr() % cpu::page_size() == 0);
        assert!(compiler::rodata_segment_virt_end().addr() % cpu::page_size() == 0);
        for i in 0..rodata_segment_size / cpu::page_size() {
            let phys_addr = compiler::rodata_segment_phys_start().add(cpu::page_size() * i);
            let virt_addr = compiler::rodata_segment_virt_start().add(cpu::page_size() * i);
            token = Some(
                KERNEL_VIRTUAL_MEMORY_SYSTEM
                    .as_ref()
                    .early_create(
                        phys_addr,
                        virt_addr,
                        Protection::R,
                        Mode::Kernel,
                        token.unwrap(),
                    )
                    .unwrap(),
            );
        }

        // Map .data segment
        let data_segment_size = compiler::data_segment_size();
        assert!(data_segment_size % cpu::page_size() == 0);
        assert!(compiler::data_segment_phys_start().addr() % cpu::page_size() == 0);
        assert!(compiler::data_segment_phys_end().addr() % cpu::page_size() == 0);
        assert!(compiler::data_segment_virt_start().addr() % cpu::page_size() == 0);
        assert!(compiler::data_segment_virt_end().addr() % cpu::page_size() == 0);
        for i in 0..data_segment_size / cpu::page_size() {
            let phys_addr = compiler::data_segment_phys_start().add(cpu::page_size() * i);
            let virt_addr = compiler::data_segment_virt_start().add(cpu::page_size() * i);
            token = Some(
                KERNEL_VIRTUAL_MEMORY_SYSTEM
                    .as_ref()
                    .early_create(
                        phys_addr,
                        virt_addr,
                        Protection::RW,
                        Mode::Kernel,
                        token.unwrap(),
                    )
                    .unwrap(),
            );
        }

        // Map .bss segment
        let bss_segment_size = compiler::bss_segment_size();
        assert!(bss_segment_size % cpu::page_size() == 0);
        assert!(compiler::bss_segment_phys_start().addr() % cpu::page_size() == 0);
        assert!(compiler::bss_segment_phys_end().addr() % cpu::page_size() == 0);
        assert!(compiler::bss_segment_virt_start().addr() % cpu::page_size() == 0);
        assert!(compiler::bss_segment_virt_end().addr() % cpu::page_size() == 0);
        for i in 0..bss_segment_size / cpu::page_size() {
            let phys_addr = compiler::bss_segment_phys_start().add(cpu::page_size() * i);
            let virt_addr = compiler::bss_segment_virt_start().add(cpu::page_size() * i);
            token = Some(
                KERNEL_VIRTUAL_MEMORY_SYSTEM
                    .as_ref()
                    .early_create(
                        phys_addr,
                        virt_addr,
                        Protection::RW,
                        Mode::Kernel,
                        token.unwrap(),
                    )
                    .unwrap(),
            );
        }

        // XXX: Map Page Tables as 128 2MiB Huge-Page
        const HUGE_PAGE_SIZE: usize = 2 * 1024 * 1024;
        assert!(compiler::pages_mem_size() % HUGE_PAGE_SIZE == 0);
        assert!(compiler::pages_mem_phys_start().addr() % HUGE_PAGE_SIZE == 0);
        assert!(compiler::pages_mem_phys_end().addr() % HUGE_PAGE_SIZE == 0);
        assert!(compiler::pages_mem_virt_start().addr() % HUGE_PAGE_SIZE == 0);
        assert!(compiler::pages_mem_virt_end().addr() % HUGE_PAGE_SIZE == 0);

        // Get first (root) page table
        let p_pt_0 = KERNEL_VIRTUAL_MEMORY_SYSTEM.as_ref().root.as_ref();
        let v_pt_0 = PageFrameAllocator::phys_to_virt(*p_pt_0);

        for i in 0..compiler::pages_mem_size() / HUGE_PAGE_SIZE {
            let virt_addr =
                unsafe { compiler::pages_mem_virt_start().byte_add(i * HUGE_PAGE_SIZE) };
            let phys_addr =
                unsafe { compiler::pages_mem_phys_start().byte_add(i * HUGE_PAGE_SIZE) };

            // Check first page table
            let vpn_0 = Self::offset(virt_addr, 0);
            let pte_0 = unsafe { v_pt_0.add(vpn_0).as_mut_ptr().as_mut().unwrap() };
            if !pte_0.is_valid() {
                panic!("");
            }

            // Check second page table
            let (p_pts_1, p_pt_1) = match vpn_0 {
                508 | 509 | 510 | 511 => {
                    let kernel_page_tables = KERNEL_VIRTUAL_MEMORY_SYSTEM
                        .as_ref()
                        .kernel_pts_1
                        .init_lock(token.unwrap());
                    let p_pt_1 = kernel_page_tables.0[vpn_0 - 508];
                    (kernel_page_tables, p_pt_1)
                }
                _ => {
                    panic!();
                }
            };
            let v_pt_1 = PageFrameAllocator::phys_to_virt(p_pt_1);
            let vpn_1 = Self::offset(virt_addr, 1);
            let pte_1 = unsafe { v_pt_1.add(vpn_1).as_mut_ptr().as_mut().unwrap() };
            if pte_1.is_valid() {
                panic!();
            }
            pte_1.set_physical_page(phys_addr);
            pte_1.mark_as_readable(true);
            pte_1.mark_as_writable(true);
            pte_1.mark_as_executable(false);
            pte_1.mark_as_user_accessible(false);
            pte_1.mark_as_valid(true);

            token = Some(p_pts_1.init_unlock());
        }

        let token = token.unwrap();
        token
    }

    /// Create a new [`VirtualMemorySystem`].
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
        let p_pt_0 = self.root.as_ref();
        let v_pt_0 = PageFrameAllocator::phys_to_virt(*p_pt_0);

        // Check first page table
        let vpn_0 = Self::offset(virt_addr, 0);
        let pte_0 = unsafe { v_pt_0.add(vpn_0).as_mut_ptr().as_mut().unwrap() };
        if !pte_0.is_valid() {
            return Err((MemoryError::InvalidAddress, token));
        }

        // Check second page table
        let (p_pts_1, p_pt_1, token) = match vpn_0 {
            0 | 1 | 2 | 3 => {
                if mode != Mode::User {
                    return Err((MemoryError::InvalidAddress, token));
                }

                let (user_page_tables, token) = self.user_pts_1.lock(token);
                let p_pt_1 = user_page_tables.0[vpn_0];
                (user_page_tables, p_pt_1, token)
            }
            508 | 509 | 510 | 511 => {
                if mode != Mode::Kernel {
                    return Err((MemoryError::InvalidAddress, token));
                }

                let (kernel_page_tables, token) = self.kernel_pts_1.lock(token);
                let p_pt_1 = kernel_page_tables.0[vpn_0 - 508];
                (kernel_page_tables, p_pt_1, token)
            }
            _ => {
                return Err((MemoryError::InvalidAddress, token));
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
                            return Err((err, p_pts_1.unlock(token)));
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
                return Err((MemoryError::AddressAlreadyInUse, p_pts_1.unlock(token)));
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
        let token = p_pts_1.unlock(token);
        Ok(token)
    }

    /// Create a new mapping from `virt_addr` to `phys_addr` with specified `protection`/`mode`
    /// during initialization.
    pub fn early_create(
        &self,
        phys_addr: PhysicalAddress<c_void>,
        virt_addr: VirtualAddress<c_void>,
        protection: Protection,
        mode: Mode,
        token: LevelInitialization,
    ) -> Result<LevelInitialization, (MemoryError, LevelInitialization)> {
        let (mut page, token): (Option<PhysicalAddress<PageTableEntry>>, _) =
            match PAGE_FRAME_ALLOCATOR.early_allocate(token) {
                Ok((page, token)) => (Some(unsafe { page.cast() }), token),
                Err((err, token)) => return Err((err, token)),
            };

        // Get first (root) page table
        let p_pt_0 = self.root.as_ref();
        let v_pt_0 = PageFrameAllocator::phys_to_virt(*p_pt_0);

        // Check first page table
        let vpn_0 = Self::offset(virt_addr, 0);
        let pte_0 = unsafe { v_pt_0.add(vpn_0).as_mut_ptr().as_mut().unwrap() };
        if !pte_0.is_valid() {
            return Err((MemoryError::InvalidAddress, token));
        }

        // Check second page table
        let (p_pts_1, p_pt_1) = match vpn_0 {
            0 | 1 | 2 | 3 => {
                if mode != Mode::User {
                    return Err((MemoryError::InvalidAddress, token));
                }

                let user_page_tables = self.user_pts_1.init_lock(token);
                let p_pt_1 = user_page_tables.0[vpn_0];
                (user_page_tables, p_pt_1)
            }
            508 | 509 | 510 | 511 => {
                if mode != Mode::Kernel {
                    return Err((MemoryError::InvalidAddress, token));
                }

                let kernel_page_tables = self.kernel_pts_1.init_lock(token);
                let p_pt_1 = kernel_page_tables.0[vpn_0 - 508];
                (kernel_page_tables, p_pt_1)
            }
            _ => {
                return Err((MemoryError::InvalidAddress, token));
            }
        };
        let v_pt_1 = PageFrameAllocator::phys_to_virt(p_pt_1);
        let vpn_1 = Self::offset(virt_addr, 1);
        let pte_1 = unsafe { v_pt_1.add(vpn_1).as_mut_ptr().as_mut().unwrap() };

        // Check third page table
        let p_pt_2 = match pte_1.is_valid() {
            true => {
                // Check entry
                assert!(pte_1.is_inner_page_table());
                assert!(pte_1.is_user_accessible() == false);

                pte_1.get_physical_page()
            }
            false => {
                // Update previous page table
                let p_pt_2 = page.take().unwrap();
                pte_1.set_physical_page(p_pt_2);
                pte_1.mark_as_valid(true);

                p_pt_2
            }
        };
        let v_pt_2 = PageFrameAllocator::phys_to_virt(p_pt_2);
        let vpn_2 = Self::offset(virt_addr, 2);
        let pte_2 = unsafe { v_pt_2.add(vpn_2).as_mut_ptr().as_mut().unwrap() };

        // Try to create mapping
        match pte_2.is_valid() {
            true => {
                // Mapping for given virtual address already exists
                return Err((MemoryError::AddressAlreadyInUse, p_pts_1.init_unlock()));
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
        let token = p_pts_1.init_unlock();
        Ok(token)
    }

    /// Create a new (readable/writable for kernel) mapping for `phys_addr` associated driver memory-mapped IO space.
    pub fn early_create_dev(
        &self,
        phys_addr: PhysicalAddress<c_void>,
        size: usize,
        token: LevelInitialization,
    ) -> Result<(VirtualAddress<c_void>, LevelInitialization), (MemoryError, LevelInitialization)>
    {
        // Calculate address shift
        let offset =
            compiler::data_segment_virt_start().addr() - compiler::data_segment_phys_start().addr();

        // Calculate aligned size virtual/physical address and size
        let size = size + (phys_addr.addr() % cpu::page_size());
        let size = (size + cpu::page_size() - 1) & !(cpu::page_size() - 1);

        let phys_raw_addr = (phys_addr.addr() + cpu::page_size() - 1) & !(cpu::page_size() - 1);
        let mut virt_drag_addr = VirtualAddress::new((phys_raw_addr + offset) as *mut c_void);
        let mut phys_drag_addr = PhysicalAddress::new(phys_raw_addr as *mut c_void);

        let mut token = Some(token);
        for _ in 0..size / cpu::page_size() {
            match self.early_create(
                phys_drag_addr,
                virt_drag_addr,
                Protection::RW,
                Mode::Kernel,
                token.unwrap(),
            ) {
                Ok(t) => {
                    token = Some(t);
                }
                Err((err, _)) => {
                    todo!(
                        "Handle error \"{}\" during Mapping::early_create_dev correctly",
                        err
                    );
                }
            };

            virt_drag_addr = unsafe { virt_drag_addr.byte_add(cpu::page_size()) };
            phys_drag_addr = unsafe { phys_drag_addr.byte_add(cpu::page_size()) };
        }

        let virt_addr = VirtualAddress::new((phys_addr.addr() + offset) as *mut c_void);
        Ok((virt_addr, token.unwrap()))
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
        let p_pt_0 = self.root.as_ref();
        let v_pt_0 = PageFrameAllocator::phys_to_virt(*p_pt_0);

        // Check first page table
        let vpn_0 = Self::offset(virt_addr, 0);
        let pte_0 = unsafe { v_pt_0.add(vpn_0).as_mut_ptr().as_mut().unwrap() };
        if !pte_0.is_valid() {
            return Err((MemoryError::InvalidAddress, token));
        }

        // Check second page table
        let (p_pts_1, p_pt_1, token) = match vpn_0 {
            0 | 1 | 2 | 3 => {
                let (user_page_tables, token) = self.user_pts_1.lock(token);
                let p_pt_1 = user_page_tables.0[vpn_0];
                (user_page_tables, p_pt_1, token)
            }
            508 | 509 | 510 | 511 => {
                let (kernel_page_tables, token) = self.kernel_pts_1.lock(token);
                let p_pt_1 = kernel_page_tables.0[vpn_0 - 508];
                (kernel_page_tables, p_pt_1, token)
            }
            _ => {
                return Err((MemoryError::InvalidAddress, token));
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
                let token = p_pts_1.unlock(token);
                return Err((MemoryError::NoSuchAddress, token));
            }
        };
        let v_pt_2 = PageFrameAllocator::phys_to_virt(p_pt_2);
        let vpn_2 = Self::offset(virt_addr, 2);
        let pte_2 = unsafe { v_pt_2.add(vpn_2).as_mut_ptr().as_mut().unwrap() };

        // Try to create mapping
        if !pte_2.is_valid() {
            let token = p_pts_1.unlock(token);
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
        let token = p_pts_1.unlock(token);
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
