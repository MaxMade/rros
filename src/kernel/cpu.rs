//! Generics abstraction to interact with the current CPU state.

use core::arch::asm;
use core::marker::PhantomData;

use crate::arch::cpu::CSR;
use crate::arch::cpu::TP;
use crate::arch::sstatus::SStatus;
use crate::kernel::cpu_map::LogicalCPUID;
use crate::sync::level::Level;
use crate::sync::level::LevelPrologue;

/// Get architecture-specific [`page_size`](crate::arch::cpu::page_size).
pub const fn page_size() -> usize {
    crate::arch::cpu::page_size()
}

/// Get `current` [`LogicalCPUID`] from [`TP`] register.
pub fn current() -> LogicalCPUID {
    let mut tp = TP::new(0);
    tp.read();
    let raw_logical_id = tp.raw();
    LogicalCPUID::new(usize::try_from(raw_logical_id).unwrap())
}

/// Let the current hart enter a low-energy mode which can not be left!
pub fn die() -> ! {
    unsafe {
        disable_interrupts();
        loop {
            asm!("wfi");
        }
    }
}

/// Enable supervisor-mode interrupts (in [`SStatus`] register).
pub unsafe fn enable_interrupts() {
    let mut sstatus = SStatus::new(0);
    sstatus.read();
    sstatus.set_sie(true);
    sstatus.write();
}

/// Disable supervisor-mode interrupts (in [`SStatus`] register).
pub unsafe fn disable_interrupts() {
    let mut sstatus = SStatus::new(0);
    sstatus.read();
    sstatus.set_sie(false);
    sstatus.write();
}

#[derive(Debug)]
/// Abstraction of interrupt flag generated from [`Level`].
pub struct InterruptFlag<L: Level> {
    enabled: bool,
    phantom: PhantomData<L>,
}

impl<L: Level> InterruptFlag<L> {
    /// Create uninitialized [`InterruptFlag`]
    pub const unsafe fn new() -> InterruptFlag<L> {
        Self {
            enabled: false,
            phantom: PhantomData,
        }
    }
}

/// Save interrupt flag and disable supervisor-mode interrupts.
pub fn save_and_disable_interrupts<L: Level>(token: L) -> (InterruptFlag<L>, LevelPrologue) {
    let mut sstatus = SStatus::new(0);
    sstatus.read();
    let ret = InterruptFlag {
        enabled: sstatus.get_sie(),
        phantom: PhantomData,
    };
    sstatus.set_sie(false);
    sstatus.write();

    let token = unsafe { LevelPrologue::create() };

    return (ret, token);
}

/// Restore previous interrupt flag.
pub fn restore_interrupts<L: Level>(flag: InterruptFlag<L>) -> L {
    let mut sstatus = SStatus::new(0);
    sstatus.read();
    sstatus.set_sie(flag.enabled);
    sstatus.write();
    unsafe { L::create() }
}

/// Check if supervisor-mode interrupts are enabled.
pub fn interrupts_enabled() -> bool {
    let mut sstatus = SStatus::new(0);
    sstatus.read();
    sstatus.get_sie()
}
