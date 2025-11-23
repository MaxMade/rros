//! Practical apprach for deadlock prevention: Use lock hierarchies!
//!
//! ```ascii
//! ┌──────────────────────┐
//! │ LevelEpilogue        │
//! └──────────────────────┘
//! enter │ ▲
//!       ▼ │ leave
//! ┌──────────────────────┐
//! │ LevelDriver          │
//! └──────────────────────┘
//! enter │ ▲
//!       ▼ │ leave
//! ┌──────────────────────┐
//! │ LevelScheduler       │
//! └──────────────────────┘
//! enter │ ▲
//!       ▼ │ leave
//! ┌──────────────────────┐
//! │ LevelMemory          │
//! └──────────────────────┘
//! enter │ ▲
//!       ▼ │ leave
//! ┌──────────────────────┐
//! │ LevelPrologue        │
//! └──────────────────────┘
//! enter │ ▲
//!       ▼ │ leave
//! ┌──────────────────────┐
//! │ LevelLockedPrologue  │
//! └──────────────────────┘
//!
//! ┌────────────────┐
//! │ Initialization │
//! └────────────────┘
//! ```

use core::marker::PhantomData;

/// Trait to abstract a level within the hierarchy.
pub trait Level
where
    Self: Sized,
{
    /// Type of upper [`Level`] within the hierarchy.
    type HigherLevel: Level;

    /// Type of upper [`Level`] within the hierarchy.
    type LowerLevel: Level;

    /// Create a new `Level` token.
    unsafe fn create() -> Self;

    /// Get an integer-based representation of the level.
    fn level() -> usize;

    /// Change from `HigherLevel` to `LowerLevel` while consuming `HigherLevel`.
    unsafe fn enter(self) -> Self::LowerLevel {
        assert!(Self::level() > Self::LowerLevel::level());
        Self::LowerLevel::create()
    }

    /// Change back from `LowerLevel` to `HigherLevel` while consuming `LowerLevel`.
    unsafe fn leave(self) -> Self::HigherLevel {
        assert!(Self::level() < Self::HigherLevel::level());
        Self::HigherLevel::create()
    }
}

/// Level Initialization
pub struct LevelInitialization {
    phantom: PhantomData<Self>,
}

impl Level for LevelInitialization {
    type HigherLevel = LevelInvalid;

    type LowerLevel = LevelInvalid;

    unsafe fn create() -> Self {
        Self {
            phantom: PhantomData,
        }
    }

    fn level() -> usize {
        panic!();
    }
}

/// Level Epilogue
pub struct LevelEpilogue {
    phantom: PhantomData<Self>,
}

impl Level for LevelEpilogue {
    type HigherLevel = LevelInvalid;

    type LowerLevel = LevelDriver;

    unsafe fn create() -> Self {
        Self {
            phantom: PhantomData,
        }
    }

    fn level() -> usize {
        5
    }
}

/// Level Driver
pub struct LevelDriver {
    phantom: PhantomData<Self>,
}

impl Level for LevelDriver {
    type HigherLevel = LevelEpilogue;

    type LowerLevel = LevelScheduler;

    unsafe fn create() -> Self {
        Self {
            phantom: PhantomData,
        }
    }

    fn level() -> usize {
        4
    }
}

/// Level Scheduler
pub struct LevelScheduler {
    phantom: PhantomData<Self>,
}

impl Level for LevelScheduler {
    type HigherLevel = LevelDriver;

    type LowerLevel = LevelMemory;

    unsafe fn create() -> Self {
        Self {
            phantom: PhantomData,
        }
    }

    fn level() -> usize {
        3
    }
}

/// Level Memory
pub struct LevelMemory {
    phantom: PhantomData<Self>,
}

impl Level for LevelMemory {
    type HigherLevel = LevelScheduler;

    type LowerLevel = LevelPrologue;

    unsafe fn create() -> Self {
        Self {
            phantom: PhantomData,
        }
    }

    fn level() -> usize {
        2
    }
}

/// Level Progloue
pub struct LevelPrologue {
    phantom: PhantomData<Self>,
}

impl Level for LevelPrologue {
    type HigherLevel = LevelMemory;

    type LowerLevel = LevelLockedPrologue;

    unsafe fn create() -> Self {
        Self {
            phantom: PhantomData,
        }
    }

    fn level() -> usize {
        1
    }
}

/// Level *Locked* Progloue (used by [`IRQTicketLocks`](crate::sync::ticketlock::IRQTicketlock))
pub struct LevelLockedPrologue {
    phantom: PhantomData<Self>,
}

impl Level for LevelLockedPrologue {
    type HigherLevel = LevelPrologue;

    type LowerLevel = LevelInvalid;

    unsafe fn create() -> Self {
        Self {
            phantom: PhantomData,
        }
    }

    fn level() -> usize {
        0
    }
}

/// Invalid level to indicate "end of hierarchy"
pub struct LevelInvalid {
    phantom: PhantomData<Self>,
}

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

