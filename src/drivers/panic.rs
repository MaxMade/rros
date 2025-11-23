//! Panic handler for unexpected interupts.

use crate::drivers::driver::Driver;
use crate::sync::level::{LevelEpilogue, LevelPrologue};
use crate::trap::cause::Trap;
use crate::trap::handler_interface::TrapContext;
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

    fn prologue(&self, token: LevelPrologue) -> (bool, LevelPrologue) {
        let _ = token;
        panic!("PANIC! Unexpected interrupt");
    }

    fn epilogue(&self, state: &mut TrapContext, token: LevelEpilogue) {
        let _ = state;
        let _ = token;
        panic!("The panic driver must never request a epilogue");
    }
}
