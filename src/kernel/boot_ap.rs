//! Interface for booting applications processors.

use core::mem;

use crate::arch::cpu::current;
use crate::kernel::address::Address;
use crate::kernel::address::PhysicalAddress;
use crate::kernel::address::VirtualAddress;
use crate::kernel::compiler;
use crate::kernel::cpu_map;
use crate::kernel::cpu_map::LogicalCPUID;
use crate::kernel::sbi;
use crate::sync::level::LevelInitialization;

extern "C" {
    fn _start(hart_id: isize, arg: isize);
}

/// Boot applications processors.
pub fn startup(token: LevelInitialization) {
    // Consume token
    let _ = token;

    // Send boot request to all applications CPUs
    for i in 0..cpu_map::online_harts() {
        // Get LogicalCPUID
        let logical_id = LogicalCPUID::new(i);
        if logical_id == current() {
            continue;
        }

        // Get HartID
        let hart_id = cpu_map::lookup_hart_id(logical_id);

        // Calculate start address
        let entry = unsafe {
            mem::transmute::<
                unsafe extern "C" fn(isize, isize),
                *mut unsafe extern "C" fn(isize, isize),
            >(_start as _)
        };
        let offset =
            compiler::text_segment_virt_start().addr() - compiler::text_segment_phys_start().addr();
        let virt_addr = VirtualAddress::new(entry);
        let phys_addr = PhysicalAddress::new(((virt_addr.addr()) - offset) as *mut _);

        // Start hart using SBI
        if let Err(error) = sbi::start_hart(hart_id, phys_addr, 0) {
            match error {
                _ => panic!("Unable to boot application CPU: {}", error),
            }
        }
    }
}
