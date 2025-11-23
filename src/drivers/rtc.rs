//! Driver for Google Goldfish Virtual RTC.
//!
//! Fore more details, see:
//! - [GOLDFISH-VIRTUAL-HARDWARE.TXT](https://android.googlesource.com/platform/external/qemu/+/master/docs/GOLDFISH-VIRTUAL-HARDWARE.TXT)
//! - [rtc-goldfish.c](https://github.com/torvalds/linux/blob/master/drivers/rtc/rtc-goldfish.c)
//! - [goldfish.h](https://github.com/torvalds/linux/blob/master/include/linux/goldfish.h)
//! - [timer-goldfish.h](https://github.com/torvalds/linux/blob/master/include/clocksource/timer-goldfish.h)

use core::ffi::c_void;

use crate::boot::device_tree::dt::DeviceTree;
use crate::drivers::driver::Driver;
use crate::drivers::driver::DriverError;
use crate::drivers::mmio::MMIOSpace;
use crate::kernel::address::Address;
use crate::kernel::address::PhysicalAddress;
use crate::kernel::time::NanoSecond;
use crate::kernel::time::TimeUnits;
use crate::mm::mapping::KERNEL_VIRTUAL_MEMORY_SYSTEM;
use crate::sync::init_cell::InitCell;
use crate::sync::level::LevelDriver;
use crate::sync::level::LevelInitialization;
use crate::sync::ticketlock::TicketlockDriver;

/// Global timer instance.
pub static RTC: InitCell<RealTimeClock> = InitCell::new();

/// Timer interfal in nanoseconds (currently 1 second)
pub const TIMER_INTERVAL_NS: u64 = 1 * 1000 * 1000 * 1000;

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
pub struct RealTimeClock {
    /// Configuration space.
    config_space: TicketlockDriver<MMIOSpace>,
}

impl RealTimeClock {
    fn __wait(config_space: &MMIOSpace, time: NanoSecond) {
        // Calculate expected time stamp
        let time_low: u32 = config_space.load(RegisterOffset::TimeLow as usize).unwrap();
        let time_high: u32 = config_space
            .load(RegisterOffset::TimeHigh as usize)
            .unwrap();
        let cur_timer = ((time_high as u64) << 32) | (time_low as u64);

        let time_start = NanoSecond::new(usize::try_from(cur_timer).unwrap());
        let time_end = time_start + time;

        loop {
            let time_low: u32 = config_space.load(RegisterOffset::TimeLow as usize).unwrap();
            let time_high: u32 = config_space
                .load(RegisterOffset::TimeHigh as usize)
                .unwrap();
            let time = ((time_high as u64) << 32) | (time_low as u64);
            let time_cur = NanoSecond::new(usize::try_from(time).unwrap());

            if time_start < time_end {
                if time_cur > time_end || time_cur < time_start {
                    break;
                }
            } else {
                if time_cur > time_start && time_cur < time_end {
                    break;
                }
            }
        }
    }

    /// Wait for a given time period during initialization.
    pub fn early_wait(&self, time: NanoSecond, token: LevelInitialization) -> LevelInitialization {
        // Lock driver
        let config_space = self.config_space.init_lock(token);

        Self::__wait(&config_space, time);

        // Unlock driver
        let token = config_space.init_unlock();
        token
    }

    /// Wait for a given time period.
    pub fn wait(&self, time: NanoSecond, token: LevelDriver) -> LevelDriver {
        // Lock driver
        let (config_space, token) = self.config_space.lock(token);

        Self::__wait(&config_space, time);

        // Unlock driver
        let token = config_space.unlock(token);
        token
    }
}

impl Driver for RealTimeClock {
    fn initiailize(
        token: LevelInitialization,
    ) -> Result<LevelInitialization, (DriverError, LevelInitialization)>
    where
        Self: Sized,
    {
        // Search device tree for node describing ns16550a
        let (device_tree, token) = DeviceTree::get_dt(token);
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
        let phys_address = PhysicalAddress::from(raw_address as *mut c_void);
        let size = raw_length;

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

        // Create configuration space
        let mmio_space = unsafe { MMIOSpace::new(virt_address, size) };

        // Get locked driver
        let mut uart = RTC.get_mut(token);

        // Update config space
        let config_space = uart.config_space.get_mut();
        *config_space = mmio_space;

        // Unlock driver
        let token = uart.destroy();

        // Finalize initialization
        let token = unsafe { RTC.finanlize(token) };

        return Ok(token);
    }
}
