//! Software-Abstractions for trap handlers.

use crate::drivers::panic::PANIC;
use crate::sync::init_cell::InitCell;
use crate::sync::level::LevelEpilogue;
use crate::sync::level::LevelInitialization;
use crate::sync::level::LevelLockedPrologue;
use crate::sync::level::LevelPrologue;
use crate::sync::per_core::PerCore;
use crate::trap::cause::Exception;
use crate::trap::cause::Interrupt;
use crate::trap::cause::Trap;
use crate::trap::handler_interface::TrapContext;

const NUM_EXCEPTION_HANDLERS: usize = 256;
const NUM_INTERRUPT_HANDLERS: usize = 256;

/// Instance for registering/requesting [`TrapHandler`]s.
pub static TRAP_HANDLERS: InitCell<TrapHandlers> = InitCell::new();

/// Convientent wrapper for dealing with shared references to handlers.
pub type HandlerRef = &'static dyn TrapHandler;

/// Abstraction of trap handlers
pub struct TrapHandlers {
    /// Register [`TrapHandlers`] for [`Trap::Interrupt`].
    pub(in crate::trap::handlers) exception_handlers: [HandlerRef; NUM_EXCEPTION_HANDLERS],
    /// Register [`TrapHandlers`] handlers for [`Trap::Exception`]
    pub(in crate::trap::handlers) interrupt_handlers: [HandlerRef; NUM_INTERRUPT_HANDLERS],
    /// Pending [`Trap::Interrupt`]s.
    pub(in crate::trap::handlers) pending_interrupts:
        PerCore<[bool; NUM_INTERRUPT_HANDLERS], LevelPrologue, LevelLockedPrologue>,
    /// Pending [`Trap::Exception`]s.
    pub(in crate::trap::handlers) pending_exceptions:
        PerCore<[bool; NUM_EXCEPTION_HANDLERS], LevelPrologue, LevelLockedPrologue>,
}

impl TrapHandlers {
    /// Prepare [`TRAP_HANDLERS`].
    pub fn initialize(token: LevelInitialization) -> LevelInitialization {
        // Get mutable reference for TRAP_HANDLERS
        let mut handlers = TRAP_HANDLERS.get_mut(token);

        // Initialize members
        let panic: HandlerRef = &PANIC;
        handlers.exception_handlers = [panic; NUM_EXCEPTION_HANDLERS];
        handlers.interrupt_handlers = [panic; NUM_INTERRUPT_HANDLERS];

        handlers.pending_interrupts = PerCore::new_copy([false; NUM_INTERRUPT_HANDLERS]);
        handlers.pending_exceptions = PerCore::new_copy([false; NUM_EXCEPTION_HANDLERS]);

        handlers.destroy()
    }

    /// Register `handler` for `trap`
    ///
    /// # Panic
    /// If another `handler` is already register for `trap`, this function will panic!
    pub fn register(
        trap: Trap,
        handler: HandlerRef,
        token: LevelInitialization,
    ) -> LevelInitialization {
        let mut handlers = TRAP_HANDLERS.get_mut(token);

        let panic: HandlerRef = &PANIC;
        match trap {
            Trap::Interrupt(interrupt) => {
                let index: usize = interrupt.into();
                if handlers.interrupt_handlers[index] as *const _ != panic as *const _ {
                    panic!(
                        "Unable to overwrite handler for {} at trap handlers interface",
                        interrupt
                    );
                }
                handlers.interrupt_handlers[index] = handler;
            }
            Trap::Exception(exception) => {
                let index: usize = exception.into();
                if handlers.exception_handlers[index] as *const _ != panic as *const _ {
                    panic!(
                        "Unable to overwrite handler for {} at trap handlers interface",
                        exception
                    );
                }
                handlers.exception_handlers[index] = handler;
            }
        }

        handlers.destroy()
    }

    /// Finish initialization of [`TRAP_HANDLERS`] after all drivers registered their corresponding
    /// handlers.
    pub fn finalize(token: LevelInitialization) -> LevelInitialization {
        let token = unsafe { TRAP_HANDLERS.finanlize(token) };
        token
    }

