//! RISC-V Platform-Level Interrupt Controller.
//!
//! Fore more details, see
//! - [RISC-V Platform-Level Interrupt Controller
//! Specification](https://github.com/riscv/riscv-plic-spec/blob/master/riscv-plic-1.0.0.pdf)
//! - [SiFive U54-MC Core Complex Manual](https://static.dev.sifive.com/U54-MC-RVCoreIP.pdf)

use core::ffi::c_void;
use core::mem;
use core::ptr;

use crate::arch::cpu::ExecutionMode;
use crate::boot::device_tree::dt::DeviceTree;
use crate::config;
use crate::drivers::driver::{Driver, DriverError};
use crate::drivers::mmio::MMIOSpace;
use crate::kernel::address::{Address, PhysicalAddress, VirtualAddress};
use crate::kernel::cpu;
use crate::kernel::cpu_map;
use crate::kernel::cpu_map::HartID;
use crate::mm::mapping::KERNEL_VIRTUAL_MEMORY_SYSTEM;
use crate::sync::level::LevelInitialization;
use crate::sync::level::LevelPrologue;
use crate::sync::ticketlock::IRQTicketlock;
use crate::trap::cause::Interrupt;

/// Global interrupt controller instance.
pub static INTERRUPT_CONTROLLER: InterruptController = InterruptController::new();

/// Total number of interrupt sources.
///
/// # See
/// `Chapter 3` of `RISC-V Platform-Level Interrupt Controller Specification`
const NUM_INTERRUPT_SOURCES: usize = 1024;

struct PLIC {
    config_space: MMIOSpace,
    num_intr_sources: usize,
    num_harts: usize,
    harts: [HartID; config::MAX_CPU_NUM],
}

/// Driver for PLIC of SiFive U5 Coreplex platform
pub struct InterruptController(IRQTicketlock<PLIC>);

/// Register offsets (in bytes) relative to start of configuration space.
#[derive(Debug)]
enum RegisterOffset {
    /// Priority of the interrupt source.
    Priority = 0x0,
    /// Pending bits of interrupt source.
    Pending = 0x1000isize,
    /// Enable bits for interrupt source per interrupt context.
    Enable = 0x2000isize,
    /// Priority threshold per interrupt context.
    PriorityThreashold = 0x20_0000isize,
    /// Claim & complete handle per interrupt context.
    ClaimComplete = 0x20_0004isize,
}

impl PLIC {
    fn set_context_priority_threashold(
        &mut self,
        hart: HartID,
        mode: ExecutionMode,
        priority_threashold: u32,
    ) {
        const PRIORITY_THREASHOLD_OFFSET: usize = RegisterOffset::PriorityThreashold as usize;
        match mode {
            ExecutionMode::Machine => self
                .config_space
                .store(
                    PRIORITY_THREASHOLD_OFFSET + usize::try_from(hart.raw()).unwrap() * 0x2000,
                    priority_threashold,
                )
                .unwrap(),
            ExecutionMode::Supervisor => self
                .config_space
                .store(
                    PRIORITY_THREASHOLD_OFFSET
                        + usize::try_from(hart.raw()).unwrap() * 0x2000
                        + 0x1000,
                    priority_threashold,
                )
                .unwrap(),
            _ => {
                panic!("Unable to configure priority threashold of PLIC for user mode!")
            }
        }
    }

    fn set_interrupt_priority(&mut self, interrupt: usize, priority: u32) {
        // Register map (relative to [`Priority`]):
        //
        // | 0x0C00 0000 | reserved            |
        // | 0x0C00 0004 | source 1 priority   |
        // | 0x0C00 0008 | source 2 priority   |
        // | ...         | ...                 |
        // | 0x0C00 0800 | source 511 priority |
        //
        // (For more details, see 8.1 Memory Map of SiFive U54-MC Core Complex Manual)
        const PRIORITY_OFFSET: usize = RegisterOffset::Priority as usize;
        self.config_space
            .store(
                PRIORITY_OFFSET + interrupt * mem::size_of::<u32>(),
                priority,
            )
            .unwrap();
    }

