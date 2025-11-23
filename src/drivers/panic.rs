//! Panic handler for unexpected interupts.

use crate::drivers::driver::Driver;
use crate::sync::level::{LevelEpilogue, LevelPrologue};
use crate::trap::cause::Trap;
use crate::trap::handlers::TrapHandler;

/// Panic handler for unexpected interupts.
pub struct Panic {}

/// Global Panic object.
pub static PANIC: Panic = Panic {};

impl Driver for Panic {
    fn initiailize(
        token: crate::sync::level::LevelInitialization,
    ) -> Result<
        crate::sync::level::LevelInitialization,
        (
            super::driver::DriverError,
            crate::sync::level::LevelInitialization,
        ),
    > {
        Ok(token)
    }
}

impl TrapHandler for Panic {
    fn cause() -> Trap {
        panic!("The panic driver must never be Driver::cause()");
    }

    fn prologue(&self, _token: LevelPrologue) -> bool {
        panic!("PANIC! Unexpected interrupt!");
    }

    fn epilogue(&self, _token: LevelEpilogue) {
        panic!("The panic driver must never request a epilogue");
    }

    fn enqueue(&self) {
        panic!("The panic driver must never be Driver::enqueue()");
    }

    fn dequeue(&self) {
        panic!("The panic driver must never be Driver::dequeue()");
    }

    fn is_enqueue(&self) -> bool {
        false
    }
}
