//! Provide Core-local Storage

use core::array;
use core::borrow::{Borrow, BorrowMut};
use core::cell::UnsafeCell;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};

use crate::arch::cpu;
use crate::config;
use crate::sync::level::Level;

use super::level::LevelScheduler;

/// Core-local Storage on [`Level`] `UpperLevel` and `LowerLevel`.
///
/// To prevent core switching upon rescheduling, the [`PerCore`] will require a token on [`Level`]
/// `UpperLevel` and produce a token on `LowerLevel`. Hereby, `LowerLevel` **must** be **below**
/// [`LevelScheduler`] to garantuee soundess during potential blocking operations.
pub struct PerCore<T, UpperLevel: Level, LowerLevel: Level> {
    values: [UnsafeCell<T>; config::MAX_CPU_NUM],
    upper_level_phantom: PhantomData<UpperLevel>,
    lower_level_phantom: PhantomData<LowerLevel>,
}

impl<T: Clone, UpperLevel: Level, LowerLevel: Level> PerCore<T, UpperLevel, LowerLevel> {
    /// Create core-local storage, where each element `T` is generated  using [`clone`][core::clone::Clone].
    pub fn new_clone(value: T) -> Self {
        // Create values
        let values = array::from_fn(|_| UnsafeCell::new(value.clone()));

        // Check if chosen levels are suitable
        assert!(LowerLevel::level() < LevelScheduler::level());

        Self {
            values,
            upper_level_phantom: PhantomData,
            lower_level_phantom: PhantomData,
        }
    }
}

impl<T: Copy, UpperLevel: Level, LowerLevel: Level> PerCore<T, UpperLevel, LowerLevel> {
    /// Create core-local storage, where each element `T` is generated  using [`copy`][core::marker::Copy].
    pub fn new_copy(value: T) -> Self {
        // Create values
        let values = array::from_fn(|_| UnsafeCell::new(value));

        // Check if chosen levels are suitable
        assert!(LowerLevel::level() < LevelScheduler::level());

        Self {
            values,
            upper_level_phantom: PhantomData,
            lower_level_phantom: PhantomData,
        }
    }
}

impl<T, UpperLevel: Level, LowerLevel: Level> PerCore<T, UpperLevel, LowerLevel> {
    /// Creates core-local storage, where each element `T` is the returned value from `cb` using
    /// that element’s index.
    pub fn new_fn<F>(cb: F) -> Self
    where
        F: FnMut(usize) -> T,
    {
        // Create values
        let mut cb = cb;
        let values = array::from_fn(|index| UnsafeCell::new(cb(index)));

        // Check if chosen levels are suitable
        assert!(LowerLevel::level() < LevelScheduler::level());

        Self {
            values,
            upper_level_phantom: PhantomData,
            lower_level_phantom: PhantomData,
        }
    }

    /// Gets a shared reference to the corresponding `T`.
    pub fn get(
        &self,
        token: UpperLevel,
    ) -> (PerCoreGuard<'_, T, UpperLevel, LowerLevel>, LowerLevel) {
        // Consume `UpperLevel`
        let _ = token;

        // # Safety
        // The `get` function on its own is completly safe to use. However, the misuse of `get_mut`
        // may cause race conditions, thus, `get_mut` is considered `unsafe`.
        let value = unsafe { self.values[cpu::current().raw()].get().as_ref().unwrap() };
        let guard = PerCoreGuard {
            value,
            upper_level_phantom: PhantomData,
            lower_level_phantom: PhantomData,
        };

        // Produce `LowerLevel`
        let token = unsafe { LowerLevel::create() };

        (guard, token)
    }

    /// Gets a mutable reference to the corresponding `T`.
    pub fn get_mut(
        &self,
        token: UpperLevel,
    ) -> (PerCoreMutGuard<'_, T, UpperLevel, LowerLevel>, LowerLevel) {
        // Consume `UpperLevel`
        let _ = token;

        let value = unsafe { self.values[cpu::current().raw()].get().as_mut().unwrap() };
        let guard = PerCoreMutGuard {
            value,
            upper_level_phantom: PhantomData,
            lower_level_phantom: PhantomData,
        };

        // Produce `LowerLevel`
        let token = unsafe { LowerLevel::create() };

        (guard, token)
    }
}