    fn set_interrupt_enabled(
        &mut self,
        interrupt: usize,
        hart: HartID,
        mode: ExecutionMode,
        enabled: bool,
    ) {
        const ENABLE_OFFSET: usize = RegisterOffset::Enable as usize;

        let hart_id = usize::try_from(hart.raw()).unwrap();
        let bit_offset = interrupt % usize::try_from(u32::BITS).unwrap();
        let byte_offset = interrupt / usize::try_from(u32::BITS).unwrap();
        let context_offset = match mode {
            ExecutionMode::Machine => 0x100 * hart_id,
            ExecutionMode::Supervisor => 0x100 * hart_id + 0x80,
            _ => panic!("Unable to set enable bit of PLIC for user mode!"),
        };

        // Read mask
        let mut mask: u32 = self
            .config_space
            .load(ENABLE_OFFSET + context_offset + byte_offset)
            .unwrap();

        // Modify mask
        if enabled {
            mask |= 1 << bit_offset;
        } else {
            mask &= !(1 << bit_offset);
        }

        // Write mask
        self.config_space
            .store(ENABLE_OFFSET + context_offset + byte_offset, mask)
            .unwrap()
    }

    fn claim(&mut self, hart: HartID, mode: ExecutionMode) -> Interrupt {
        const CLAIM_OFFSET: usize = RegisterOffset::ClaimComplete as usize;

        let hart_id = usize::try_from(hart.raw()).unwrap();
        let context_offset = match mode {
            ExecutionMode::Machine => 2 * hart_id * 0x1000,
            ExecutionMode::Supervisor => 2 * hart_id * 0x1000 + 0x1000,
            _ => panic!("Unable to set enable bit of PLIC for user mode!"),
        };

        // Read pending interrupt
        let interrupt: u32 = self
            .config_space
            .load(CLAIM_OFFSET + context_offset)
            .unwrap();
        if interrupt == 0 {
            panic!("No such interrupt to be claimed!");
        }

        Interrupt::Interrupt(interrupt.into())
    }
}

impl InterruptController {
    /// Create a new uninitialized `InterruptController` instance.
    pub const fn new() -> Self {
        unsafe {
            Self(IRQTicketlock::new(PLIC {
                config_space: MMIOSpace::new(VirtualAddress::new(ptr::null_mut()), 0),
                num_intr_sources: 0,
                num_harts: 0,
                harts: [HartID::new(0); config::MAX_CPU_NUM],
            }))
        }
    }

    /// Configure [`InterruptController`] for given [`Interrupt`].
    pub fn configure(
        &self,
        interrupt: Interrupt,
        token: LevelInitialization,
    ) -> LevelInitialization {
        // Colculate index.
        let idx = usize::try_from(interrupt).unwrap();

        // Lock driver
        let mut plic = self.0.init_lock(token);

        // All hart except from 0 are routable!
        let curr_logical_id = cpu::current();
        let hart_id = match curr_logical_id.raw() {
            0 => *plic
                .harts
                .iter()
                .find(|hart_id| hart_id.raw() != 0)
                .unwrap(),
            _ => cpu_map::lookup_hart_id(curr_logical_id),
        };
        plic.set_interrupt_enabled(idx, hart_id, ExecutionMode::Supervisor, true);

        // Unlock driver
        plic.init_unlock()
    }

    /// Mask [`Interrupt`].
    pub fn mask(&self, interrupt: Interrupt, token: LevelInitialization) -> LevelInitialization {
        let mut plic = self.0.init_lock(token);
        plic.set_interrupt_priority(usize::try_from(interrupt).unwrap(), 0);
        plic.init_unlock()
    }

    /// Unmask [`Interrupt`].
    pub fn unmask(&self, interrupt: Interrupt, token: LevelInitialization) -> LevelInitialization {
        let mut plic = self.0.init_lock(token);
        plic.set_interrupt_priority(usize::try_from(interrupt).unwrap(), 1);
        plic.init_unlock()
    }

    /// Get pending interrupt.
    pub fn source(&self, token: LevelPrologue) -> (Interrupt, LevelPrologue) {
        // Get current hart
        let hart_id = cpu_map::lookup_hart_id(cpu::current());

        // Lock PLIC
        let (mut plic, token) = self.0.lock(token);

        // Claim interrupt
        let interrupt = plic.claim(hart_id, ExecutionMode::Supervisor);

        // Unlock PLIC
        let token = plic.unlock(token);

        return (interrupt, token);
    }