    /// Get corresponding [`HandlerRef`] for [`Trap`].
    pub fn get(trap: Trap, token: LevelPrologue) -> (HandlerRef, LevelPrologue) {
        let handler = match trap {
            Trap::Interrupt(interrupt) => {
                let index: usize = interrupt.into();
                TRAP_HANDLERS.as_ref().interrupt_handlers[index]
            }
            Trap::Exception(exception) => {
                let index: usize = exception.into();
                TRAP_HANDLERS.as_ref().exception_handlers[index]
            }
        };

        (handler, token)
    }

    /// Enqueue a pending [`Trap`].
    ///
    /// If a [`Trap`] interrupts an other currently running `epilogue` with its own corresponding
    /// `prologue`, the corresponding [`Trap`] is enqueue and executed later on.
    pub fn enqueue(trap: Trap, token: LevelPrologue) -> LevelPrologue {
        let token = match trap {
            Trap::Interrupt(interrupt) => {
                let index: usize = interrupt.into();
                let (mut pending_interrupt, token) =
                    TRAP_HANDLERS.as_ref().pending_interrupts.get_mut(token);
                pending_interrupt[index] = true;
                pending_interrupt.destroy(token)
            }
            Trap::Exception(exception) => {
                let index: usize = exception.into();
                let (mut pending_exception, token) =
                    TRAP_HANDLERS.as_ref().pending_exceptions.get_mut(token);
                pending_exception[index] = true;
                pending_exception.destroy(token)
            }
        };

        token
    }

    /// Dequeue a pending [`Trap`].
    ///
    /// If a [`Trap`] interrupts an other currently running `epilogue` with its own corresponding
    /// `prologue`, the corresponding [`Trap`] is enqueue and dequeued later on.
    pub fn dequeue(token: LevelPrologue) -> (Option<Trap>, LevelPrologue) {
        let mut trap = None;

        // Check for pending interrupt
        let (mut pending_interrupts, token) =
            TRAP_HANDLERS.as_ref().pending_interrupts.get_mut(token);
        for (i, pending) in pending_interrupts.iter().enumerate() {
            if *pending {
                let interrupt = Interrupt::from(i);
                trap = Some(Trap::Interrupt(interrupt));
                break;
            }
        }
        if let Some(Trap::Interrupt(interrupt)) = trap {
            // Mark interrupt as processed
            let index: usize = interrupt.into();
            pending_interrupts[index] = false;

            // Return pending interrupt
            return (trap, pending_interrupts.destroy(token));
        }
        let token = pending_interrupts.destroy(token);

        // Check for pending exception
        let (mut pending_exceptions, token) =
            TRAP_HANDLERS.as_ref().pending_exceptions.get_mut(token);
        for (i, pending) in pending_exceptions.iter().enumerate() {
            if *pending {
                let exception = Exception::from(i);
                trap = Some(Trap::Exception(exception));
                break;
            }
        }
        if let Some(Trap::Exception(exception)) = trap {
            // Mark exception as processed
            let index: usize = exception.into();
            pending_exceptions[index] = false;

            // Return pending exception
            return (trap, pending_exceptions.destroy(token));
        }
        let token = pending_exceptions.destroy(token);

        (None, token)
    }
}

extern "C" {
    fn __trap_entry();
}

/// Interface for handling traps -  suitable for interrupts and exceptions.
pub trait TrapHandler: Sync {
    /// Get [`Trap`] cause.
    fn cause() -> Trap
    where
        Self: Sized;

    /// High-priority task of Prologue/Epilogue model.
    ///
    /// Every interrupt capable driver has to implement the `prologue` which takes care of any task
    /// why must be executed immediately upon receiving the corresponding interrupt. This
    /// enables low-latency interrupt handling, but in turn implies the strict requirements for the
    /// handler: It *must* be as short as possible as interrupts are disabled during execution.
    /// Thus, no locking/blocking/waiting/... is allowed! For such tasks, an optional `epilogue`
    /// can be requested by return `true`.
    fn prologue(&self, token: LevelPrologue) -> (bool, LevelPrologue);

    /// Low-priority task of Prologue/Epilogue model.
    ///
    /// The `epilogue` implements the second half of the interrupt handling process which take care
    /// of all deferrable task. Thus, locking/blocking/waiting/... is allowed! While `prologue`
    /// must be implemented by every [`TrapHandler`], the `epilogue` is optional.
    fn epilogue(&self, state: Option<&mut TrapContext>, token: LevelEpilogue) -> LevelEpilogue {
        // Ignore state
        let _ = state;

        // Nothing to do here
        token
    }
}
