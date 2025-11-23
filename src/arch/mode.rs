//! Abstraction of execution mode.

use core::fmt::Display;

/// Abstraction of execution mode.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ExecutionMode {
    /// User mode.
    User,
    /// Supervisor mode.
    Supervisor,
    /// Machine mode.
    Machine,
}

impl Display for ExecutionMode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ExecutionMode::User => write!(f, "User"),
            ExecutionMode::Supervisor => write!(f, "Supervisor"),
            ExecutionMode::Machine => write!(f, "Machine"),
        }
    }
}
