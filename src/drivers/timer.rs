//! Driver for Google Goldfish Virtual RTC.
//!
//! Fore more details, see:
//! - [GOLDFISH-VIRTUAL-HARDWARE.TXT](https://android.googlesource.com/platform/external/qemu/+/master/docs/GOLDFISH-VIRTUAL-HARDWARE.TXT)
//! - [rtc-goldfish.c](https://github.com/torvalds/linux/blob/master/drivers/rtc/rtc-goldfish.c)
//! - [goldfish.h](https://github.com/torvalds/linux/blob/master/include/linux/goldfish.h)
//! - [timer-goldfish.h](https://github.com/torvalds/linux/blob/master/include/clocksource/timer-goldfish.h)

use crate::boot::device_tree::dt::DeviceTree;
use crate::drivers::mmio::MMIOSpace;
use crate::kernel::address::PhysicalAddress;
use crate::kernel::address::VirtualAddress;
use crate::sync::init_cell::InitCell;
use crate::sync::level::LevelInitialization;
use crate::sync::ticketlock::IRQTicketlock;
use crate::trap::cause::Interrupt;
use crate::trap::cause::Trap;
use crate::trap::handlers::TrapHandler;
use crate::trap::handlers::TRAP_HANDLERS;
use crate::trap::intc::INTERRUPT_CONTROLLER;

use super::driver::{Driver, DriverError};

/// Global timer instance.
pub static TIMER: InitCell<GoldfishTimer> = InitCell::new();

/// Timer interfal in nanoseconds (currently 100 ms)
pub const TIMER_INTERVAL_NS: usize = 100 * 1000;

#[allow(unused)]
#[derive(Debug)]
enum RegisterOffset {
    /// Get low bits of current time and update `TimeHigh`
    TimeLow = 0x00,
    /// Get high bits of current time at last `TimeLow` read
    TimeHigh = 0x04,
    /// Set low bits of alarm and activate it
    AlarmLow = 0x08,
    /// Set high bits of next alarm
    AlarmHigh = 0x0c,
    /// Enable alarm interrupt
    IrqEnabled = 0x10,
    /// Disarm an existing alarm
    ClearAlarm = 0x14,
    /// Get alarm status (running or not)
    AlarmStatus = 0x18,
    /// Clear interrupt
    ClearInterrupt = 0x1c,
}

/// Driver for Google Goldfish RTC.
pub struct GoldfishTimer {
    /// Configuration space.
    pub(in crate::drivers::timer) config_space: IRQTicketlock<MMIOSpace>,
    /// Interrupt configuration.
    pub(in crate::drivers::timer) interrupt: Interrupt,
}

impl Driver for GoldfishTimer {
    fn initiailize(
        token: LevelInitialization,
    ) -> Result<LevelInitialization, (DriverError, LevelInitialization)>
    where
        Self: Sized,
    {
        // Search device tree for node describing ns16550a
        let device_tree = DeviceTree::get_dt();
        let device = match device_tree.get_node_by_compatible_property("goldfish-rtc") {
            Some(device) => device,
            None => return Err((DriverError::NonCompatibleDevice, token)),
        };

        // Get address and size of configuration space
        let reg_property = match device.property_iter().filter(|p| p.name == "reg").next() {
            Some(reg_property) => reg_property,
            None => {
                return Err((DriverError::NonCompatibleDevice, token));
            }
        };
        let (raw_address, raw_length) = match reg_property.into_addr_length_iter().next() {
            Some((raw_address, raw_length)) => (raw_address, raw_length),
            None => {
                return Err((DriverError::NonCompatibleDevice, token));
            }
        };
        let _phys_addres = PhysicalAddress::from(raw_address as *mut u8);
        let size = raw_length;

        // TODO: Convert physical address to virtual address
        let virt_address = VirtualAddress::from(raw_address as *mut u8);

        // Create configuration space
        let mmio_space = unsafe { MMIOSpace::new(virt_address, size) };

        // Read interrupt configuration
        let interrupts = match device
            .property_iter()
            .filter(|p| p.name == "interrupts")
            .next()
        {
            Some(interrupts) => interrupts,
            None => {
                return Err((DriverError::NonCompatibleDevice, token));
            }
        };
        let mut interrupts = interrupts.into_interrupt_iter();

        // Process (single) interrupt
        let interrupt = interrupts.next().unwrap();
        let interrupt = Interrupt::Interrupt(u64::from(interrupt));
        assert!(interrupts.next().is_none());

        // Get locked driver
        let (uart, token) = TIMER.as_mut(token);
        let mut config_space = uart.config_space.init_lock(token);

        // Update config space
        *config_space = mmio_space;

        // Write interrupt configuration
        uart.interrupt = interrupt;

        // Configure alarm
        config_space
            .store(
                RegisterOffset::AlarmHigh as usize,
                (TIMER_INTERVAL_NS >> 32) as u32,
            )
            .unwrap();
        config_space
            .store(RegisterOffset::AlarmLow as usize, TIMER_INTERVAL_NS as u32)
            .unwrap();

        // Configure time
        config_space
            .store(RegisterOffset::TimeHigh as usize, 0u32)
            .unwrap();
        config_space
            .store(RegisterOffset::TimeLow as usize, 0u32)
            .unwrap();

        // Enable interrupts
        config_space
            .store(RegisterOffset::IrqEnabled as usize, 1u32)
            .unwrap();

        // Unlock driver
        let token = config_space.init_unlock();

        // Configure interrupt controller
        let token = INTERRUPT_CONTROLLER.configure(interrupt, token);
        let token = INTERRUPT_CONTROLLER.unmask(interrupt, token);

        // Register handler
        let (trap_handlers, token) = TRAP_HANDLERS.as_mut(token);
        let (uart, token) = TIMER.as_mut(token);
        let token = trap_handlers.register(Trap::Interrupt(interrupt), uart, token);

        // Finalize initialization
        let token = unsafe { TIMER.finanlize(token) };

        return Ok(token);
    }
}

impl TrapHandler for GoldfishTimer {
    fn cause() -> crate::trap::cause::Trap
    where
        Self: Sized,
    {
        Trap::Interrupt(TIMER.as_ref().interrupt)
    }

    fn prologue(
        &self,
        token: crate::sync::level::LevelPrologue,
    ) -> (bool, crate::sync::level::LevelPrologue) {
        // Lock driver
        let (mut config_space, token) = TIMER.as_ref().config_space.lock(token);

        config_space
            .store(RegisterOffset::ClearInterrupt as usize, 1u32)
            .unwrap();

        // Unlock driver
        let token = config_space.unlock(token);

        (false, token)
    }
}