/// Trait to allow to "skip" layers using convinient adapter.
pub trait Adapter<HigherLevel, LowerLevel, Guard>
where
    Self: Sized,
    HigherLevel: Level,
    LowerLevel: Level,
    Guard: AdapterGuard<HigherLevel, LowerLevel>,
{
    /// Create a new [`Adapter`].
    fn new() -> Self;

    /// Change from `HigherLevel` to `LowerLevel` while consuming `HigherLevel`.
    unsafe fn enter(self, level: HigherLevel) -> Guard {
        // Consule level
        let _ = level;

        // Sanity check of HigherLevel and LowerLevel
        assert!(HigherLevel::level() > LowerLevel::level());

        // Create guard
        Guard::new()
    }
}

/// Trait to return form `Adapter::enter`.
pub trait AdapterGuard<HigherLevel, LowerLevel>
where
    Self: Sized,
    HigherLevel: Level,
    LowerLevel: Level,
{
    /// Create a new [`AdapterGuard`].
    unsafe fn new() -> Self;

    /// Change back from `LowerLevel` to `HigherLevel` while consuming `LowerLevel`.
    unsafe fn leave(self, level: LowerLevel) -> HigherLevel {
        // Consule level
        let _ = level;

        // Sanity check of HigherLevel and LowerLevel
        assert!(HigherLevel::level() > LowerLevel::level());

        // Produce level
        HigherLevel::create()
    }
}

/// [`Adapter`] for [`LevelEpilogue`] to [`LevelScheduler`]
pub struct AdapterEpilogueScheduler {
    phantom: PhantomData<Self>,
}

/// [`AdapterGuard`] for [`LevelEpilogue`] to [`LevelScheduler`]
pub struct AdapterGuardEpilogueScheduler {
    phantom: PhantomData<Self>,
}

impl Adapter<LevelEpilogue, LevelScheduler, AdapterGuardEpilogueScheduler>
    for AdapterEpilogueScheduler
{
    fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl AdapterGuard<LevelEpilogue, LevelScheduler> for AdapterGuardEpilogueScheduler {
    unsafe fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

/// [`Adapter`] for [`LevelEpilogue`] to [`LevelMemory`]
pub struct AdapterEpilogueMemory {
    phantom: PhantomData<Self>,
}

/// [`AdapterGuard`] for [`LevelEpilogue`] to [`LevelMemory`]
pub struct AdapterGuardEpilogueMemory {
    phantom: PhantomData<Self>,
}

impl Adapter<LevelEpilogue, LevelMemory, AdapterGuardEpilogueMemory> for AdapterEpilogueMemory {
    fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl AdapterGuard<LevelEpilogue, LevelMemory> for AdapterGuardEpilogueMemory {
    unsafe fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

/// [`Adapter`] for [`LevelEpilogue`] to [`LevelPrologue`]
pub struct AdapterEpiloguePrologue {
    phantom: PhantomData<Self>,
}

/// [`AdapterGuard`] for [`LevelEpilogue`] to [`LevelPrologue`]
pub struct AdapterGuardEpiloguePrologue {
    phantom: PhantomData<Self>,
}

impl Adapter<LevelEpilogue, LevelPrologue, AdapterGuardEpiloguePrologue>
    for AdapterEpiloguePrologue
{
    fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl AdapterGuard<LevelEpilogue, LevelPrologue> for AdapterGuardEpiloguePrologue {
    unsafe fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

/// [`Adapter`] for [`LevelDriver`] to [`LevelMemory`]
pub struct AdapterDriverMemory {
    phantom: PhantomData<Self>,
}

/// [`AdapterGuard`] for [`LevelDriver`] to [`LevelMemory`]
pub struct AdapterGuardDriverMemory {
    phantom: PhantomData<Self>,
}

impl Adapter<LevelDriver, LevelMemory, AdapterGuardDriverMemory> for AdapterDriverMemory {
    fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl AdapterGuard<LevelDriver, LevelMemory> for AdapterGuardDriverMemory {
    unsafe fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

/// [`Adapter`] for [`LevelDriver`] to [`LevelPrologue`]
pub struct AdapterDriverPrologue {
    phantom: PhantomData<Self>,
}

/// [`AdapterGuard`] for [`LevelDriver`] to [`LevelPrologue`]
pub struct AdapterGuardDriverPrologue {
    phantom: PhantomData<Self>,
}

impl Adapter<LevelDriver, LevelPrologue, AdapterGuardDriverPrologue> for AdapterDriverPrologue {
    fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl AdapterGuard<LevelDriver, LevelPrologue> for AdapterGuardDriverPrologue {
    unsafe fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

/// [`Adapter`] for [`LevelScheduler`] to [`LevelPrologue`]
pub struct AdapterSchedulerPrologue {
    phantom: PhantomData<Self>,
}

/// [`AdapterGuard`] for [`LevelScheduler`] to [`LevelPrologue`]
pub struct AdapterGuardSchedulerPrologue {
    phantom: PhantomData<Self>,
}

impl Adapter<LevelScheduler, LevelPrologue, AdapterGuardSchedulerPrologue>
    for AdapterSchedulerPrologue
{
    fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl AdapterGuard<LevelScheduler, LevelPrologue> for AdapterGuardSchedulerPrologue {
    unsafe fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}
