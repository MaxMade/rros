//! Convienient helper to access/modify CPU state.

use core::arch::asm;
use core::fmt::Display;
use core::ops::{Deref, DerefMut};

use crate::arch::sie::SIE;
use crate::kernel::address::Address;
use crate::kernel::address::PhysicalAddress;
use crate::mm::pte::PageTableEntry;

/// Get default page size (`4096` bytes)
pub const fn page_size() -> usize {
    4096
}

/// Generic abstraction of a `Control and Status Register`.
pub trait CSR {
    /// Create a new [`CSR`] from fixed the fixed value `inner`.
    fn new(inner: u64) -> Self
    where
        Self: Sized;

    /// Write current `inner` value back to register.
    fn write(&self);

    /// Read current register value and store it within [`CSR`].
    fn read(&mut self);

    /// Get `inner` value of [`CSR`].
    fn inner(&self) -> u64;
}

impl Display for dyn CSR {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:#018x}", self.inner())
    }
}

/// Abstraction of `tp` (thread pointer) register.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TP(u64);

impl TP {
    /// Create zeroed abstraction of `tp` register.
    pub fn new(value: u64) -> Self {
        Self { 0: value }
    }

    /// Load current value from `tp` register.
    pub fn read(&mut self) {
        let mut x: u64;
        unsafe {
            asm!(
                "mv {x}, tp",
                x = out(reg) x,
            );
        }
        self.0 = x;
    }

    /// Store current value to `tp` register.
    pub fn write(&self) {
        let x: u64 = self.0;
        unsafe {
            asm!(
                "mv tp, {x}",
                x = in(reg) x,
            );
        }
    }

    /// Get raw inner value.
    pub const fn raw(self) -> u64 {
        self.0
    }
}

/// Fine-grained Interrupt Pending Register
///
/// #See
/// Section `4.1.3 Supervisor Interrupt Registers (sip and sie)` of `Volume II: RISC-V Privileged Architectures`
#[derive(Debug)]
pub struct SIP(u64);

impl SIP {
    /// Create new, initialized `Supervisor Interrupt Pending` register.
    pub fn new() -> Self {
        let mut reg = SIP(0);
        reg.read();
        return reg;
    }

    /// Update value of `Supervisor Interrupt Pending` based on underlying  `sip` register.
    pub fn read(&mut self) {
        let mut x: u64;
        unsafe {
            asm!(
                "csrr {x}, sip",
                x = out(reg) x,
            );
        }
        self.0 = x;
    }

    /// Update `SIP` register based on value of `Supervisor Interrupt Pending`.
    pub fn write(&self) {
        let x: u64 = self.0;
        unsafe {
            asm!(
                "csrw sip, {x}",
                x = in(reg) x,
            );
        }
    }

    /// Check if external interrupts are pending.
    pub fn is_external_interrupt_pending(&self) -> bool {
        self.0 & (1 << 9) != 0
    }

    /// Check if timer interrupts are pending.
    pub fn is_timer_interrupt_pending(&self) -> bool {
        self.0 & (1 << 5) != 0
    }

    /// Check if software interrupts are pending.
    pub fn is_software_interrupt_pending(&self) -> bool {
        self.0 & (1 << 1) != 0
    }

    /// Mark external interrupts as enabled.
    pub fn clear_external_interrupt_pending(&mut self) {
        self.0 &= !(1 << 9);
        self.write();
    }

    /// Mark timer interrupts as enabled.
    pub fn clear_timer_interrupt_pending(&mut self) {
        self.0 &= !(1 << 5);
        self.write();
    }

    /// Mark software interrupts as enabled.
    pub fn clear_software_interrupt_pending(&mut self) {
        self.0 &= !(1 << 1);
        self.write();
    }

    /// Set all enable-bits for interrupt and write updated value back to register.
    pub fn enable_all_interrupts(&mut self) {
        self.0 = u64::MAX;
        self.write();
    }

    /// Clear all enable-bits for interrupt and write updated value back to register.
    pub fn disable_all_interrupts(&mut self) {
        self.0 = 0u64;
        self.write();
    }
}

/// Abstraction of `sscratch` register.
///
/// #See
/// `4.1.6 Supervisor Scratch Register (sscratch)` of `Volume II: RISC-V Privileged Architectures`
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SScratch(u64);

impl SScratch {
    /// Create `SScratch` from raw value.
    pub const fn new(value: u64) -> Self {
        Self { 0: value }
    }

    /// Get raw inner value.
    pub const fn raw(self) -> u64 {
        self.0
    }
}

impl Display for SScratch {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:#018x}", self.0)
    }
}

/// Abstraction of `sepc` register.
///
/// #See
/// `4.1.7 Supervisor Exception Program Counter (sepc)` of `Volume II: RISC-V Privileged
/// Architectures`
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SEPC(u64);

impl SEPC {
    /// Create `SEPC` from raw value.
    pub const fn new(value: u64) -> Self {
        Self { 0: value }
    }

    /// Get raw inner value.
    pub const fn raw(self) -> u64 {
        self.0
    }
}

