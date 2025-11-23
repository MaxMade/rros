//! RISC-V Platform-Level Interrupt Controller.
//!
//! Fore more details, see
//! - [RISC-V Platform-Level Interrupt Controller
//! Specification](https://github.com/riscv/riscv-plic-spec/blob/master/riscv-plic-1.0.0.pdf)
//! - [SiFive U54-MC Core Complex Manual](https://static.dev.sifive.com/U54-MC-RVCoreIP.pdf)

use core::mem;
use core::ptr;

use crate::boot::device_tree::dt::DeviceTree;
use crate::config;
use crate::drivers::driver::{Driver, DriverError};
use crate::drivers::mmio::MMIOSpace;
use crate::kernel::address::{Address, PhysicalAddress, VirtualAddress};
use crate::kernel::cpu::ExecutionMode;
use crate::kernel::cpu::HartID;
use crate::kernel::cpu_map;
use crate::sync::level::LevelDriver;
use crate::sync::level::LevelInitialization;
use crate::sync::ticketlock::IRQTicketlockDriver;
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
    deliviery_modes: [Option<DelivieryMode>; NUM_INTERRUPT_SOURCES],
}

/// Driver for PLIC of SiFive U5 Coreplex platform
pub struct InterruptController(IRQTicketlockDriver<PLIC>);

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

/// Interrupt deliviery mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DelivieryMode {
    /// Route pending interrupt to single hart in round-robin fashion.
    Unicast,
    /// Route pending interrupt to every hart.
    Broadcast,
}

impl PLIC {
    fn set_context_priority_threashold(
        &mut self,
        hart: HartID,
        mode: ExecutionMode,
        priority_threashold: u32,
    ) {
        const PRIORITY_THREASHOLD_OFFSET: usize = RegisterOffset::Priority as usize;

        // Register map (relative to [`PriorityThreashold`]):
        //
        // |--------|----------------------------------|
        // | 0x0000 | Hart 0 M-mode priority threshold |
        // | 0x0004 | Hart 0 M-mode claim/complete     |
        // |--------|----------------------------------|
        // | 0x1000 | Hart 1 M-mode priority threshold |
        // | 0x1004 | Hart 1 M-mode claim/complete     |
        // |--------|----------------------------------|
        // | 0x2000 | Hart 1 S-mode priority threshold |
        // | 0x2004 | Hart 1 S-mode claim/complete     |
        // |--------|----------------------------------|
        // | 0x3000 | Hart 2 M-mode priority threshold |
        // | 0x3004 | Hart 2 M-mode claim/complete     |
        // |--------|----------------------------------|
        // | 0x4000 | Hart 2 S-mode priority threshold |
        // | 0x4004 | Hart 2 S-mode claim/complete     |
        // |--------|----------------------------------|
        // | 0x5000 | Hart 3 M-mode priority threshold |
        // | 0x5004 | Hart 3 M-mode claim/complete     |
        // |--------|----------------------------------|
        // | 0x6000 | Hart 3 S-mode priority threshold |
        // | 0x6004 | Hart 3 S-mode claim/complete     |
        // |--------|----------------------------------|
        // | 0x7000 | Hart 4 M-mode priority threshold |
        // | 0x7004 | Hart 4 M-mode claim/complete     |
        // |--------|----------------------------------|
        // | 0x8000 | Hart 4 S-mode priority threshold |
        // | 0x8004 | Hart 4 S-mode claim/complete     |
        // |--------|----------------------------------|
        //
        // (For more details, see 8.1 Memory Map of SiFive U54-MC Core Complex Manual)

        // Special case: Hart 0
        if hart.raw() == 0 {
            match mode {
                ExecutionMode::User => panic!("Unable to configure priority threashold of PLIC for user mode!"),
                ExecutionMode::Supervisor => panic!("Unable to configure priority threashold of PLIC for supervisor mode on hart 0!"),
                ExecutionMode::Machine =>  self.config_space
                    .store(PRIORITY_THREASHOLD_OFFSET, 0u32)
                    .unwrap(),
            }
        }

        match mode {
            ExecutionMode::User => {
                panic!("Unable to configure priority threashold of PLIC for user mode!")
            }
            ExecutionMode::Supervisor => self
                .config_space
                .store(
                    PRIORITY_THREASHOLD_OFFSET + usize::try_from(hart.raw()).unwrap() * 0x2000,
                    priority_threashold,
                )
                .unwrap(),
            ExecutionMode::Machine => self
                .config_space
                .store(
                    PRIORITY_THREASHOLD_OFFSET
                        + 0x1000
                        + (usize::try_from(hart.raw()).unwrap() - 1) * 0x2000,
                    priority_threashold,
                )
                .unwrap(),
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
}

impl InterruptController {
    /// Create a new uninitialized `InterruptController` instance.
    pub const fn new() -> Self {
        unsafe {
            Self(IRQTicketlockDriver::new(PLIC {
                config_space: MMIOSpace::new(VirtualAddress::new(ptr::null_mut()), 0),
                num_intr_sources: 0,
                num_harts: 0,
                harts: [HartID::new(0); config::MAX_CPU_NUM],
                deliviery_modes: [None; NUM_INTERRUPT_SOURCES],
            }))
        }
    }

    /// Configure [`InterruptController`] for given [`Interrupt`].
    pub fn configure(
        &self,
        interrupt: Interrupt,
        mode: DelivieryMode,
        token: LevelInitialization,
    ) -> LevelInitialization {
        // Colculate index.
        let idx = usize::try_from(interrupt).unwrap();

        // Lock driver
        let mut plic = self.0.init_lock(token);

        // Update deliviery mode
        assert!(plic.deliviery_modes[idx].is_none());
        plic.deliviery_modes[idx] = Some(mode);

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
        plic.set_interrupt_priority(usize::try_from(interrupt).unwrap(), 0);
        plic.init_unlock()
    }

    /// Send end-of-interrupt signal.
    pub unsafe fn end_of_interrupt(&self, interrupt: Interrupt) {
        todo!();
    }
}

impl Driver for InterruptController {
    fn initiailize(
        token: crate::sync::level::LevelInitialization,
    ) -> Result<LevelInitialization, (DriverError, LevelInitialization)> {
        // Search device tree for node describing ns16550a
        let device_tree = DeviceTree::get_dt();
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
        let mut phys_addres = PhysicalAddress::from(raw_address as *mut u8);
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

        // TODO: Convert physical address to virtual address
        let virt_address = VirtualAddress::from(phys_addres.as_mut_ptr());

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

        //Set Threashold of each interrupt source (for each context) to 0
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
