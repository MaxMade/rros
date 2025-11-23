//! Information provided by compiler/linker.

use core::ffi::c_void;

use crate::kernel::address::Address;
use crate::kernel::address::PhysicalAddress;
use crate::kernel::address::VirtualAddress;

extern "C" {
    static mut __virt_text_start: c_void;
    static mut __virt_text_end: c_void;

    static mut __virt_rodata_start: c_void;
    static mut __virt_rodata_end: c_void;

    static mut __virt_data_start: c_void;
    static mut __virt_data_end: c_void;

    static mut __virt_bss_start: c_void;
    static mut __virt_bss_end: c_void;

    static mut __virt_pages_start: c_void;
    static mut __virt_pages_end: c_void;

    static mut __phys_text_start: c_void;
    static mut __phys_text_end: c_void;

    static mut __phys_rodata_start: c_void;
    static mut __phys_rodata_end: c_void;

    static mut __phys_data_start: c_void;
    static mut __phys_data_end: c_void;

    static mut __phys_bss_start: c_void;
    static mut __phys_bss_end: c_void;

    static mut __phys_pages_start: c_void;
    static mut __phys_pages_end: c_void;
}

/// Get the virtual address of the start of the `.text` segment.
pub fn text_segment_virt_start() -> VirtualAddress<c_void> {
    return VirtualAddress::from(unsafe { &mut __virt_text_start as *mut c_void });
}

/// Get the virtual address of the end of the `.text` segment.
pub fn text_segment_virt_end() -> VirtualAddress<c_void> {
    return VirtualAddress::from(unsafe { &mut __virt_text_end as *mut c_void });
}

/// Get the size of `.text` segment.
pub fn text_segment_size() -> usize {
    text_segment_virt_end().addr() - text_segment_virt_start().addr()
}

/// Get the virtual address of the start of the `.rodata` segment.
pub fn rodata_segment_virt_start() -> VirtualAddress<c_void> {
    return VirtualAddress::from(unsafe { &mut __virt_rodata_start as *mut c_void });
}

/// Get the virtual address of the end of the `.rodata` segment.
pub fn rodata_segment_virt_end() -> VirtualAddress<c_void> {
    return VirtualAddress::from(unsafe { &mut __virt_rodata_end as *mut c_void });
}

/// Get the size of `.rodata` segment.
pub fn rodata_segment_size() -> usize {
    rodata_segment_virt_end().addr() - rodata_segment_virt_start().addr()
}

/// Get the virtual address of the start of the `.data` segment.
pub fn data_segment_virt_start() -> VirtualAddress<c_void> {
    return VirtualAddress::from(unsafe { &mut __virt_data_start as *mut c_void });
}

/// Get the virtual address of the end of the `.data` segment.
pub fn data_segment_virt_end() -> VirtualAddress<c_void> {
    return VirtualAddress::from(unsafe { &mut __virt_data_end as *mut c_void });
}

/// Get the size of `.data` segment.
pub fn data_segment_size() -> usize {
    data_segment_virt_end().addr() - data_segment_virt_start().addr()
}

/// Get the virtual address of the start of the `.bss` segment.
pub fn bss_segment_virt_start() -> VirtualAddress<c_void> {
    return VirtualAddress::from(unsafe { &mut __virt_bss_start as *mut c_void });
}

/// Get the virtual address of the end of the `.bss` segment.
pub fn bss_segment_virt_end() -> VirtualAddress<c_void> {
    return VirtualAddress::from(unsafe { &mut __virt_bss_end as *mut c_void });
}

/// Get the size of `.bss` segment.
pub fn bss_segment_size() -> usize {
    bss_segment_virt_end().addr() - bss_segment_virt_start().addr()
}

/// Get the virtual address of the start of the `pages` range.
pub fn pages_mem_virt_start() -> VirtualAddress<c_void> {
    return VirtualAddress::from(unsafe { &mut __virt_pages_start as *mut c_void });
}

/// Get the virtual address of the end of the `pages` range.
pub fn pages_mem_virt_end() -> VirtualAddress<c_void> {
    return VirtualAddress::from(unsafe { &mut __virt_pages_end as *mut c_void });
}

/// Get the size of `pages` memory.
pub fn pages_mem_size() -> usize {
    pages_mem_virt_end().addr() - pages_mem_virt_start().addr()
}

/// Get the physical address of the start of the `.text` segment.
pub fn text_segment_phys_start() -> PhysicalAddress<c_void> {
    return PhysicalAddress::from(unsafe { &mut __phys_text_start as *mut c_void });
}

/// Get the physical address of the end of the `.text` segment.
pub fn text_segment_phys_end() -> PhysicalAddress<c_void> {
    return PhysicalAddress::from(unsafe { &mut __phys_text_end as *mut c_void });
}

/// Get the physical address of the start of the `.rodata` segment.
pub fn rodata_segment_phys_start() -> PhysicalAddress<c_void> {
    return PhysicalAddress::from(unsafe { &mut __phys_rodata_start as *mut c_void });
}

/// Get the physical address of the end of the `.rodata` segment.
pub fn rodata_segment_phys_end() -> PhysicalAddress<c_void> {
    return PhysicalAddress::from(unsafe { &mut __phys_rodata_end as *mut c_void });
}

/// Get the physical address of the start of the `.data` segment.
pub fn data_segment_phys_start() -> PhysicalAddress<c_void> {
    return PhysicalAddress::from(unsafe { &mut __phys_data_start as *mut c_void });
}

/// Get the physical address of the end of the `.data` segment.
pub fn data_segment_phys_end() -> PhysicalAddress<c_void> {
    return PhysicalAddress::from(unsafe { &mut __phys_data_end as *mut c_void });
}

/// Get the physical address of the start of the `.bss` segment.
pub fn bss_segment_phys_start() -> PhysicalAddress<c_void> {
    return PhysicalAddress::from(unsafe { &mut __phys_bss_start as *mut c_void });
}

/// Get the physical address of the end of the `.bss` segment.
pub fn bss_segment_phys_end() -> PhysicalAddress<c_void> {
    return PhysicalAddress::from(unsafe { &mut __phys_bss_end as *mut c_void });
}

/// Get the physical address of the start of the `pages` range.
pub fn pages_mem_phys_start() -> PhysicalAddress<c_void> {
    return PhysicalAddress::from(unsafe { &mut __phys_pages_start as *mut c_void });
}

/// Get the physical address of the end of the `pages` range.
pub fn pages_mem_phys_end() -> PhysicalAddress<c_void> {
    return PhysicalAddress::from(unsafe { &mut __phys_pages_end as *mut c_void });
}
