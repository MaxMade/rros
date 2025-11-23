use core::fmt::Debug;
use core::fmt::Display;
use core::fmt::Pointer;

use core::marker::PhantomData;
use core::ptr;

/// Common trait for all address abstractions.
pub trait Address<T>
where
    Self: Pointer + Clone + Copy + Eq + PartialEq + Ord + PartialOrd + From<*mut T> + Into<*mut T>,
{
    /// Creates a new `Address`.
    fn create(ptr: *mut T) -> Self;

    /// Use `Self` as raw immutable pointer.
    fn as_ptr(&self) -> *const T;

    /// Use `Self` as raw mutable pointer.
    fn as_mut_ptr(&mut self) -> *mut T;

    /// Calculates the offset from a pointer.
    ///
    /// count is in units of `T`; e.g., a `count` of 3 represents a pointer offset of `3 *
    /// size_of::<T>() bytes`.
    fn add(self, count: usize) -> Self {
        let ptr: *mut T = unsafe { self.into().add(count) };
        return Self::from(ptr);
    }

    /// Calculates the offset from a pointer in bytes.
    ///
    /// `count` is in units of bytes.
    ///
    /// This is purely a convenience for casting to a `u8` pointer and using add on it. See that
    /// method for documentation and safety requirements.
    ///
    /// For non-`Sized` pointees this operation changes only the data pointer, leaving the metadata
    /// untouched.
    unsafe fn byte_add(self, count: usize) -> Self {
        let ptr = self.into().cast::<u8>().add(count);
        return Self::from(ptr.cast());
    }

    /// Calculates the offset from a pointer.
    ///
    /// count is in units of `T`; e.g., a `count` of 3 represents a pointer offset of `3 *
    /// size_of::<T>() bytes`.
    fn sub(self, count: usize) -> Self {
        let ptr: *mut T = unsafe { self.into().sub(count) };
        return Self::from(ptr);
    }

    /// Calculates the offset from a pointer in bytes.
    ///
    /// `count` is in units of bytes.
    ///
    /// This is purely a convenience for casting to a `u8` pointer and using sub on it. See that
    /// method for documentation and safety requirements.
    ///
    /// For non-`Sized` pointees this operation changes only the data pointer, leaving the metadata
    /// untouched.
    unsafe fn byte_sub(self, count: usize) -> Self {
        let ptr = self.into().cast::<u8>().sub(count);
        return Self::from(ptr.cast());
    }

    /// Perform bitwise `and` on pointer.
    unsafe fn and(self, rhs: usize) -> Self {
        Self::create((self.addr() & rhs) as *mut T)
    }

    /// Perform bitwise `or` on pointer.
    unsafe fn or(self, rhs: usize) -> Self {
        Self::create((self.addr() | rhs) as *mut T)
    }

    /// Perform bitwise `xor` on pointer.
    unsafe fn xor(self, rhs: usize) -> Self {
        Self::create((self.addr() ^ rhs) as *mut T)
    }

    /// Perform bitwise `not` on pointer.
    unsafe fn not(self) -> Self {
        Self::create(!(self.addr()) as *mut T)
    }

    /// Perform bitwise `right shift` on pointer.
    unsafe fn shr(self, rhs: usize) -> Self {
        Self::create((self.addr() >> rhs) as *mut T)
    }

    /// Perform `left shift` on pointer.
    unsafe fn shl(self, rhs: usize) -> Self {
        Self::create(((self.addr()) << rhs) as *mut T)
    }

    /// Gets the “address” portion of the pointer..
    fn addr(self) -> usize {
        let ptr: *mut T = self.into();
        return ptr as usize;
    }

    /// Gets a 'NULL` pointer in the respective address space.
    fn null() -> Self {
        let ptr: *mut T = ptr::null_mut();
        return Self::from(ptr);
    }

    /// Checks if pointer is a 'NULL` pointer in the respective address space.
    fn is_null(&self) -> bool {
        let ptr: *mut T = self.clone().into();
        return ptr.is_null();
    }

    /// Returns a shared reference to the value.
    unsafe fn as_ref<'a>(&self) -> &'a T {
        return &*self.as_ptr();
    }

    /// Returns a unique reference to the value.
    unsafe fn as_mut<'a>(&mut self) -> &'a mut T {
        return &mut *self.as_mut_ptr();
    }

    /// Cast pointer to another type
    unsafe fn cast<U, V: Address<U>>(self) -> V {
        let mut input = self;
        V::from(input.as_mut_ptr().cast())
    }
}