impl Display for SEPC {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:#018x}", self.0)
    }
}

/// Abstraction of `scause` register.
///
/// #See
/// `4.1.8 Supervisor Cause Register (scause)` of `Volume II: RISC-V Privileged
/// Architectures`
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SCause(u64);

impl SCause {
    /// Create `SCause` from raw value.
    pub const fn new(value: u64) -> Self {
        Self { 0: value }
    }

    /// Get raw inner value.
    pub const fn raw(self) -> u64 {
        self.0
    }
}

impl Display for SCause {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:#018x}", self.0)
    }
}

/// Abstraction of `STVal` register.
///
/// #See
/// `4.1.9 Supervisor Trap Value (stval) Register` of `Volume II: RISC-V Privileged Architectures`
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct STVal(u64);

impl STVal {
    /// Create `STVal` from raw value.
    pub const fn new(value: u64) -> Self {
        Self { 0: value }
    }

    /// Get raw inner value.
    pub const fn raw(self) -> u64 {
        self.0
    }
}

impl Display for STVal {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:#018x}", self.0)
    }
}

/// Abstraction of `SATP` register.
///
/// #See
/// `4.1.11 Supervisor Address Translation and Protection (satp) Register` of `Volume II: RISC-V Privileged Architectures`
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SATP(u64);

impl SATP {
    /// Create `SATP` from raw value.
    pub fn new() -> Self {
        let mut satp = Self(0);
        satp.read();
        satp.0 |= 0x8 << 60;
        satp
    }

    /// Get raw inner value.
    pub const fn raw(self) -> u64 {
        self.0
    }

    /// Load current value from `satp` register.
    pub fn read(&mut self) {
        let mut x: u64;
        unsafe {
            asm!(
                "csrr {x}, satp",
                x = out(reg) x,
            );
        }
        self.0 = x;
    }

    /// Store current value to `sajtp` register.
    pub fn write(&self) {
        let x: u64 = self.0;
        unsafe {
            asm!(
                "csrw satp, {x}",
                x = in(reg) x,
            );
        }
    }

    /// Get address of root page table
    pub const fn get_root_page_table(&self) -> PhysicalAddress<PageTableEntry> {
        let ppn = (self.0 & 0xFFF_FFFF_FFFF) as u64;
        PhysicalAddress::new((ppn * page_size() as u64) as *mut _)
    }

    /// Set address of root page table
    pub fn set_root_page_table(&mut self, phys_addr: PhysicalAddress<PageTableEntry>) {
        let ppn = phys_addr.addr() / page_size();
        self.0 &= !0xFFF_FFFF_FFFF;
        self.0 |= ppn as u64;
    }
}

impl Display for SATP {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:#010x}", self.0)
    }
}

/// Abstraction of general-purpose register
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Register(u64);

impl Register {
    /// Create `Register` from raw value.
    pub const fn new(value: u64) -> Self {
        Self { 0: value }
    }

    /// Get raw inner value.
    pub const fn raw(self) -> u64 {
        self.0
    }
}

impl Display for Register {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:#018x}", self.0)
    }
}

impl AsRef<u64> for Register {
    fn as_ref(&self) -> &u64 {
        &self.0
    }
}

impl AsMut<u64> for Register {
    fn as_mut(&mut self) -> &mut u64 {
        &mut self.0
    }
}

impl Deref for Register {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Register {
    fn deref_mut(&mut self) -> &mut u64 {
        &mut self.0
    }
}

/// Abstraction of execution mode.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ExecutionMode {
    /// User mode.
    User,
    /// Supervisor mode.
    Supervisor,
    /// Machine mode.
    Machine,
}

impl Display for ExecutionMode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ExecutionMode::User => write!(f, "User"),
            ExecutionMode::Supervisor => write!(f, "Supervisor"),
            ExecutionMode::Machine => write!(f, "Machine"),
        }
    }
}

/// Time Register
///
/// #See
/// Section `4.1.4 Supervisor Timers and Performance Counters` of `Volume II: RISC-V Privileged Architectures`
#[derive(Debug)]
pub struct Time(u64);

impl Time {
    /// Create new, initialized [`Time`].
    pub fn new() -> Self {
        let mut reg = Time(0);
        reg.read();
        return reg;
    }

    /// Update value of [`Time`] Register based on underlying `time` register.
    pub fn read(&mut self) {
        let mut x: u64;
        unsafe {
            asm!(
                "csrr {x}, time",
                x = out(reg) x,
            );
        }
        self.0 = x;
    }

    /// Get raw inner value.
    pub const fn raw(self) -> u64 {
        self.0
    }
}

impl Display for Time {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:#018x}", self.0)
    }
}

/// Retired-Instruction-Counter Register
///
/// #See
/// Section `4.1.4 Supervisor Timers and Performance Counters` of `Volume II: RISC-V Privileged Architectures`
#[derive(Debug)]
pub struct InstructionRetiredCounter(u64);

