//! Provide Core-local Storage

use core::array;
use core::cell::UnsafeCell;

use crate::config;
use crate::kernel::cpu::TP;

/// Core-local Storage
pub struct PerCore<T>([UnsafeCell<T>; config::MAX_CPU_NUM]);

impl<T: Clone> PerCore<T> {
    /// Create core-local storage, where each element `T` is generated  using [`clone`][core::clone::Clone].
    pub fn new_clone(value: T) -> Self {
        let per_core = array::from_fn(|_| UnsafeCell::new(value.clone()));
        Self(per_core)
    }
}

impl<T: Copy> PerCore<T> {
    /// Create core-local storage, where each element `T` is generated  using [`copy`][core::marker::Copy].
    pub fn new_copy(value: T) -> Self {
        let per_core = array::from_fn(|_| UnsafeCell::new(value));
        Self(per_core)
    }
}

impl<T> PerCore<T> {
    /// Creates core-local storage, where each element `T` is the returned value from `cb` using
    /// that element’s index.
    pub fn new_fn<F>(cb: F) -> Self
    where
        F: FnMut(usize) -> T,
    {
        let mut cb = cb;
        let per_core = array::from_fn(|index| UnsafeCell::new(cb(index)));
        Self(per_core)
    }

    /// Gets a shared reference to the corresponding `T`.
    pub fn get(&self) -> &T {
        let mut tp = TP::new(0);
        tp.read();

        // # Safety
        // The `get` function on its own is completly safe to use. However, the misuse of `get_mut`
        // may cause race conditions, thus, `get_mut` is considered `unsafe`.
        unsafe { self.0[tp.raw() as usize].get().as_ref().unwrap() }
    }

    /// Gets a mutable reference to the corresponding `T`.
    ///
    /// # Caution
    ///
    /// Do **not** hold any shared/exclusive references while invoking a (potentially) blocking
    /// operation! The operationg may trigger a re-scheduling of the current thread onto a
    /// potentially other hart. This violates the [`Sync`]/[`Send`] requirements.
    pub unsafe fn get_mut(&self) -> &mut T {
        let mut tp = TP::new(0);
        tp.read();
        self.0[tp.raw() as usize].get().as_mut().unwrap()
    }
}

unsafe impl<T> Sync for PerCore<T> {}

unsafe impl<T> Send for PerCore<T> {}
