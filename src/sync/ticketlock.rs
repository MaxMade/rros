//! Spin-based ticket lock implementing [Level](crate::sync::level::Level) design.

use core::cell::UnsafeCell;
use core::hint;
use core::marker::PhantomData;
use core::ops::Deref;
use core::ops::DerefMut;
use core::sync::atomic::AtomicUsize;
use core::sync::atomic::Ordering;

use crate::sync::level::Level;

use crate::sync::level::LevelDriver;
use crate::sync::level::LevelEpilogue;
use crate::sync::level::LevelMemory;
use crate::sync::level::LevelPrologue;
use crate::sync::level::LevelScheduler;

/// Generic Ticketlock
pub struct Ticketlock<T, UpperLevel: Level, LowerLevel: Level> {
    data: UnsafeCell<T>,
    ticket: AtomicUsize,
    counter: AtomicUsize,
    phantom: PhantomData<(UpperLevel, LowerLevel)>,
}

impl<T, UpperLevel: Level, LowerLevel: Level> Ticketlock<T, UpperLevel, LowerLevel> {
    /// Create a new `Ticketlock`
    pub const fn new(value: T) -> Self {
        // Sanity check:
        //
        // (UpperLevel::LowerLevel == LowerLevel) && (LowerLeve::HeigherLevel == UpperLevel)
        assert!(UpperLevel::level() > LowerLevel::level());

        Self {
            data: UnsafeCell::new(value),
            ticket: AtomicUsize::new(0),
            counter: AtomicUsize::new(0),
            phantom: PhantomData,
        }
    }

    /// Acquire lock while consume `UpperLevel` `token` (and producing `LowerLevel` `token`).
    #[inline]
    pub fn lock(
        &self,
        token: UpperLevel,
    ) -> (TicketlockGuard<'_, T, UpperLevel, LowerLevel>, LowerLevel) {
        // Get ticket
        let ticket = self.ticket.fetch_add(1, Ordering::Relaxed);

        // Wait for ticket
        while ticket != self.counter.load(Ordering::Acquire) {
            hint::spin_loop();
        }

        // Create ticket lock guard
        let guard = TicketlockGuard {
            counter: &self.counter,
            data: unsafe { &mut *self.data.get() },
            phantom: PhantomData,
        };

        // Create token
        //
        // # Safety
        // This Ticketlock synchronization primitive implements the strict hierarchical level per
        // design.
        let token = unsafe { LowerLevel::create() };

        return (guard, token);
    }

    #[inline]
    pub fn try_lock(
        &self,
        token: UpperLevel,
    ) -> Result<(TicketlockGuard<'_, T, UpperLevel, LowerLevel>, LowerLevel), UpperLevel> {
        let counter = self.counter.load(Ordering::Acquire);

        if self
            .ticket
            .compare_exchange(counter, counter + 1, Ordering::Relaxed, Ordering::Relaxed)
            .is_err()
        {
            return Err(token);
        }

        // Create ticket lock guard
        let guard = TicketlockGuard {
            counter: &self.counter,
            data: unsafe { &mut *self.data.get() },
            phantom: PhantomData,
        };

        // Create token
        //
        // # Safety
        // This Ticketlock synchronization primitive implements the strict hierarchical level per
        // design.
        let token = unsafe { LowerLevel::create() };

        return Ok((guard, token));
    }

    /// Return `true` if the lock is currently held.
    #[inline]
    pub fn is_locked(&self) -> bool {
        self.counter.load(Ordering::Relaxed) == self.ticket.load(Ordering::Relaxed)
    }

    /// Consume this [`TicketMutex`] and unwraps the underlying data.
    pub fn into_inner(self) -> T {
        self.data.into_inner()
    }
}

unsafe impl<T: Send, UpperLevel: Level, LowerLevel: Level> Sync
    for Ticketlock<T, UpperLevel, LowerLevel>
{
}

unsafe impl<T: Send, UpperLevel: Level, LowerLevel: Level> Send
    for Ticketlock<T, UpperLevel, LowerLevel>
{
}

/// Generic `TicketlockGuard`
struct TicketlockGuard<'a, T: 'a, UpperLevel: Level, LowerLevel: Level> {
    data: &'a mut T,
    counter: &'a AtomicUsize,
    phantom: PhantomData<(UpperLevel, LowerLevel)>,
}

impl<'a, T, UpperLevel: Level, LowerLevel: Level> TicketlockGuard<'a, T, UpperLevel, LowerLevel> {
    /// Release lock while consume `LowerLevel` `token` (and producing `UpperLevel` `token`).
    #[inline]
    pub fn unlock(self, token: LowerLevel) -> UpperLevel {
        // Release lock
        self.counter.fetch_add(1, Ordering::Release);

        // Create token
        //
        // # Safety
        // This Ticketlock synchronization primitive implements the strict hierarchical level per
        // design.
        let token = unsafe { UpperLevel::create() };
        return token;
    }
}

impl<'a, T, UpperLevel: Level, LowerLevel: Level> Deref
    for TicketlockGuard<'a, T, UpperLevel, LowerLevel>
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &*self.data
    }
}

impl<'a, T, UpperLevel: Level, LowerLevel: Level> DerefMut
    for TicketlockGuard<'a, T, UpperLevel, LowerLevel>
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.data
    }
}

/// Specialized [`Ticketlock`] for locking `Epilogue` level.
pub type TicketlockEpilogue<T> = Ticketlock<T, LevelEpilogue, LevelDriver>;

/// Specialized [`Ticketlock`] for locking `Driver` level.
pub type TicketlockDriver<T> = Ticketlock<T, LevelDriver, LevelScheduler>;

/// Specialized [`Ticketlock`] for locking `Scheduler` level.
pub type TicketlockScheduler<T> = Ticketlock<T, LevelScheduler, LevelMemory>;

/// Specialized [`Ticketlock`] for locking `Memory` level.
pub type TicketlockMemory<T> = Ticketlock<T, LevelMemory, LevelPrologue>;