impl InstructionRetiredCounter {
    /// Create new, initialized [`InstructionRetiredCounter`].
    pub fn new() -> Self {
        let mut reg = InstructionRetiredCounter(0);
        reg.read();
        return reg;
    }

    /// Update value of [`InstructionRetiredCounter`] Register based on underlying `instret` register.
    pub fn read(&mut self) {
        let mut x: u64;
        unsafe {
            asm!(
                "csrr {x}, instret",
                x = out(reg) x,
            );
        }
        self.0 = x;
    }

    /// Get raw inner value.
    pub const fn raw(self) -> u64 {
        self.0
    }
}

/// Cycle-Counter Register
///
/// #See
/// Section `4.1.4 Supervisor Timers and Performance Counters` of `Volume II: RISC-V Privileged Architectures`
#[derive(Debug)]
pub struct CycleCounter(u64);

impl CycleCounter {
    /// Create new, initialized [`CycleCounter`].
    pub fn new() -> Self {
        let mut reg = CycleCounter(0);
        reg.read();
        return reg;
    }

    /// Update value of [`CycleCounter`] register based on underlying `cycle` register.
    pub fn read(&mut self) {
        let mut x: u64;
        unsafe {
            asm!(
                "csrr {x}, cycle",
                x = out(reg) x,
            );
        }
        self.0 = x;
    }

    /// Get raw inner value.
    pub const fn raw(self) -> u64 {
        self.0
    }
}

/// Counter-Enable Register
///
/// #See
/// Section `4.1.5 Counter-Enable Register (scounteren)` of `Volume II: RISC-V Privileged Architectures`
#[derive(Debug)]
pub struct CounterEnable(u64);

impl CounterEnable {
    /// Create new, initialized `time`.
    pub fn new() -> Self {
        let mut reg = CounterEnable(0);
        reg.read();
        return reg;
    }

    /// Update value of [`CounterEnable`] Register based on underlying `scounteren` register.
    pub fn read(&mut self) {
        let mut x: u64;
        unsafe {
            asm!(
                "csrr {x}, scounteren",
                x = out(reg) x,
            );
        }
        self.0 = x;
    }

    /// Write value of [`CounterEnable`] Register back to underlying `scounteren` register.
    pub fn write(&self) {
        let x: u64 = self.0;
        unsafe {
            asm!(
                "csrw scounteren, {x}",
                x = in(reg) x,
            );
        }
    }

    /// Check if [`CycleCounter`] register is enabled.
    pub fn is_cycle_enabled(&self) -> bool {
        (self.0 & (1 << 0)) != 0
    }

    /// Check if [`Time`] register is enabled.
    pub fn is_time_enabled(&self) -> bool {
        (self.0 & (1 << 1)) != 0
    }

    /// Check if [`InstructionRetiredCounter`] register is enabled.
    pub fn is_instret_enabled(&self) -> bool {
        (self.0 & (1 << 2)) != 0
    }

    /// Get raw inner value.
    pub const fn raw(self) -> u64 {
        self.0
    }

    /// Enable/disable [`CycleCounter`] register.
    pub fn set_cycle_enabled(&mut self, enabled: bool) {
        match enabled {
            true => self.0 |= 1 << 0,
            false => self.0 &= !(1 << 0),
        };
        self.write();
    }

    /// Enable/disable [`Time`] register.
    pub fn set_time_enabled(&mut self, enabled: bool) {
        match enabled {
            true => self.0 |= 1 << 1,
            false => self.0 &= !(1 << 1),
        };
        self.write();
    }

    /// Enable/disable [`InstructionRetiredCounter`] register.
    pub fn set_instret_enabled(&mut self, enabled: bool) {
        match enabled {
            true => self.0 |= 1 << 2,
            false => self.0 &= !(1 << 2),
        };
        self.write();
    }
}

impl Display for CounterEnable {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:#018x}", self.0)
    }
}

/// Supervisor time-compare register
///
/// #See
/// Section `1.1. Supervisor Timer Register (stimecmp)` of `RISC-V "stimecmp / vstimecmp" Extension`
#[derive(Debug)]
pub struct TimeCompare(u64);

impl TimeCompare {
    /// Create new, initialized `time`.
    pub fn new() -> Self {
        let mut reg = TimeCompare(0);
        reg.read();
        return reg;
    }

    /// Update value of [`TimeCompare`] Register based on underlying `scounteren` register.
    pub fn read(&mut self) {
        let mut x: u64;
        unsafe {
            asm!(
                "csrr {x}, stimecmp",
                x = out(reg) x,
            );
        }
        self.0 = x;
    }

    /// Write value of [`TimeCompare`] Register back to underlying `stimecmp` register.
    pub fn write(&self) {
        let x: u64 = self.0;
        unsafe {
            asm!(
                "csrw stimecmp, {x}",
                x = in(reg) x,
            );
        }
    }

    /// Set `stimecmp` register.
    pub fn set(&mut self, value: u64) {
        self.0 = value
    }

    /// Get `stimecmp` register.
    pub fn get(&mut self) -> u64 {
        self.0
    }
}

impl Display for TimeCompare {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:#018x}", self.0)
    }
}
