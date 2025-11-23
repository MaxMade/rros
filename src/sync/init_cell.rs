//! Cell to provide safe read-/write-access during its initialization phase and shared access
//! afterwards.

use core::borrow::Borrow;
use core::borrow::BorrowMut;
use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use core::ops::Deref;
use core::ops::DerefMut;

use crate::sync::level::Level;
use crate::sync::level::LevelInitialization;

/// Cell used for safe initialization.
pub struct InitCell<T> {
    initialized: UnsafeCell<bool>,
    value: UnsafeCell<MaybeUninit<T>>,
}

impl<T> InitCell<T> {
    /// Create a new uninitialized cell.
    pub const fn new() -> Self {
        Self {
            initialized: UnsafeCell::new(false),
            value: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }

    /// Get a shared reference to the inner value.
    pub fn get(&self) -> &T {
        let initialized = unsafe { self.initialized.get().as_ref().unwrap() };
        if !initialized {
            panic!("Tried to access non-initialized InitCell!");
        }

        let value = unsafe { self.value.get().as_ref().unwrap().assume_init_ref() };
        value
    }

    /// Finanlize initialization routine
    pub unsafe fn finanlize(&self, token: LevelInitialization) -> LevelInitialization {
        let initialized = unsafe { self.initialized.get().as_mut().unwrap() };
        if *initialized {
            panic!("Tried to update initialized (read-only) InitCell!");
        }
        *initialized = true;
        token
    }

    /// Get a exclusive reference to the inner value.
    pub fn get_mut(&self, token: LevelInitialization) -> InitCellGuard<'_, T> {
        let initialized = unsafe { self.initialized.get().as_mut().unwrap() };
        if *initialized {
            panic!("Tried to update initialized (read-only) InitCell!");
        }

        // Consume token
        let _ = token;

        let value = unsafe { self.value.get().as_mut().unwrap().assume_init_mut() };
        InitCellGuard { value }
    }
}

impl<T> AsRef<T> for InitCell<T> {
    fn as_ref(&self) -> &T {
        self.get()
    }
}

impl<T> Borrow<T> for InitCell<T> {
    fn borrow(&self) -> &T {
        self.get()
    }
}

unsafe impl<T: Sync> Sync for InitCell<T> {}

unsafe impl<T: Send> Send for InitCell<T> {}

/// Guard for holding exclusive reference to entry within [`InitCell`] during initialization.
#[derive(Debug)]
pub struct InitCellGuard<'a, T> {
    value: &'a mut T,
}

impl<'a, T> InitCellGuard<'a, T> {
    /// Destroy guard to produce [`LevelInitialization`] while consuming [`InitCellGuard`].
    pub fn destroy(self) -> LevelInitialization {
        // Create `LevelInitialization`
        unsafe { LevelInitialization::create() }
    }
}

impl<'a, T> AsRef<T> for InitCellGuard<'a, T> {
    fn as_ref(&self) -> &T {
        self.value
    }
}

impl<'a, T> Borrow<T> for InitCellGuard<'a, T> {
    fn borrow(&self) -> &T {
        self.value
    }
}

impl<'a, T> Deref for InitCellGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<'a, T> AsMut<T> for InitCellGuard<'a, T> {
    fn as_mut(&mut self) -> &mut T {
        self.value
    }
}

impl<'a, T> BorrowMut<T> for InitCellGuard<'a, T> {
    fn borrow_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

impl<'a, T> DerefMut for InitCellGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value
    }
}
