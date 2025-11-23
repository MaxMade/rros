//! Practical apprach for deadlock prevention: Use lock hierarchies!
//!
//! ```ascii
//! ┌────────────────┐
//! │ LevelEpilogue  │
//! └────────────────┘
//! enter │ ▲
//!       ▼ │ leave
//! ┌────────────────┐
//! │  LevelDriver   │
//! └────────────────┘
//! enter │ ▲
//!       ▼ │ leave
//! ┌────────────────┐
//! │ LevelScheduler │
//! └────────────────┘
//! enter │ ▲
//!       ▼ │ leave
//! ┌────────────────┐
//! │  LevelMemory   │
//! └────────────────┘
//! enter │ ▲
//!       ▼ │ leave
//! ┌────────────────┐
//! │ LevelPrologue  │
//! └────────────────┘
//! ```

pub trait Level
where
    Self: Sized,
{
    type HigherLevel: Level;
    type LowerLevel: Level;

    /// Create a new `Level` token.
    unsafe fn create() -> Self;

    fn level() -> usize;

    fn enter(self) -> Self::LowerLevel {
        assert!(Self::level() > Self::LowerLevel::level());
        unsafe { Self::LowerLevel::create() }
    }

    fn leave(self) -> Self::HigherLevel {
        assert!(Self::level() < Self::HigherLevel::level());
        unsafe { Self::HigherLevel::create() }
    }
}

pub struct LevelEpilogue {}

impl Level for LevelEpilogue {
    type HigherLevel = LevelInvalid;

    type LowerLevel = LevelDriver;

    unsafe fn create() -> Self {
        Self {}
    }

    fn level() -> usize {
        4
    }
}

pub struct LevelDriver {}

impl Level for LevelDriver {
    type HigherLevel = LevelEpilogue;

    type LowerLevel = LevelScheduler;

    unsafe fn create() -> Self {
        Self {}
    }

    fn level() -> usize {
        3
    }
}

pub struct LevelScheduler {}

impl Level for LevelScheduler {
    type HigherLevel = LevelDriver;

    type LowerLevel = LevelMemory;

    unsafe fn create() -> Self {
        Self {}
    }

    fn level() -> usize {
        2
    }
}

pub struct LevelMemory {}

impl Level for LevelMemory {
    type HigherLevel = LevelScheduler;

    type LowerLevel = LevelPrologue;

    unsafe fn create() -> Self {
        Self {}
    }

    fn level() -> usize {
        1
    }
}

pub struct LevelPrologue {}

impl Level for LevelPrologue {
    type HigherLevel = LevelMemory;

    type LowerLevel = LevelInvalid;

    unsafe fn create() -> Self {
        Self {}
    }

    fn level() -> usize {
        0
    }
}

pub struct LevelInvalid {}

impl Level for LevelInvalid {
    type HigherLevel = LevelInvalid;

    type LowerLevel = LevelInvalid;

    unsafe fn create() -> Self {
        panic!();
    }

    fn level() -> usize {
        panic!()
    }
}
