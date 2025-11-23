//! *RROS* - *R*ust for *R*ISC-V *O*perating *S*ystem

#![no_std]
#![no_main]
#![warn(missing_docs)]
#![feature(error_in_core)]

use core::panic::PanicInfo;

use boot::device_tree::dt::DeviceTree;
use sync::level::Level;

mod boot;
mod config;
mod kernel;
mod sync;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

/// Kernel initialization routine entered by boot processor
#[no_mangle]
pub extern "C" fn kernel_init(hart_id: u64, dtb_ptr: *const u8) -> ! {
    // Create initialization token
    // # Safety
    // The `LevelInitialization` token is dedicated to mark the initialization routine of the
    // operating system itself. Thus, completely safe to use within `kernel_init`.
    let level_initialization = unsafe { sync::level::LevelInitialization::create() };

    // Convert hart ID.
    let hard_id = kernel::cpu::HartID::new(hart_id);

    // Register hart
    let (logical_id, level_initialization) =
        kernel::cpu_map::register_hart(hard_id.clone(), level_initialization);

    // Initialize device tree
    // # Safety
    // The provided pointer to the device tree blob is valid and thus safe to use.
    let (device_tree, level_initialization) =
        unsafe { DeviceTree::initialize(dtb_ptr, level_initialization) };
    assert!(device_tree.get_cpu_count() < config::MAX_CPU_NUM);

    // Check availability of OpenSBI by querying specification version
    if let Err(error) = kernel::sbi::specification_version() {
        panic!("Unable to query OpenSBI version: {}", error);
    }

    // Check for OpenSBI Hart State Management Extension
    let sbi_hsm_state =
        match kernel::sbi::probe_extension(kernel::sbi::SBIExtensionID::HartStateManagement) {
            Ok(state) => state,
            Err(error) => {
                panic!("Unable to state of OpenSBI HSM extension: {}", error);
            }
        };
    if !sbi_hsm_state {
        panic!("OpenSBI HSM Extension: Unsupported!\n");
    }

    loop {}
}

/// Kernel initialization routine entered by application processors
#[no_mangle]
pub extern "C" fn kernel_ap_init(hart_id: u64) -> ! {
    let hard_id = kernel::cpu::HartID::new(hart_id);
    loop {}
}
