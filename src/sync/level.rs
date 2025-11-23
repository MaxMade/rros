//! Practical apprach for deadlock prevention: Use lock hierarchies!
//! ```ascii
//! ┌─────────────────────┐
//! │ LevelEpilogue       │
//! └─────────────────────┘
//! enter │ ▲
//!       ▼ │ leave
//! ┌─────────────────────┐
//! │ LevelDriver         │
//! └─────────────────────┘
//! enter │ ▲
//!       ▼ │ leave
//! ┌─────────────────────┐
//! │ LevelScheduler      │
//! └─────────────────────┘
//! enter │ ▲
//!       ▼ │ leave
//! ┌─────────────────────┐
//! │ LevelMemory         │
//! └─────────────────────┘
//! enter │ ▲
//!       ▼ │ leave
//! ┌─────────────────────┐
//! │ LevelMapping        │
//! └─────────────────────┘
//! enter │ ▲
//!       ▼ │ leave
//! ┌─────────────────────┐
//! │ LevelPaging         │
//! └─────────────────────┘
//! enter │ ▲
//!       ▼ │ leave
//! ┌─────────────────────┐
//! │ LevelPrologue       │
//! └─────────────────────┘
//! enter │ ▲
//!       ▼ │ leave
//! ┌─────────────────────┐
//! │ LevelLockedPrologue │
//! └─────────────────────┘
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

/// Default execution [`Level`] for `epilogue`s (for more details, see [`TrapHandler`](crate::trap::handlers::TrapHandler))
pub struct LevelEpilogue {
    phantom: PhantomData<Self>,
}

impl Level for LevelEpilogue {
    type HigherLevel = LevelDriver;

    type LowerLevel = LevelEpilogue;

    unsafe fn create() -> Self {
        Self {
            phantom: PhantomData,
        }
    }

    fn level() -> usize {
        7
    }
}

/// [`Adapter`] for [`LevelEpilogue`] to [`LevelDriver`]
pub struct AdapterEpilogueDriver {
    phantom: PhantomData<Self>,
}

/// [`AdapterGuard`] for [`LevelEpilogue`] to [`LevelDriver`]
pub struct AdapterGuardEpilogueDriver {
    phantom: PhantomData<Self>,
}

