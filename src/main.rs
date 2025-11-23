//! *RROS* - *R*ust for *R*ISC-V *O*perating *S*ystem

#![no_std]
#![no_main]
#![warn(missing_docs)]

use core::panic::PanicInfo;

use kernel::cpu::HartID;

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
    let _hard_id = HartID::new(hart_id);
    loop {}
}

/// Kernel initialization routine entered by application processors
#[no_mangle]
pub extern "C" fn kernel_ap_init(hart_id: u64) -> ! {
    let _hard_id = HartID::new(hart_id);
    loop {}
}
