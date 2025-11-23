//! Information provided by compiler/linker.

use core::ffi::c_void;

use crate::kernel::address::Address;
use crate::kernel::address::VirtualAddress;

extern "C" {
    static mut _text_start: c_void;
    static mut _text_end: c_void;

    static mut _rodata_start: c_void;
    static mut _rodata_end: c_void;

    static mut _data_start: c_void;
    static mut _data_end: c_void;

    static mut _bss_start: c_void;
    static mut _bss_end: c_void;

    static mut _pages_start: c_void;
    static mut _pages_end: c_void;
}

/// Get start address of `.text` segment.
pub fn text_segment_start() -> VirtualAddress<c_void> {
    return VirtualAddress::from(unsafe { &mut _text_start as *mut c_void });
}

/// Get end address of `.text` segment.
pub fn text_segment_end() -> VirtualAddress<c_void> {
    return VirtualAddress::from(unsafe { &mut _text_end as *mut c_void });
}

/// Get size of `.text` segment.
pub fn text_segment_size() -> usize {
    text_segment_end().addr() - text_segment_start().addr()
}

/// Get start address of `.rodata` segment.
pub fn rodata_segment_start() -> VirtualAddress<c_void> {
    return VirtualAddress::from(unsafe { &mut _rodata_start as *mut c_void });
}

/// Get end address of `.rodata` segment.
pub fn rodata_segment_end() -> VirtualAddress<c_void> {
    return VirtualAddress::from(unsafe { &mut _rodata_end as *mut c_void });
}

/// Get size of `.rodata` segment.
pub fn rodata_segment_size() -> usize {
    rodata_segment_end().addr() - rodata_segment_start().addr()
}

/// Get start address of `.data` segment.
pub fn data_segment_start() -> VirtualAddress<c_void> {
    return VirtualAddress::from(unsafe { &mut _data_start as *mut c_void });
}

/// Get end address of `.data` segment.
pub fn data_segment_end() -> VirtualAddress<c_void> {
    return VirtualAddress::from(unsafe { &mut _data_end as *mut c_void });
}

/// Get size of `.data` segment.
pub fn data_segment_size() -> usize {
    data_segment_end().addr() - data_segment_start().addr()
}

/// Get start address of `.bss` segment.
pub fn bss_segment_start() -> VirtualAddress<c_void> {
    return VirtualAddress::from(unsafe { &mut _bss_start as *mut c_void });
}

/// Get end address of `.bss` segment.
pub fn bss_segment_end() -> VirtualAddress<c_void> {
    return VirtualAddress::from(unsafe { &mut _bss_end as *mut c_void });
}

/// Get size of `.bss` segment.
pub fn bss_segment_size() -> usize {
    bss_segment_end().addr() - bss_segment_start().addr()
}

/// Get start address of `pages` range.
pub fn pages_mem_start() -> VirtualAddress<c_void> {
    return VirtualAddress::from(unsafe { &mut _pages_start as *mut c_void });
}

/// Get end address of `pages` range.
pub fn pages_mem_end() -> VirtualAddress<c_void> {
    return VirtualAddress::from(unsafe { &mut _pages_end as *mut c_void });
}

/// Get size of `pages` memory.
pub fn pages_mem_size() -> usize {
    pages_mem_end().addr() - pages_mem_start().addr()
}
