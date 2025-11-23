//! [`Error`]s associated with memory management.

use core::error::Error;
use core::fmt::Display;

/// [`Error`]s associated with memory management.
#[derive(Debug)]
pub enum MemoryError {
    /// Out-of-Memory.
    OutOfMemory,
}

impl Display for MemoryError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            MemoryError::OutOfMemory => write!(f, "Out of Memory"),
        }
    }
}

impl Error for MemoryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }

    fn description(&self) -> &str {
        "description() is deprecated; use Display"
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.source()
    }
}