impl Adapter<LevelEpilogue, LevelDriver, AdapterGuardEpilogueDriver> for AdapterEpilogueDriver {
    fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl AdapterGuard<LevelEpilogue, LevelDriver> for AdapterGuardEpilogueDriver {
    unsafe fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
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

/// [`Adapter`] for [`LevelEpilogue`] to [`LevelMapping`]
pub struct AdapterEpilogueMapping {
    phantom: PhantomData<Self>,
}

/// [`AdapterGuard`] for [`LevelEpilogue`] to [`LevelMapping`]
pub struct AdapterGuardEpilogueMapping {
    phantom: PhantomData<Self>,
}

impl Adapter<LevelEpilogue, LevelMapping, AdapterGuardEpilogueMapping> for AdapterEpilogueMapping {
    fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl AdapterGuard<LevelEpilogue, LevelMapping> for AdapterGuardEpilogueMapping {
    unsafe fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

/// [`Adapter`] for [`LevelEpilogue`] to [`LevelPaging`]
pub struct AdapterEpiloguePaging {
    phantom: PhantomData<Self>,
}

/// [`AdapterGuard`] for [`LevelEpilogue`] to [`LevelPaging`]
pub struct AdapterGuardEpiloguePaging {
    phantom: PhantomData<Self>,
}

impl Adapter<LevelEpilogue, LevelPaging, AdapterGuardEpiloguePaging> for AdapterEpiloguePaging {
    fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl AdapterGuard<LevelEpilogue, LevelPaging> for AdapterGuardEpiloguePaging {
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

/// [`Adapter`] for [`LevelEpilogue`] to [`LevelLockedPrologue`]
pub struct AdapterEpilogueLockedPrologue {
    phantom: PhantomData<Self>,
}

/// [`AdapterGuard`] for [`LevelEpilogue`] to [`LevelLockedPrologue`]
pub struct AdapterGuardEpilogueLockedPrologue {
    phantom: PhantomData<Self>,
}

impl Adapter<LevelEpilogue, LevelLockedPrologue, AdapterGuardEpilogueLockedPrologue>
    for AdapterEpilogueLockedPrologue
{
    fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl AdapterGuard<LevelEpilogue, LevelLockedPrologue> for AdapterGuardEpilogueLockedPrologue {
    unsafe fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

/// Default [`Level`] for device driver locking
pub struct LevelDriver {
    phantom: PhantomData<Self>,
}

impl Level for LevelDriver {
    type HigherLevel = LevelScheduler;

    type LowerLevel = LevelEpilogue;

    unsafe fn create() -> Self {
        Self {
            phantom: PhantomData,
        }
    }

    fn level() -> usize {
        6
    }
}

/// [`Adapter`] for [`LevelDriver`] to [`LevelScheduler`]
pub struct AdapterDriverScheduler {
    phantom: PhantomData<Self>,
}

/// [`AdapterGuard`] for [`LevelDriver`] to [`LevelScheduler`]
pub struct AdapterGuardDriverScheduler {
    phantom: PhantomData<Self>,
}

impl Adapter<LevelDriver, LevelScheduler, AdapterGuardDriverScheduler> for AdapterDriverScheduler {
    fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl AdapterGuard<LevelDriver, LevelScheduler> for AdapterGuardDriverScheduler {
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

/// [`Adapter`] for [`LevelDriver`] to [`LevelMapping`]
pub struct AdapterDriverMapping {
    phantom: PhantomData<Self>,
}

/// [`AdapterGuard`] for [`LevelDriver`] to [`LevelMapping`]
pub struct AdapterGuardDriverMapping {
    phantom: PhantomData<Self>,
}

impl Adapter<LevelDriver, LevelMapping, AdapterGuardDriverMapping> for AdapterDriverMapping {
    fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl AdapterGuard<LevelDriver, LevelMapping> for AdapterGuardDriverMapping {
    unsafe fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

/// [`Adapter`] for [`LevelDriver`] to [`LevelPaging`]
pub struct AdapterDriverPaging {
    phantom: PhantomData<Self>,
}

/// [`AdapterGuard`] for [`LevelDriver`] to [`LevelPaging`]
pub struct AdapterGuardDriverPaging {
    phantom: PhantomData<Self>,
}

impl Adapter<LevelDriver, LevelPaging, AdapterGuardDriverPaging> for AdapterDriverPaging {
    fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl AdapterGuard<LevelDriver, LevelPaging> for AdapterGuardDriverPaging {
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

/// [`Adapter`] for [`LevelDriver`] to [`LevelLockedPrologue`]
pub struct AdapterDriverLockedPrologue {
    phantom: PhantomData<Self>,
}

/// [`AdapterGuard`] for [`LevelDriver`] to [`LevelLockedPrologue`]
pub struct AdapterGuardDriverLockedPrologue {
    phantom: PhantomData<Self>,
}

impl Adapter<LevelDriver, LevelLockedPrologue, AdapterGuardDriverLockedPrologue>
    for AdapterDriverLockedPrologue
{
    fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl AdapterGuard<LevelDriver, LevelLockedPrologue> for AdapterGuardDriverLockedPrologue {
    unsafe fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

/// Required [`Level`] for interacting with the scheduling/task management interface
pub struct LevelScheduler {
    phantom: PhantomData<Self>,
}

impl Level for LevelScheduler {
    type HigherLevel = LevelMemory;

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

/// [`Adapter`] for [`LevelScheduler`] to [`LevelMemory`]
pub struct AdapterSchedulerMemory {
    phantom: PhantomData<Self>,
}

/// [`AdapterGuard`] for [`LevelScheduler`] to [`LevelMemory`]
pub struct AdapterGuardSchedulerMemory {
    phantom: PhantomData<Self>,
}

impl Adapter<LevelScheduler, LevelMemory, AdapterGuardSchedulerMemory> for AdapterSchedulerMemory {
    fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl AdapterGuard<LevelScheduler, LevelMemory> for AdapterGuardSchedulerMemory {
    unsafe fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

/// [`Adapter`] for [`LevelScheduler`] to [`LevelMapping`]
pub struct AdapterSchedulerMapping {
    phantom: PhantomData<Self>,
}

/// [`AdapterGuard`] for [`LevelScheduler`] to [`LevelMapping`]
pub struct AdapterGuardSchedulerMapping {
    phantom: PhantomData<Self>,
}

impl Adapter<LevelScheduler, LevelMapping, AdapterGuardSchedulerMapping>
    for AdapterSchedulerMapping
{
    fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl AdapterGuard<LevelScheduler, LevelMapping> for AdapterGuardSchedulerMapping {
    unsafe fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

/// [`Adapter`] for [`LevelScheduler`] to [`LevelPaging`]
pub struct AdapterSchedulerPaging {
    phantom: PhantomData<Self>,
}

/// [`AdapterGuard`] for [`LevelScheduler`] to [`LevelPaging`]
pub struct AdapterGuardSchedulerPaging {
    phantom: PhantomData<Self>,
}

impl Adapter<LevelScheduler, LevelPaging, AdapterGuardSchedulerPaging> for AdapterSchedulerPaging {
    fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl AdapterGuard<LevelScheduler, LevelPaging> for AdapterGuardSchedulerPaging {
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

/// [`Adapter`] for [`LevelScheduler`] to [`LevelLockedPrologue`]
pub struct AdapterSchedulerLockedPrologue {
    phantom: PhantomData<Self>,
}

/// [`AdapterGuard`] for [`LevelScheduler`] to [`LevelLockedPrologue`]
pub struct AdapterGuardSchedulerLockedPrologue {
    phantom: PhantomData<Self>,
}

impl Adapter<LevelScheduler, LevelLockedPrologue, AdapterGuardSchedulerLockedPrologue>
    for AdapterSchedulerLockedPrologue
{
    fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl AdapterGuard<LevelScheduler, LevelLockedPrologue> for AdapterGuardSchedulerLockedPrologue {
    unsafe fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

/// Required [`Level`] for interacting with (generic) memory mangement interfaces
pub struct LevelMemory {
    phantom: PhantomData<Self>,
}

impl Level for LevelMemory {
    type HigherLevel = LevelMapping;

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

/// [`Adapter`] for [`LevelMemory`] to [`LevelMapping`]
pub struct AdapterMemoryMapping {
    phantom: PhantomData<Self>,
}

/// [`AdapterGuard`] for [`LevelMemory`] to [`LevelMapping`]
pub struct AdapterGuardMemoryMapping {
    phantom: PhantomData<Self>,
}

impl Adapter<LevelMemory, LevelMapping, AdapterGuardMemoryMapping> for AdapterMemoryMapping {
    fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl AdapterGuard<LevelMemory, LevelMapping> for AdapterGuardMemoryMapping {
    unsafe fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

/// [`Adapter`] for [`LevelMemory`] to [`LevelPaging`]
pub struct AdapterMemoryPaging {
    phantom: PhantomData<Self>,
}

/// [`AdapterGuard`] for [`LevelMemory`] to [`LevelPaging`]
pub struct AdapterGuardMemoryPaging {
    phantom: PhantomData<Self>,
}

impl Adapter<LevelMemory, LevelPaging, AdapterGuardMemoryPaging> for AdapterMemoryPaging {
    fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl AdapterGuard<LevelMemory, LevelPaging> for AdapterGuardMemoryPaging {
    unsafe fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

/// [`Adapter`] for [`LevelMemory`] to [`LevelPrologue`]
pub struct AdapterMemoryPrologue {
    phantom: PhantomData<Self>,
}

/// [`AdapterGuard`] for [`LevelMemory`] to [`LevelPrologue`]
pub struct AdapterGuardMemoryPrologue {
    phantom: PhantomData<Self>,
}

impl Adapter<LevelMemory, LevelPrologue, AdapterGuardMemoryPrologue> for AdapterMemoryPrologue {
    fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl AdapterGuard<LevelMemory, LevelPrologue> for AdapterGuardMemoryPrologue {
    unsafe fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

/// [`Adapter`] for [`LevelMemory`] to [`LevelLockedPrologue`]
pub struct AdapterMemoryLockedPrologue {
    phantom: PhantomData<Self>,
}

/// [`AdapterGuard`] for [`LevelMemory`] to [`LevelLockedPrologue`]
pub struct AdapterGuardMemoryLockedPrologue {
    phantom: PhantomData<Self>,
}

impl Adapter<LevelMemory, LevelLockedPrologue, AdapterGuardMemoryLockedPrologue>
    for AdapterMemoryLockedPrologue
{
    fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl AdapterGuard<LevelMemory, LevelLockedPrologue> for AdapterGuardMemoryLockedPrologue {
    unsafe fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

/// Required [`Level`] for interacting with mapping interface
pub struct LevelMapping {
    phantom: PhantomData<Self>,
}

impl Level for LevelMapping {
    type HigherLevel = LevelPaging;

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

/// [`Adapter`] for [`LevelMapping`] to [`LevelPaging`]
pub struct AdapterMappingPaging {
    phantom: PhantomData<Self>,
}

/// [`AdapterGuard`] for [`LevelMapping`] to [`LevelPaging`]
pub struct AdapterGuardMappingPaging {
    phantom: PhantomData<Self>,
}

impl Adapter<LevelMapping, LevelPaging, AdapterGuardMappingPaging> for AdapterMappingPaging {
    fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl AdapterGuard<LevelMapping, LevelPaging> for AdapterGuardMappingPaging {
    unsafe fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

/// [`Adapter`] for [`LevelMapping`] to [`LevelPrologue`]
pub struct AdapterMappingPrologue {
    phantom: PhantomData<Self>,
}

/// [`AdapterGuard`] for [`LevelMapping`] to [`LevelPrologue`]
pub struct AdapterGuardMappingPrologue {
    phantom: PhantomData<Self>,
}

impl Adapter<LevelMapping, LevelPrologue, AdapterGuardMappingPrologue> for AdapterMappingPrologue {
    fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl AdapterGuard<LevelMapping, LevelPrologue> for AdapterGuardMappingPrologue {
    unsafe fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

/// [`Adapter`] for [`LevelMapping`] to [`LevelLockedPrologue`]
pub struct AdapterMappingLockedPrologue {
    phantom: PhantomData<Self>,
}

/// [`AdapterGuard`] for [`LevelMapping`] to [`LevelLockedPrologue`]
pub struct AdapterGuardMappingLockedPrologue {
    phantom: PhantomData<Self>,
}

impl Adapter<LevelMapping, LevelLockedPrologue, AdapterGuardMappingLockedPrologue>
    for AdapterMappingLockedPrologue
{
    fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl AdapterGuard<LevelMapping, LevelLockedPrologue> for AdapterGuardMappingLockedPrologue {
    unsafe fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

/// Required [`Level`] for interacting with page allocator
pub struct LevelPaging {
    phantom: PhantomData<Self>,
}

impl Level for LevelPaging {
    type HigherLevel = LevelPrologue;

    type LowerLevel = LevelMapping;

    unsafe fn create() -> Self {
        Self {
            phantom: PhantomData,
        }
    }

    fn level() -> usize {
        2
    }
}

/// [`Adapter`] for [`LevelPaging`] to [`LevelPrologue`]
pub struct AdapterPagingPrologue {
    phantom: PhantomData<Self>,
}

/// [`AdapterGuard`] for [`LevelPaging`] to [`LevelPrologue`]
pub struct AdapterGuardPagingPrologue {
    phantom: PhantomData<Self>,
}

impl Adapter<LevelPaging, LevelPrologue, AdapterGuardPagingPrologue> for AdapterPagingPrologue {
    fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl AdapterGuard<LevelPaging, LevelPrologue> for AdapterGuardPagingPrologue {
    unsafe fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

/// [`Adapter`] for [`LevelPaging`] to [`LevelLockedPrologue`]
pub struct AdapterPagingLockedPrologue {
    phantom: PhantomData<Self>,
}

/// [`AdapterGuard`] for [`LevelPaging`] to [`LevelLockedPrologue`]
pub struct AdapterGuardPagingLockedPrologue {
    phantom: PhantomData<Self>,
}

impl Adapter<LevelPaging, LevelLockedPrologue, AdapterGuardPagingLockedPrologue>
    for AdapterPagingLockedPrologue
{
    fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl AdapterGuard<LevelPaging, LevelLockedPrologue> for AdapterGuardPagingLockedPrologue {
    unsafe fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

/// Default execution [`Level`] for `prologue`s (for more details, see [`TrapHandler`](crate::trap::handlers::TrapHandler))
pub struct LevelPrologue {
    phantom: PhantomData<Self>,
}

impl Level for LevelPrologue {
    type HigherLevel = LevelLockedPrologue;

    type LowerLevel = LevelPaging;

    unsafe fn create() -> Self {
        Self {
            phantom: PhantomData,
        }
    }

    fn level() -> usize {
        1
    }
}

/// [`Adapter`] for [`LevelPrologue`] to [`LevelLockedPrologue`]
pub struct AdapterPrologueLockedPrologue {
    phantom: PhantomData<Self>,
}

/// [`AdapterGuard`] for [`LevelPrologue`] to [`LevelLockedPrologue`]
pub struct AdapterGuardPrologueLockedPrologue {
    phantom: PhantomData<Self>,
}

impl Adapter<LevelPrologue, LevelLockedPrologue, AdapterGuardPrologueLockedPrologue>
    for AdapterPrologueLockedPrologue
{
    fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl AdapterGuard<LevelPrologue, LevelLockedPrologue> for AdapterGuardPrologueLockedPrologue {
    unsafe fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

/// Pseudo-[`Level`] for acquiring a [`IRQTicketLocks`](crate::sync::ticketlock::IRQTicketlock) which should also be safely possible within a `prologue` (for more details, see [`TrapHandler`](crate::trap::handlers::TrapHandler))
pub struct LevelLockedPrologue {
    phantom: PhantomData<Self>,
}

impl Level for LevelLockedPrologue {
    type HigherLevel = LevelInvalid;

    type LowerLevel = LevelPrologue;

    unsafe fn create() -> Self {
        Self {
            phantom: PhantomData,
        }
    }

    fn level() -> usize {
        0
    }
}
