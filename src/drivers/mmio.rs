//! Helpers for *M*emory-*M*apped-*IO*.
use crate::kernel::address;
use crate::kernel::address::Address;

use core::error;
use core::fmt;
use core::mem;

/// Abstraction of memory-mapped IO space.
pub struct MMIOSpace {
    addr: address::VirtualAddress<u8>,
    size: usize,
}

impl MMIOSpace {
    /// Create a new memory-mapped IO space.
    pub const unsafe fn new(addr: address::VirtualAddress<u8>, size: usize) -> Self {
        Self { addr, size }
    }

    /// Load value from memory-mapped IO space while performing required bounds checks.
    ///
    /// * `offset`: Byte offset within memory space.
    pub fn load<T: Sized>(&self, offset: usize) -> Result<T, MMIOSpaceError> {
        /* Perform bounds check */
        if offset + mem::size_of::<T>() > self.size {
            return Err(MMIOSpaceError::OutOfBoundsAccess);
        }

        /* Perform access */
        unsafe {
            let ptr: address::VirtualAddress<T> = self.addr.byte_add(offset).cast();
            let element = ptr.as_ptr().read_volatile();
            return Ok(element);
        }
    }

    /// Store value in memory-mapped IO space while performing required bounds checks.
    ///
    /// * `offset`: Byte offset within memory space.
    /// * `element`: Source element
    pub fn store<T: Sized>(&mut self, offset: usize, element: T) -> Result<(), MMIOSpaceError> {
        /* Perform bounds check */
        if offset + mem::size_of::<T>() > self.size {
            return Err(MMIOSpaceError::OutOfBoundsAccess);
        }

        /* Perform access */
        unsafe {
            let mut ptr: address::VirtualAddress<T> = self.addr.byte_add(offset).cast();
            ptr.as_mut_ptr().write_volatile(element);
            return Ok(());
        }
    }
}

/// Error in repsect to memory-mapped IO accesses.
#[derive(Debug)]
pub enum MMIOSpaceError {
    /// Out-of-bounds access.
    OutOfBoundsAccess,
}

unsafe impl Sync for MMIOSpace {}

unsafe impl Send for MMIOSpace {}

impl fmt::Display for MMIOSpaceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MMIOSpaceError::OutOfBoundsAccess => write!(f, "Out of bounds access"),
        }
    }
}

impl error::Error for MMIOSpaceError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }

    fn description(&self) -> &str {
        "description() is deprecated; use Display"
    }

    fn cause(&self) -> Option<&dyn error::Error> {
        self.source()
    }
}
