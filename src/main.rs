//! *RROS* - *R*ust for *R*ISC-V *O*perating *S*ystem

#![no_std]
#![no_main]
#![warn(missing_docs)]
#![feature(error_in_core)]

use core::panic::PanicInfo;

use boot::device_tree::dt::DeviceTree;
use drivers::driver::Driver;
use sync::level::Level;

use crate::sync::epilogue;

pub mod arch;
pub mod boot;
pub mod config;
pub mod drivers;
pub mod kernel;
pub mod mm;
pub mod sync;
pub mod trap;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // Detect potential recursion!
    static RECURSION_DETECTION: core::sync::atomic::AtomicBool =
        core::sync::atomic::AtomicBool::new(false);
    while RECURSION_DETECTION
        .compare_exchange(
            false,
            true,
            core::sync::atomic::Ordering::Relaxed,
            core::sync::atomic::Ordering::Relaxed,
        )
        .is_ok()
    {
        // First hart will print emergency message
        printk!(kernel::printer::LogLevel::Emergency, "Panic: {}!", info);
    }

    // Dying...
    kernel::cpu::die();
}

fn synchronize(token: sync::level::LevelEpilogue) -> sync::level::LevelEpilogue {
    static COUNTER: core::sync::atomic::AtomicUsize = core::sync::atomic::AtomicUsize::new(0);

    COUNTER.fetch_add(1, core::sync::atomic::Ordering::Relaxed);

    while COUNTER.load(core::sync::atomic::Ordering::Relaxed) % kernel::cpu_map::online_harts() != 0
    {
        core::hint::spin_loop();
    }

    token
}

/// Kernel initialization routine entered by boot processor
#[no_mangle]
pub extern "C" fn kernel_init(hart_id: u64, dtb_ptr: *const u8, dtb_size: u32) -> ! {
    let hart_id = arch::cpu::HartID::new(hart_id);

    // Create initialization token
    // # Safety
    // The `LevelInitialization` token is dedicated to mark the initialization routine of the
    // operating system itself. Thus, completely safe to use within `kernel_init`.
    let level_initialization = unsafe { sync::level::LevelInitialization::create() };

    // Initialize page frame allocator
    let level_initialization =
        mm::page_allocator::PageFrameAllocator::initialize(level_initialization);

    // Initalize fine-grained kernel mapping
    let level_initialization = mm::mapping::VirtualMemorySystem::initalize(level_initialization);
    mm::mapping::KERNEL_VIRTUAL_MEMORY_SYSTEM.as_ref().load();

    // Load mapping
    mm::mapping::KERNEL_VIRTUAL_MEMORY_SYSTEM.as_ref().load();

    // Initialize device tree
    // # Safety
    // The provided pointer to the device tree blob is valid and thus safe to use.
    let (device_tree, level_initialization) =
        unsafe { DeviceTree::initialize(dtb_ptr, dtb_size, level_initialization) };
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

    // Initialize CPU map
    let level_initialization = kernel::cpu_map::initialize(level_initialization);

    // Write logical core ID to thread register
    let logical_id = kernel::cpu_map::lookup_logical_id(hart_id);
    let tp = arch::cpu::TP::new(u64::try_from(logical_id.raw()).unwrap());
    tp.write();

    // Initialize trap vector
    trap::handlers::load_trap_vector();

    // Initialize default trap handlers
    let level_initialization = trap::handlers::TrapHandlers::initialize(level_initialization);

    // Initialize interrupt controller
    let level_initialization =
        match trap::intc::InterruptController::initiailize(level_initialization) {
            Ok(token) => token,
            Err((error, _)) => panic!("Unable to initialize UART driver: {}!", error),
        };

    // Initialize serial driver
    let level_initialization = match drivers::uart::Uart::initiailize(level_initialization) {
        Ok(token) => token,
        Err((error, _)) => panic!("Unable to initialize UART driver: {}!", error),
    };

    let level_initialization = match drivers::rtc::RealTimeClock::initiailize(level_initialization)
    {
        Ok(token) => token,
        Err((error, _)) => panic!("Unable to initialize timer driver: {}!", error),
    };

    // Finalize trap handlers **after** initialization of drivers
    let level_initialization = trap::handlers::TrapHandlers::finalize(level_initialization);

    // Initialize global printer
    let level_initialization = match kernel::printer::initialize(level_initialization) {
        Ok(token) => token,
        Err((error, _)) => panic!("Unable to initialize global printer: {}!", error),
    };

    // Boot application processors
    kernel::boot_ap::startup(level_initialization);

    // Enter epilogue level
    let level_epilogue = epilogue::try_enter().unwrap();

    // Enable interrupts
    unsafe {
        arch::cpu::unmask_all_interrupts();
        kernel::cpu::enable_interrupts();
    }

    // Synchronize with remaining harts
    let level_epilogue = synchronize(level_epilogue);

    printk!(
        kernel::printer::LogLevel::Info,
        "Core {}: Finished initialization\n",
        kernel::cpu::current()
    );

    loop {}
}

/// Kernel initialization routine entered by application processors
#[no_mangle]
pub extern "C" fn kernel_ap_init(hart_id: u64) -> ! {
    let hart_id = arch::cpu::HartID::new(hart_id);

    // Write logical core ID to thread register
    let logical_id = kernel::cpu_map::lookup_logical_id(hart_id);
    let tp = arch::cpu::TP::new(u64::try_from(logical_id.raw()).unwrap());
    tp.write();

    // Initialize trap vector
    trap::handlers::load_trap_vector();

    // Initalize fine-grained kernel mapping
    mm::mapping::KERNEL_VIRTUAL_MEMORY_SYSTEM.as_ref().load();

    // Enter epilogue level
    let level_epilogue = epilogue::try_enter().unwrap();

    // Enable interrupts
    unsafe {
        arch::cpu::unmask_all_interrupts();
        kernel::cpu::enable_interrupts();
    }

    // Synchronize with remaining harts
    let level_epilogue = synchronize(level_epilogue);

    printk!(
        kernel::printer::LogLevel::Info,
        "Core {}: Finished initialization\n",
        kernel::cpu::current()
    );

    loop {}
}
