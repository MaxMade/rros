//! Cell to statically guarantee read-only access to e.g. struct members.

use core::borrow::Borrow;
use core::ops::Deref;

/// True constant cell.
pub struct ConstCell<T>(T);

impl<T> ConstCell<T> {
    /// Create an new uninitialized cell.
    pub const fn new(value: T) -> Self {
        ConstCell(value)
    }
}

impl<T> AsRef<T> for ConstCell<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T> Borrow<T> for ConstCell<T> {
    fn borrow(&self) -> &T {
        &self.0
    }
}

impl<T> Deref for ConstCell<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

unsafe impl<T: Sync> Sync for ConstCell<T> {}

unsafe impl<T: Send> Send for ConstCell<T> {}
