//! Interface for entering/leaving epilogue level.

use core::mem;
use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering;

use crate::config;
use crate::kernel::cpu;
use crate::sync::level::LevelEpilogue;
use crate::trap::handlers::TrapHandlers;

use super::level::Level;

static EPILOGUE_STATE: [AtomicBool; config::MAX_CPU_NUM] =
    unsafe { mem::transmute([false; config::MAX_CPU_NUM]) };

/// Try to enter `epilogue` level.
pub fn try_enter() -> Option<LevelEpilogue> {
    // Disable interrupts
    let interrupt_enabled = cpu::interrupts_enabled();
    unsafe { cpu::disable_interrupts() };

    // Try to acquire epilogue
    let success = EPILOGUE_STATE[cpu::current().raw()]
        .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
        .is_ok();

    // Re-enable interrupts if necessary
    if interrupt_enabled {
        unsafe { cpu::enable_interrupts() };
    }

    if success {
        // Produce new token
        return unsafe { Some(LevelEpilogue::create()) };
    } else {
        return None;
    }
}

/// Leave `epilogue` level.
pub fn leave(token: LevelEpilogue) {
    // Disable interrupts
    let (interrupt_flag, token) = cpu::save_and_disable_interrupts(token);

    // Execute prologue
    let mut epilogue_token = Some(token);
    while let (Some(trap), token) = TrapHandlers::dequeue(epilogue_token.take().unwrap()) {
        // Get corresponding handler
        let (handler, token) = TrapHandlers::get(trap, token);
        epilogue_token = Some(token);

        // Enable interrupts
        unsafe { cpu::enable_interrupts() };

        // Execute epilogue
        let epilogue_token = unsafe { LevelEpilogue::create() };
        handler.epilogue(None, epilogue_token);

        // Disable interrupts
        unsafe { cpu::disable_interrupts() };
    }

    // Release epilogue
    EPILOGUE_STATE[cpu::current().raw()]
        .compare_exchange(true, false, Ordering::Relaxed, Ordering::Relaxed)
        .unwrap();

    // Re-enable interrupts if necessary
    cpu::restore_interrupts(interrupt_flag);

    // Consume token
    let _ = token;
}
