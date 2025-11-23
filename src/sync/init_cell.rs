//! Cell to provide safe read-/write-access during its initialization phase and shared access
//! afterwards.

use core::{cell::UnsafeCell, mem::MaybeUninit};

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
    pub fn as_ref(&self) -> &T {
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
    pub fn as_mut(&self, token: LevelInitialization) -> (&mut T, LevelInitialization) {
        let initialized = unsafe { self.initialized.get().as_mut().unwrap() };
        if *initialized {
            panic!("Tried to update initialized (read-only) InitCell!");
        }

        let value = unsafe { self.value.get().as_mut().unwrap().assume_init_mut() };
        (value, token)
    }
}

unsafe impl<T: Sync> Sync for InitCell<T> {}

unsafe impl<T: Send> Send for InitCell<T> {}
