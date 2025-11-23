//! Panic handler for unexpected interupts.

use crate::drivers::driver::Driver;

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