    /// Send end-of-interrupt signal.
    pub fn end_of_interrupt(&self, interrupt: Interrupt, token: LevelPrologue) -> LevelPrologue {
        const CLAIM_COMPLETE_OFFSET: usize = RegisterOffset::ClaimComplete as usize;

        // Lock PLIC
        let (mut plic, token) = self.0.lock(token);

        // Get current hart
        let hart_id = cpu_map::lookup_hart_id(cpu::current());
        let hart_id = usize::try_from(hart_id.raw()).unwrap();

        // Calculate context offset
        let context_offset = 2 * hart_id * 0x1000 + 0x1000;

        // Write back interupt to complete
        plic.config_space
            .store(
                CLAIM_COMPLETE_OFFSET + context_offset,
                usize::try_from(interrupt).unwrap() as u32,
            )
            .unwrap();

        // Unlock PLIC
        let token = plic.unlock(token);
        token
    }
}

impl Driver for InterruptController {
    fn initiailize(
        token: crate::sync::level::LevelInitialization,
    ) -> Result<LevelInitialization, (DriverError, LevelInitialization)> {
        // Search device tree for node describing ns16550a
        let (device_tree, token) = DeviceTree::get_dt(token);
        let device = match device_tree.get_node_by_compatible_property("sifive,plic-1.0.0") {
            Some(device) => device,
            None => return Err((DriverError::NonCompatibleDevice, token)),
        };

        // Get address and size of configuration space
        let reg_property = match device.property_iter().filter(|p| p.name == "reg").next() {
            Some(reg_property) => reg_property,
            None => return Err((DriverError::NonCompatibleDevice, token)),
        };
        let (raw_address, raw_length) = match reg_property.into_addr_length_iter().next() {
            Some((raw_address, raw_length)) => (raw_address, raw_length),
            None => return Err((DriverError::NonCompatibleDevice, token)),
        };
        let phys_address = PhysicalAddress::from(raw_address as *mut c_void);
        let size = raw_length;

        // Parse maximum number of supported interrupt sources
        let ndev = match device
            .property_iter()
            .filter(|p| p.name == "riscv,ndev")
            .next()
        {
            Some(ndev) => ndev,
            None => return Err((DriverError::NonCompatibleDevice, token)),
        };

        let num_intr_sources = match ndev.get_value() {
            crate::boot::device_tree::property::PropertyValue::U32(ndev) => ndev as usize,
            _ => return Err((DriverError::NonCompatibleDevice, token)),
        };

        // Convert physical address to virtual address
        let (virt_address, token) =
            match KERNEL_VIRTUAL_MEMORY_SYSTEM
                .as_ref()
                .early_create_dev(phys_address, size, token)
            {
                Ok((virt_address, token)) => (unsafe { virt_address.cast() }, token),
                Err((_, token)) => {
                    return Err((DriverError::NoDataAvailable, token));
                }
            };

        // Acquire lock gurad for driver (MMIO space)
        let mut plic = INTERRUPT_CONTROLLER.0.init_lock(token);

        // Update MMIO Space
        unsafe { plic.config_space.relocate(virt_address, size) };

        // Update number of interrupt sources
        plic.num_intr_sources = num_intr_sources;

        // Configure list of online harts
        for (_, hart_id) in cpu_map::iter() {
            let idx = plic.num_harts;
            plic.harts[idx] = hart_id;
            plic.num_harts += 1;
        }

        // Set Priority of each interrupt source to 0
        for i in 1..NUM_INTERRUPT_SOURCES {
            plic.set_interrupt_priority(i, 0);
        }

        // Set Threashold of each interrupt source (for each context) to 0
        for (_, hart_id) in cpu_map::iter() {
            if hart_id.raw() != 0 {
                plic.set_context_priority_threashold(hart_id, ExecutionMode::Supervisor, 0);
            }
        }

        // Release lock gurad
        let token = plic.init_unlock();

        Ok(token)
    }
}
