//! Spin-based ticket lock implementing [Level] design.

use core::cell::UnsafeCell;
use core::hint;
use core::marker::PhantomData;
use core::ops::Deref;
use core::ops::DerefMut;
use core::sync::atomic::AtomicUsize;
use core::sync::atomic::Ordering;

use crate::kernel::cpu;
use crate::kernel::cpu::InterruptFlag;
use crate::sync::level::Level;

use crate::sync::level::LevelDriver;
use crate::sync::level::LevelEpilogue;
use crate::sync::level::LevelInitialization;
use crate::sync::level::LevelLockedPrologue;
use crate::sync::level::LevelMapping;
use crate::sync::level::LevelMemory;
use crate::sync::level::LevelPaging;
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
        Self {
            data: UnsafeCell::new(value),
            ticket: AtomicUsize::new(0),
            counter: AtomicUsize::new(0),
            phantom: PhantomData,
        }
    }

    /// Returns a mutable reference to the underlying data.
    ///
    /// Since this call borrows the [`Ticketlock`] mutably, no actual locking needs to take place –
    /// the mutable borrow statically guarantees no locks exist.
    pub fn get_mut(&mut self) -> &mut T {
        unsafe { &mut *self.data.get() }
    }

    /// Acquire lock while consume `UpperLevel` `token` (and producing `LowerLevel` `token`).
    #[inline]
    pub fn lock(
        &self,
        token: UpperLevel,
    ) -> (TicketlockGuard<'_, T, UpperLevel, LowerLevel>, LowerLevel) {
        // Consume UpperLevel token
        let _ = token;

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

        // Produce LowerLevel token
        //
        // # Safety
        // This Ticketlock synchronization primitive implements the strict hierarchical level per
        // design.
        let token = unsafe { LowerLevel::create() };

        return (guard, token);
    }

    /// Acquire lock during initialization.
    #[inline]
    pub fn init_lock(
        &self,
        token: LevelInitialization,
    ) -> TicketlockGuard<'_, T, LevelInitialization, LevelInitialization> {
        // Consume UpperLevel token
        let _ = token;

        // Create ticket lock guard
        TicketlockGuard {
            counter: &self.counter,
            data: unsafe { &mut *self.data.get() },
            phantom: PhantomData,
        }
    }

    /// Try to acquire lock while consume `UpperLevel` `token` (and producing `LowerLevel` `token`).
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

        // Produce LowerLevel token
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

    /// Consume this [`Ticketlock`] and unwraps the underlying data.
    pub fn into_inner(self) -> T {
        self.data.into_inner()
    }

    /// Get raw pointer underlying data **without** acquiring the lock or strict hierarchical
    /// constraints.
    ///
    /// # Safety
    /// This function is per definition `unsafe` and it is the responsibility of
    pub const unsafe fn as_ptr(&self) -> *mut T {
        self.data.get()
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
pub struct TicketlockGuard<'a, T: 'a, UpperLevel: Level, LowerLevel: Level> {
    data: &'a mut T,
    counter: &'a AtomicUsize,
    phantom: PhantomData<(UpperLevel, LowerLevel)>,
}

impl<'a, T, UpperLevel: Level, LowerLevel: Level> TicketlockGuard<'a, T, UpperLevel, LowerLevel> {
    /// Release lock while consume `LowerLevel` `token` (and producing `UpperLevel` `token`).
    #[inline]
    pub fn unlock(self, token: LowerLevel) -> UpperLevel {
        // Consume UpperLevel token
        let _ = token;

        // Release lock
        self.counter.fetch_add(1, Ordering::Release);

        // Produce LowerLevel token
        //
        // # Safety
        // This Ticketlock synchronization primitive implements the strict hierarchical level per
        // design.
        let token = unsafe { UpperLevel::create() };
        return token;
    }

    /// Release lock while consume `LowerLevel` `token` (and producing `UpperLevel` `token`).
    #[inline]
    pub fn init_unlock(self) -> LevelInitialization {
        unsafe { LevelInitialization::create() }
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
pub type TicketlockMemory<T> = Ticketlock<T, LevelMemory, LevelMapping>;

/// Specialized [`Ticketlock`] for locking `Mapping` level.
pub type TicketlockMapping<T> = Ticketlock<T, LevelMapping, LevelPaging>;

/// Specialized [`Ticketlock`] for locking `Paging` level.
pub type TicketlockPaging<T> = Ticketlock<T, LevelPaging, LevelPrologue>;

/// Interrupt-safe Ticketlock
pub struct IRQTicketlock<T> {
    lock: Ticketlock<T, LevelPrologue, LevelLockedPrologue>,
}

impl<T> IRQTicketlock<T> {
    /// Create a new `IRQTicketlock`
    pub const fn new(value: T) -> Self {
        Self {
            lock: Ticketlock::new(value),
        }
    }

    /// Returns a mutable reference to the underlying data.
    ///
    /// Since this call borrows the [`Ticketlock`] mutably, no actual locking needs to take place –
    /// the mutable borrow statically guarantees no locks exist.
    pub fn get_mut(&mut self) -> &mut T {
        self.lock.get_mut()
    }

    /// Disable interrupts and acquire lock (and saving [`InterruptFlag`]) while consume [`LevelPrologue`] `token` (and producing
    /// [`LevelLockedPrologue`] `token`).
    #[inline]
    pub fn lock(
        &self,
        token: LevelPrologue,
    ) -> (
        IRQTicketlockGuard<'_, T, LevelPrologue, LevelLockedPrologue>,
        LevelLockedPrologue,
    ) {
        let (flag, token) = cpu::save_and_disable_interrupts(token);
        let (guard, token) = self.lock.lock(token);

        let guard = IRQTicketlockGuard { guard, flag };

        return (guard, token);
    }

    /// Disable interrupts and acquire lock during initialization without do anything at all.
    #[inline]
    pub fn init_lock(
        &self,
        token: LevelInitialization,
    ) -> IRQTicketlockGuard<'_, T, LevelInitialization, LevelInitialization> {
        let guard = self.lock.init_lock(token);
        let guard = IRQTicketlockGuard {
            guard,
            flag: unsafe { InterruptFlag::new() },
        };

        return guard;
    }

    /// Try to disable interrupts and acquire lock (and saving [`InterruptFlag`]) while consume [`LevelPrologue`] `token` (and producing
    /// [`LevelLockedPrologue`] `token`).
    #[inline]
    pub fn try_lock(
        &self,
        token: LevelPrologue,
    ) -> Result<
        (
            IRQTicketlockGuard<'_, T, LevelPrologue, LevelLockedPrologue>,
            LevelLockedPrologue,
        ),
        LevelPrologue,
    > {
        let (flag, token) = cpu::save_and_disable_interrupts(token);

        let (guard, token) = match self.lock.try_lock(token) {
            Ok((guard, token)) => (guard, token),
            Err(token) => {
                cpu::restore_interrupts(flag);
                return Err(token);
            }
        };

        let guard = IRQTicketlockGuard { guard, flag };

        return Ok((guard, token));
    }

    /// Return `true` if the lock is currently held.
    #[inline]
    pub fn is_locked(&self) -> bool {
        self.lock.is_locked()
    }

    /// Consume this [`Ticketlock`] and unwraps the underlying data.
    pub fn into_inner(self) -> T {
        self.lock.into_inner()
    }

    /// Get raw pointer underlying data **without** acquiring the lock or strict hierarchical
    /// constraints.
    ///
    /// # Safety
    /// This function is per definition `unsafe` and it is the responsibility of
    pub const unsafe fn as_ptr(&self) -> *mut T {
        self.lock.as_ptr()
    }
}

unsafe impl<T: Send> Sync for IRQTicketlock<T> {}

unsafe impl<T: Send> Send for IRQTicketlock<T> {}

/// Interrupt-safe ticketlock guard.
pub struct IRQTicketlockGuard<'a, T: 'a, UpperLevel: Level, LowerLevel: Level> {
    guard: TicketlockGuard<'a, T, UpperLevel, LowerLevel>,
    flag: InterruptFlag<UpperLevel>,
}

impl<'a, T, UpperLevel: Level, LowerLevel: Level>
    IRQTicketlockGuard<'a, T, UpperLevel, LowerLevel>
{
    /// Release lock and restoring the saved [`InterruptFlag`] while consume `LowerLevel` `token`
    /// (and producing `UpperLevel` `token`).
    #[inline]
    pub fn unlock(self, token: LowerLevel) -> UpperLevel {
        let token = self.guard.unlock(token);
        cpu::restore_interrupts(self.flag);
        return token;
    }

    /// Release lock and restoring the saved [`InterruptFlag`]
    /// and producing [`LevelPrologue`] `token` without doing anything at all
    #[inline]
    pub fn init_unlock(self) -> LevelInitialization {
        self.guard.init_unlock()
    }
}

impl<'a, T, UpperLevel: Level, LowerLevel: Level> Deref
    for IRQTicketlockGuard<'a, T, UpperLevel, LowerLevel>
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.guard.deref()
    }
}

impl<'a, T, UpperLevel: Level, LowerLevel: Level> DerefMut
    for IRQTicketlockGuard<'a, T, UpperLevel, LowerLevel>
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.guard.deref_mut()
    }
}
