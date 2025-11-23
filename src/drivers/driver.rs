//! Generic driver interface.

use crate::sync::level::LevelInitialization;

/// Driver interface
pub trait Driver {
    /// Initialize underlying driver
    fn initialize(token: LevelInitialization) -> LevelInitialization;
}