unsafe impl<T, UpperLevel: Level, LowerLevel: Level> Sync for PerCore<T, UpperLevel, LowerLevel> {}

unsafe impl<T, UpperLevel: Level, LowerLevel: Level> Send for PerCore<T, UpperLevel, LowerLevel> {}

/// Guard for holding shared reference to entry within [`PerCore`].
#[derive(Debug)]
pub struct PerCoreGuard<'a, T, UpperLevel: Level, LowerLevel: Level> {
    value: &'a T,
    upper_level_phantom: PhantomData<UpperLevel>,
    lower_level_phantom: PhantomData<LowerLevel>,
}

impl<'a, T, UpperLevel: Level, LowerLevel: Level> PerCoreGuard<'a, T, UpperLevel, LowerLevel> {
    /// Destroy guard to procduce `UpperLevel` while consuming `LowerLevel`.
    pub fn destroy(self, token: LowerLevel) -> UpperLevel {
        // Consume `LowerLevel`
        let _ = token;

        // Consume `UpperLevel`
        unsafe { UpperLevel::create() }
    }
}

impl<'a, T, UpperLevel: Level, LowerLevel: Level> AsRef<T>
    for PerCoreGuard<'a, T, UpperLevel, LowerLevel>
{
    fn as_ref(&self) -> &T {
        self.value
    }
}

impl<'a, T, UpperLevel: Level, LowerLevel: Level> Borrow<T>
    for PerCoreGuard<'a, T, UpperLevel, LowerLevel>
{
    fn borrow(&self) -> &T {
        self.value
    }
}

impl<'a, T, UpperLevel: Level, LowerLevel: Level> Deref
    for PerCoreGuard<'a, T, UpperLevel, LowerLevel>
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}

/// Guard for holding exclusive reference to entry within [`PerCore`].
#[derive(Debug)]
pub struct PerCoreMutGuard<'a, T, UpperLevel: Level, LowerLevel: Level> {
    value: &'a mut T,
    upper_level_phantom: PhantomData<UpperLevel>,
    lower_level_phantom: PhantomData<LowerLevel>,
}

impl<'a, T, UpperLevel: Level, LowerLevel: Level> PerCoreMutGuard<'a, T, UpperLevel, LowerLevel> {
    /// Destroy guard to procduce `UpperLevel` while consuming `LowerLevel`.
    pub fn destroy(self, token: LowerLevel) -> UpperLevel {
        // Consume `LowerLevel`
        let _ = token;

        // Consume `UpperLevel`
        unsafe { UpperLevel::create() }
    }
}

impl<'a, T, UpperLevel: Level, LowerLevel: Level> AsRef<T>
    for PerCoreMutGuard<'a, T, UpperLevel, LowerLevel>
{
    fn as_ref(&self) -> &T {
        self.value
    }
}

impl<'a, T, UpperLevel: Level, LowerLevel: Level> Borrow<T>
    for PerCoreMutGuard<'a, T, UpperLevel, LowerLevel>
{
    fn borrow(&self) -> &T {
        self.value
    }
}

impl<'a, T, UpperLevel: Level, LowerLevel: Level> Deref
    for PerCoreMutGuard<'a, T, UpperLevel, LowerLevel>
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<'a, T, UpperLevel: Level, LowerLevel: Level> AsMut<T>
    for PerCoreMutGuard<'a, T, UpperLevel, LowerLevel>
{
    fn as_mut(&mut self) -> &mut T {
        self.value
    }
}

impl<'a, T, UpperLevel: Level, LowerLevel: Level> BorrowMut<T>
    for PerCoreMutGuard<'a, T, UpperLevel, LowerLevel>
{
    fn borrow_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

impl<'a, T, UpperLevel: Level, LowerLevel: Level> DerefMut
    for PerCoreMutGuard<'a, T, UpperLevel, LowerLevel>
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value
    }
}
