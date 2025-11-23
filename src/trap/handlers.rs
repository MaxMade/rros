//! Software-Abstractions for trap handlers.

use core::mem;

use crate::kernel::cpu::STVec;
use crate::kernel::cpu::STVecMode;
use crate::sync::level::LevelEpilogue;
use crate::sync::level::LevelInitialization;
use crate::sync::level::LevelPrologue;
use crate::trap::cause::Trap;

extern "C" {
    fn __trap_entry();
}

/// Interface for handling traps -  suitable for interrupts and exceptions.
pub trait TrapHandler {
    /// Get [`Trap`] cause.
    fn cause() -> Trap;

    /// High-priority task of Prologue/Epilogue model.
    ///
    /// Every interrupt capable driver has to implement the `prologue` which takes care of any task
    /// why must be executed immediately upon receiving the corresponding interrupt. This
    /// enables low-latency interrupt handling, but in turn implies the strict requirements for the
    /// handler: It *must* be as short as possible as interrupts are disabled during execution.
    /// Thus, no locking/blocking/waiting/... is allowed! For such tasks, an optional `epilogue`
    /// can be requested by return `true`.
    fn prologue(&self, token: LevelPrologue) -> bool;

    /// Low-priority task of Prologue/Epilogue model.
    ///
    /// The `epilogue` implements the second half of the interrupt handling process which take care
    /// of all deferrable task. Thus, locking/blocking/waiting/... is allowed! While `prologue`
    /// must be implemented by every [`TrapHandler`], the `epilogue` is optional.
    fn epilogue(&self, token: LevelEpilogue) {
        /* Nothing to do here */
    }

    /// Callback to enqueue an `epilogue`.
    ///
    /// If an `prologue`, which interrupted another running `epilogue`, requests the corresponding
    /// `epilogue` is deferred and executed later on.
    ///
    /// The default implementation is best-suited for most occasions. Please do only overwrite this
    /// implementation if you are absolution sure what are you doing.
    fn enqueue(&self) {
        todo!("Provide default implementation using Driver::cause()");
    }

    /// Callback to dequeue an `epilogue`.
    ///
    /// If an `prologue`, which interrupted another running `epilogue`, requests the corresponding
    /// `epilogue` is deferred and executed later on. The `dequeue` marks the first phase of
    /// execution.
    ///
    /// The default implementation is best-suited for most occasions. Please do only overwrite this
    /// implementation if you are absolution sure what are you doing.
    fn dequeue(&self) {
        todo!("Provide default implementation using Driver::cause()");
    }

    /// Check if handler is already enqueued.
    ///
    /// The default implementation is best-suited for most occasions. Please do only overwrite this
    /// implementation if you are absolution sure what are you doing.
    fn is_enqueue(&self) -> bool {
        todo!("Provide default implementation using Driver::cause()");
    }
}

/// Load address of `__trap_entry` into `stvect` regsiter.
///
/// # Caution
/// This operation must be executed on every hart!
pub fn load_trap_vector(token: LevelInitialization) -> LevelInitialization {
    /* Set stvec register */
    let mut stvec = STVec::new();
    stvec.set_mode(STVecMode::Direct);
    let base: u64 = unsafe { mem::transmute(__trap_entry as unsafe extern "C" fn()) };
    assert!(base % 4 == 0);
    stvec.set_base(base >> 2);
    stvec.write();
    token
}