/// Abstraction of a virtual address.
// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct VirtualAddress<T> {
    pointer: *mut T,
    phantom: PhantomData<T>,
}

impl<T> VirtualAddress<T> {
    /// Creates a new `VirtualAddress`.
    pub const fn new(pointer: *mut T) -> Self {
        Self {
            pointer,
            phantom: PhantomData,
        }
    }
}

impl<T> Debug for VirtualAddress<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "VirtualAddress({:p})", self)
    }
}

impl<T> Clone for VirtualAddress<T> {
    fn clone(&self) -> Self {
        Self {
            pointer: self.pointer,
            phantom: PhantomData,
        }
    }
}

impl<T> Copy for VirtualAddress<T> {}

impl<T> PartialEq for VirtualAddress<T> {
    fn eq(&self, other: &Self) -> bool {
        self.pointer.eq(&other.pointer)
    }
}

impl<T> Eq for VirtualAddress<T> {}

impl<T> PartialOrd for VirtualAddress<T> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.pointer.partial_cmp(&other.pointer)
    }
}

impl<T> Ord for VirtualAddress<T> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.pointer.cmp(&other.pointer)
    }
}

impl<T> Display for VirtualAddress<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:p}", self)
    }
}

impl<T> Pointer for VirtualAddress<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Pointer::fmt(&self.pointer, f)
    }
}

impl<T> From<*mut T> for VirtualAddress<T> {
    fn from(value: *mut T) -> Self {
        VirtualAddress {
            pointer: value,
            phantom: PhantomData,
        }
    }
}

impl<T> Into<*mut T> for VirtualAddress<T> {
    fn into(self) -> *mut T {
        self.pointer
    }
}

impl<T> Address<T> for VirtualAddress<T> {
    fn create(ptr: *mut T) -> Self {
        Self {
            pointer: ptr,
            phantom: PhantomData,
        }
    }

    fn as_ptr(&self) -> *const T {
        self.pointer
    }

    fn as_mut_ptr(&mut self) -> *mut T {
        self.pointer as *mut T
    }
}

/// Abstraction of a physical address.
pub struct PhysicalAddress<T> {
    pointer: *mut T,
    phantom: PhantomData<T>,
}

impl<T> PhysicalAddress<T> {
    /// Creates a new `PhysicalAddress`.
    pub const fn new(pointer: *mut T) -> Self {
        Self {
            pointer,
            phantom: PhantomData,
        }
    }
}

impl<T> Debug for PhysicalAddress<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "PhysicalAddress({:p})", self)
    }
}

impl<T> Clone for PhysicalAddress<T> {
    fn clone(&self) -> Self {
        Self {
            pointer: self.pointer,
            phantom: PhantomData,
        }
    }
}

impl<T> Copy for PhysicalAddress<T> {}

impl<T> PartialEq for PhysicalAddress<T> {
    fn eq(&self, other: &Self) -> bool {
        self.pointer.eq(&other.pointer)
    }
}

impl<T> Eq for PhysicalAddress<T> {}

impl<T> PartialOrd for PhysicalAddress<T> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.pointer.partial_cmp(&other.pointer)
    }
}

impl<T> Ord for PhysicalAddress<T> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.pointer.cmp(&other.pointer)
    }
}

impl<T> Display for PhysicalAddress<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:p}", self)
    }
}

impl<T> Pointer for PhysicalAddress<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Pointer::fmt(&self.pointer, f)
    }
}

impl<T> From<*mut T> for PhysicalAddress<T> {
    fn from(value: *mut T) -> Self {
        PhysicalAddress {
            pointer: value,
            phantom: PhantomData,
        }
    }
}

impl<T> Into<*mut T> for PhysicalAddress<T> {
    fn into(self) -> *mut T {
        self.pointer
    }
}

impl<T> Address<T> for PhysicalAddress<T> {
    fn create(ptr: *mut T) -> Self {
        Self {
            pointer: ptr,
            phantom: PhantomData,
        }
    }

    fn as_ptr(&self) -> *const T {
        self.pointer
    }

    fn as_mut_ptr(&mut self) -> *mut T {
        self.pointer as *mut T
    }
}
