//! Software-Abstractions for trap handling.

use crate::kernel::trap::Trap;
use crate::sync::level::LevelEpilogue;
use crate::sync::level::LevelPrologue;

/// Interface for handling traps -  suitable for interrupts and exceptions
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
