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
                    let page = Self::virt_to_phys(compiler::pages_mem_start()).add(page_offset);
                    assert!(page.addr() % cpu::page_size() == 0);
                    assert!(page >= Self::virt_to_phys(compiler::pages_mem_start()));
                    assert!(page < Self::virt_to_phys(compiler::pages_mem_end()));

                    return Ok((page, token));
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
        // Sanity check: Is page valid?
        assert!(page.addr() % cpu::page_size() == 0);
        assert!(page >= Self::virt_to_phys(compiler::pages_mem_start()));
        assert!(page < Self::virt_to_phys(compiler::pages_mem_end()));

        // Calculate offset
        let page_offset = page.addr() - Self::virt_to_phys(compiler::pages_mem_start()).addr();
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

    fn virt_to_phys<T>(phys: VirtualAddress<T>) -> PhysicalAddress<T> {
        unsafe { PhysicalAddress::new(phys.byte_sub(ADDRESS_SHIFT as usize).as_mut_ptr()) }
    }
}

impl Default for PageFrameAllocator {
    fn default() -> Self {
        Self::new()
    }
}
