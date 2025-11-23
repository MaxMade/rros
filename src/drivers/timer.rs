//! Timer using RISC-V `Timer` extension.

use crate::arch::cpu::CounterEnable;
use crate::arch::cpu::TimeCompare;
use crate::arch::cpu::CSR;
use crate::arch::sie::SIE;
use crate::arch::sip::SIP;
use crate::arch::time::Time;
use crate::drivers::driver::Driver;
use crate::drivers::driver::DriverError;
use crate::drivers::rtc::RTC;
use crate::kernel::time::MicroSecond;
use crate::sync::init_cell::InitCell;
use crate::sync::level::LevelDriver;
use crate::sync::level::LevelEpilogue;
use crate::sync::level::LevelInitialization;
use crate::sync::level::LevelPrologue;
use crate::trap::cause::Interrupt;
use crate::trap::cause::Trap;
use crate::trap::handler_interface::TrapContext;
use crate::trap::handlers::TrapHandler;
use crate::trap::handlers::TrapHandlers;

const TIMER_INTERVAL_US: usize = 5_000_000;

/// Global timer instance.
pub static TIMER: InitCell<Timer> = InitCell::new();

/// Timer using RISC-V `Timer` extension.
pub struct Timer {
    ticks_per_us: usize,
}

impl Timer {
    /// Get number of ticks per millisecond
    pub fn ticks_per_us(&self) -> usize {
        self.ticks_per_us
    }

    /// Activate timer with default timer interval.
    pub fn activate(&self, token: LevelDriver) -> LevelDriver {
        // Calculate number of ticks
        let ticks = (self.ticks_per_us * TIMER_INTERVAL_US) as u64;

        // Update compare register
        let mut time = Time::new(0);
        let mut time_compare = TimeCompare::new();
        time.read();
        time_compare.set(time.inner() + ticks);
        time_compare.write();

        token
    }
}

impl Driver for Timer {
    fn initiailize(
        token: LevelInitialization,
    ) -> Result<LevelInitialization, (DriverError, LevelInitialization)>
    where
        Self: Sized,
    {
        // Enable time register
        let mut counter_enable = CounterEnable::new();
        counter_enable.set_time_enabled(true);
        counter_enable.write();

        // Calibrate stime
        let mut time = Time::new(0);
        const NUM_US: u64 = 50_000;
        time.read();
        let stime_start = time.inner();
        let token = RTC
            .as_ref()
            .early_wait(MicroSecond::new(NUM_US as usize).convert(), token);
        time.read();
        let stime_end = time.inner();

        let ticks = match stime_end < stime_start {
            true => (u64::MAX - stime_start) + stime_end,
            false => stime_end - stime_start,
        };
        let ticks_per_us = ticks / NUM_US;

        // Initialize timer
        let mut timer = TIMER.get_mut(token);
        timer.ticks_per_us = ticks_per_us as usize;
        let token = timer.destroy();

        let token = unsafe { TIMER.finanlize(token) };

        // Register handler
        let token = TrapHandlers::register(
            Trap::Interrupt(Interrupt::TimerInterrupt),
            TIMER.as_ref(),
            token,
        );

        return Ok(token);
    }
}

impl TrapHandler for Timer {
    fn cause() -> Trap
    where
        Self: Sized,
    {
        Trap::Interrupt(Interrupt::TimerInterrupt)
    }

    fn prologue(&self, token: LevelPrologue) -> (bool, LevelPrologue) {
        // Disable timer interrupts
        let mut sie = SIE::new(0);
        sie.read();
        sie.mark_timer_interrupt_enabled(false);
        sie.write();

        // Clear timer pending bit
        let mut sip = SIP::new();
        sip.read();
        sip.clear_timer_interrupt_pending();
        sip.write();

        (true, token)
    }

    fn epilogue(&self, _state: Option<&mut TrapContext>, token: LevelEpilogue) -> LevelEpilogue {
        // Calculate number of ticks
        let ticks = (self.ticks_per_us * TIMER_INTERVAL_US) as u64;

        // Update compare register
        let mut time = Time::new(0);
        let mut time_compare = TimeCompare::new();
        time.read();
        time_compare.set(time.inner() + ticks);
        time_compare.write();

        // Re-enable timer interrupts
        let mut sie = SIE::new(0);
        sie.read();
        sie.mark_timer_interrupt_enabled(true);
        sie.write();

        token
    }
}
