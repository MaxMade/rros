//! Software-Abstractions for trap handlers.

use core::array;
use core::mem;
use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering;

use crate::drivers::panic::PANIC;
use crate::kernel::cpu::STVec;
use crate::kernel::cpu::STVecMode;
use crate::sync::init_cell::InitCell;
use crate::sync::level::LevelEpilogue;
use crate::sync::level::LevelInitialization;
use crate::sync::level::LevelPrologue;
use crate::sync::per_core::PerCore;
use crate::trap::cause::Trap;

const NUM_EXCEPTION_HANDLERS: usize = 256;
const NUM_INTERRUPT_HANDLERS: usize = 256;

/// Instance for registering/requesting [`TrapHandler`]s.
pub static TRAP_HANDLERS: InitCell<TrapHandlers> = InitCell::new();

/// Abstraction of trap handlers
pub struct TrapHandlers {
    exception_handlers: [&'static dyn TrapHandler; NUM_EXCEPTION_HANDLERS],
    interrupt_handlers: [&'static dyn TrapHandler; NUM_INTERRUPT_HANDLERS],
    globally_pending: AtomicBool,
    locally_pending_interrupt: PerCore<[AtomicBool; NUM_INTERRUPT_HANDLERS]>,
    locally_pending_exception: PerCore<[AtomicBool; NUM_EXCEPTION_HANDLERS]>,
}

impl TrapHandlers {
    /// Prepare [`TRAP_HANDLERS`].
    pub fn initailize(token: LevelInitialization) -> LevelInitialization {
        // Get mutable reference for TRAP_HANDLERS
        let (handlers, token) = TRAP_HANDLERS.as_mut(token);

        // Initialize members
        let panic: &'static dyn TrapHandler = &PANIC;
        handlers.exception_handlers = [panic; NUM_EXCEPTION_HANDLERS];
        handlers.interrupt_handlers = [panic; NUM_INTERRUPT_HANDLERS];

        handlers.globally_pending.store(false, Ordering::Relaxed);

        handlers.locally_pending_interrupt =
            PerCore::new_fn(|_| array::from_fn(|_| AtomicBool::new(false)));
        handlers.locally_pending_exception =
            PerCore::new_fn(|_| array::from_fn(|_| AtomicBool::new(false)));

        token
    }

    /// Register `handler` for `trap`
    ///
    /// # Panic
    /// If another `handler` is already register for `trap`, this function will panic!
    pub fn register(
        &self,
        trap: Trap,
        handler: &'static dyn TrapHandler,
        token: LevelInitialization,
    ) -> LevelInitialization {
        // Get mutable reference for TRAP_HANDLERS
        let (handlers, token) = TRAP_HANDLERS.as_mut(token);

        let panic: &'static dyn TrapHandler = &PANIC;
        match trap {
            Trap::Interrupt(interrupt) => {
                let index: usize = interrupt.into();
                if handlers.interrupt_handlers[index] as *const _ == panic as *const _ {
                    panic!(
                        "Unable to overwrite handler for {} at trap handlers interface",
                        interrupt
                    );
                }
                handlers.interrupt_handlers[index] = handler;
            }
            Trap::Exception(exception) => {
                let index: usize = exception.into();
                if handlers.exception_handlers[index] as *const _ == panic as *const _ {
                    panic!(
                        "Unable to overwrite handler for {} at trap handlers interface",
                        exception
                    );
                }
                handlers.exception_handlers[index] = handler;
            }
        }

        token
    }

    /// Finish initialization of [`TRAP_HANDLERS`] after all drivers registered their corresponding
    /// handlers.
    pub fn finalize(token: LevelInitialization) -> LevelInitialization {
        let token = unsafe { TRAP_HANDLERS.finanlize(token) };
        token
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
