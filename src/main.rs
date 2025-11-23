//! *RROS* - *R*ust for *R*ISC-V *O*perating *S*ystem

#![no_std]
#![no_main]
#![warn(missing_docs)]

use core::panic::PanicInfo;

use sync::level::Level;

mod config;
mod kernel;
mod sync;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

/// Kernel initialization routine entered by boot processor
#[no_mangle]
pub extern "C" fn kernel_init(hart_id: u64, _dtb_ptr: *const u8) -> ! {
    // Create initialization token
    // # Safety
    // The `LevelInitialization` token is dedicated to mark the initialization routine of the
    // operating system itself. Thus, completly safe to use within `kernel_init`.
    let level_initialization = unsafe { sync::level::LevelInitialization::create() };

    // Convert hart ID.
    let hard_id = kernel::cpu::HartID::new(hart_id);

    // Register hart
    let (logical_id, level_initialization) =
        kernel::cpu_map::register_hart(hard_id.clone(), level_initialization);

    loop {}
}

/// Kernel initialization routine entered by application processors
#[no_mangle]
pub extern "C" fn kernel_ap_init(hart_id: u64) -> ! {
    let hard_id = kernel::cpu::HartID::new(hart_id);
    loop {}
}
