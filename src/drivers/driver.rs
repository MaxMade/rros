//! Generic driver interface.

use core::error::Error;
use core::fmt::Display;

use crate::sync::level::LevelInitialization;

/// Driver interface
pub trait Driver {
    /// Initialize underlying driver
    fn initiailize(
        token: LevelInitialization,
    ) -> Result<LevelInitialization, (DriverError, LevelInitialization)>;
}

/// Generic driver errors.
#[derive(Debug)]
pub enum DriverError {
    /// Result of attempting to initialize driver with non-comptible device node.
    NonCompatibleDevice,
    /// Failed attempt to request data from device.
    NoDataAvailable,
}

impl Display for DriverError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            DriverError::NonCompatibleDevice => write!(f, "Non-comptible device node"),
            DriverError::NoDataAvailable => write!(f, "No data available"),
        }
    }
}

impl Error for DriverError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.source()
    }
}
