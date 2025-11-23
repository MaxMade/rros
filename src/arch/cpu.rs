//! Convienient helper to access/modify CPU state.

use core::arch::asm;
use core::fmt::Display;
use core::ops::{Deref, DerefMut};

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

    /// Check if [`Cycle`](crate::arch::cycle::Cycle) register is enabled.
    pub fn is_cycle_enabled(&self) -> bool {
        (self.0 & (1 << 0)) != 0
    }

    /// Check if [`Time`](crate::arch::time::Time) register is enabled.
    pub fn is_time_enabled(&self) -> bool {
        (self.0 & (1 << 1)) != 0
    }

    /// Check if [`InstRet`](crate::arch::inst_ret::InstRet) register is enabled.
    pub fn is_instret_enabled(&self) -> bool {
        (self.0 & (1 << 2)) != 0
    }

    /// Get raw inner value.
    pub const fn raw(self) -> u64 {
        self.0
    }

    /// Enable/disable [`Cycle`](crate::arch::cycle::Cycle) register.
    pub fn set_cycle_enabled(&mut self, enabled: bool) {
        match enabled {
            true => self.0 |= 1 << 0,
            false => self.0 &= !(1 << 0),
        };
        self.write();
    }

    /// Enable/disable [`Time`](crate::arch::time::Time) register.
    pub fn set_time_enabled(&mut self, enabled: bool) {
        match enabled {
            true => self.0 |= 1 << 1,
            false => self.0 &= !(1 << 1),
        };
        self.write();
    }

    /// Enable/disable [`InstRet`](crate::arch::inst_ret::InstRet) register.
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

    /// Update value of [`TimeCompare`](crate::arch::time::Time) Register based on underlying `scounteren` register.
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
