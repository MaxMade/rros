//! Page-Frame Allocator.

use core::ffi::c_void;

use crate::kernel::address::Address;
use crate::kernel::address::PhysicalAddress;
use crate::kernel::address::VirtualAddress;
use crate::kernel::compiler;
use crate::kernel::cpu;
use crate::mm::error::MemoryError;
use crate::sync::level::LevelInitialization;
use crate::sync::level::LevelPaging;
use crate::sync::ticketlock::TicketlockPaging;

/// Global [`PageFrameAllocator`] instance.
pub static PAGE_FRAME_ALLOCATOR: PageFrameAllocator = PageFrameAllocator::new();

const MAX_SIZE: usize = 0x1000000;

const ADDRESS_SHIFT: u64 = 0xffffffff00000000;

/// Page-Frame Allocator capable of managing at most 16 MiB.
pub struct PageFrameAllocator {
    state: TicketlockPaging<[u64; 64]>,
}

impl PageFrameAllocator {
    const fn new() -> Self {
        Self {
            state: TicketlockPaging::new([0; 64]),
        }
    }

    /// Initialize global [`PAGE_FRAME_ALLOCATOR`] instance.
    pub fn initialize(token: LevelInitialization) -> LevelInitialization {
        let mut allocator_state = PAGE_FRAME_ALLOCATOR.state.init_lock(token);

        let start_addr = Self::virt_to_phys(compiler::pages_mem_start());
        assert!(start_addr.addr() % cpu::page_size() == 0);

        let size = usize::min(MAX_SIZE, compiler::pages_mem_size());
        assert!(size % cpu::page_size() == 0);

        let mut state = [0u64; 64];
        for i in 0..size / cpu::page_size() {
            let idx = i / u64::BITS as usize;
            let offset = i / u64::BITS as usize;

            state[idx] |= 1 << offset;
        }
        *allocator_state = state;

        allocator_state.init_unlock()
    }

    /// Try to allocate a new page
    pub fn allocate(
        &self,
        token: LevelPaging,
    ) -> Result<(PhysicalAddress<c_void>, LevelPaging), (MemoryError, LevelPaging)> {
        // Lock allocator
        let (mut allocator_state, token) = self.state.lock(token);

        // Search for available page
        for (idx, state) in allocator_state.iter_mut().enumerate() {
            for offset in 0..u64::BITS as usize {
                if *state & 1 << offset != 0 {
                    // Mark page as occupied
                    *state &= !(1 << offset);

                    // Unlock allocator
                    let token = allocator_state.unlock(token);

                    // Calculate address of page
                    let page_offset = (u64::BITS as usize * idx + offset) * cpu::page_size();
                    let mut v_page = compiler::pages_mem_start().add(page_offset);
                    let p_page = Self::virt_to_phys(v_page);

                    // Sanity check
                    assert!(v_page.addr() % cpu::page_size() == 0);
                    assert!(v_page >= compiler::pages_mem_start());
                    assert!(v_page < compiler::pages_mem_end());

                    // Zero page
                    unsafe { v_page.as_mut_ptr().write_bytes(0, cpu::page_size()) };

                    return Ok((p_page, token));
                }
            }
        }

        // Unlock allocator
        let token = allocator_state.unlock(token);
        return Err((MemoryError::OutOfMemory, token));
    }

    /// Free allocated page
    ///
    /// # Safety
    /// This function is unsafe because undefined behavior can result if ...
    /// - `ptr` refers to a block of memory currently allocated via this allocator.
    /// - the references page is still in use.
    pub unsafe fn free(self, page: PhysicalAddress<c_void>, token: LevelPaging) -> LevelPaging {
        let p_page = page;
        let v_page = Self::phys_to_virt(p_page);

        // Sanity check: Is page valid?
        assert!(v_page.addr() % cpu::page_size() == 0);
        assert!(v_page >= compiler::pages_mem_start());
        assert!(v_page < compiler::pages_mem_end());

        // Calculate offset
        let page_offset = v_page.addr() - compiler::pages_mem_start().addr();
        let idx = (page_offset / cpu::page_size()) / u64::BITS as usize;
        let offset = (page_offset / cpu::page_size()) % u64::BITS as usize;

        // Lock allocator
        let (mut allocator_state, token) = self.state.lock(token);

        // Sanity check: Was page allocated
        assert!(allocator_state[idx] & 1 << offset == 0);

        // Mark page as free
        allocator_state[idx] |= 1 << offset;

        // Unlock allocator
        let token = allocator_state.unlock(token);
        return token;
    }

    /// Convert [`VirtualAddress`] returned by
    /// [`allocate`](crate::mm::page_allocator::PageFrameAllocator::allocate), to a [`PhysicalAddress`].
    pub fn virt_to_phys<T>(virt_addr: VirtualAddress<T>) -> PhysicalAddress<T> {
        // Sanity check: Refers virt_addr a valid page?
        assert!(virt_addr.addr() % cpu::page_size() == 0);
        assert!(virt_addr >= unsafe { compiler::pages_mem_start().cast() });
        assert!(virt_addr < unsafe { compiler::pages_mem_end().cast() });

        unsafe { PhysicalAddress::new(virt_addr.byte_sub(ADDRESS_SHIFT as usize).as_mut_ptr()) }
    }

    /// Convert [`PhysicalAddress`] returned by
    /// [`allocate`](crate::mm::page_allocator::PageFrameAllocator::allocate), to a [`VirtualAddress`].
    pub fn phys_to_virt<T>(phys_addr: PhysicalAddress<T>) -> VirtualAddress<T> {
        let virt_addr =
            unsafe { VirtualAddress::new(phys_addr.byte_add(ADDRESS_SHIFT as usize).as_mut_ptr()) };

        // Sanity check: Refers virt_addr a valid page?
        assert!(virt_addr.addr() % cpu::page_size() == 0);
        assert!(virt_addr >= unsafe { compiler::pages_mem_start().cast() });
        assert!(virt_addr < unsafe { compiler::pages_mem_end().cast() });
        virt_addr
    }
}

impl Default for PageFrameAllocator {
    fn default() -> Self {
        Self::new()
    }
}
