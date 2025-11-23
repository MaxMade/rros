//! Convienient helper to access/modify CPU state.

use core::arch::asm;
use core::fmt::Display;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};

use crate::kernel::cpu_map::LogicalCPUID;
use crate::sync::level::{Level, LevelPrologue};

/// Get default page size (`4096` bytes)
pub const fn page_size() -> usize {
    4096
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

/// Abstraction of hard ID.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct HartID(u64);

impl HartID {
    /// Create HartID from raw value.
    pub const fn new(value: u64) -> Self {
        Self { 0: value }
    }

    /// Get raw inner value.
    pub const fn raw(self) -> u64 {
        self.0
    }
}

impl Display for HartID {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:#018x}", self.0)
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

/// Get `current` [`LogicalCPUID`] from [`TP`] register.
pub fn current() -> LogicalCPUID {
    let mut tp = TP::new(0);
    tp.read();
    let raw_logical_id = tp.raw();
    LogicalCPUID::new(usize::try_from(raw_logical_id).unwrap())
}

/// Current operating status of hart.
///
/// #See
/// Section `4.1.1 Supervisor Status Register (sstatus)` of `Volume II: RISC-V Privileged Architectures`
#[derive(Debug)]
pub struct SStatus(u64);

impl SStatus {
    /// Create `STVal` from raw value.
    pub const fn new(value: u64) -> Self {
        Self { 0: value }
    }

    /// Update value of `SStatus` based on underlying  `sstatus` register.
    pub fn read(&mut self) {
        let mut x: u64;
        unsafe {
            asm!(
                "csrr {x}, sstatus",
                x = out(reg) x,
            );
        }
        self.0 = x;
    }

    /// Update `sstatus` register based on value of `SStatus`.
    pub fn write(&self) {
        let x: u64 = self.0;
        unsafe {
            asm!(
                "csrw sstatus, {x}",
                x = in(reg) x,
            );
        }
    }

    /// Get raw inner value.
    pub const fn raw(self) -> u64 {
        self.0
    }

    /// Get `Global Interrupt-Enable Bit` for `S-Mode` (`SIE`)
    pub fn get_sie(&self) -> bool {
        (self.0 & (0b1 << 1)) != 0
    }

    /// Set `Global Interrupt-Enable Bit` for `S-Mode` (`SIE`)
    ///
    /// # Examples
    /// ```
    /// // Disable SIE
    /// let mut sstatus = SStatus::new();
    /// sstatus.set_sie(false);
    /// sstatus.write();
    /// ```
    pub fn set_sie(&mut self, value: bool) {
        self.0 &= !(0b1 << 1);
        if value {
            self.0 |= 0b1 << 1;
        }
    }

    /// Get `Global Preserved Interrupt-Enable Bit` for `S-Mode` (`SPIE`)
    pub fn get_spie(&self) -> bool {
        ((self.0 >> 5) & (0b1 << 1)) != 0
    }

    /// Set `Global Preserved Interrupt-Enable Bit` for `S-Mode` (`SPIE`)
    pub fn set_spie(&mut self, value: bool) {
        self.0 &= !(0b1 << 5);
        if value {
            self.0 |= 0b1 << 5;
        }
    }

    /// Get `Big-Endian Enable Bit` for `U-Mode` (`UBE`)
    pub fn get_ube(&self) -> SStatusEndianness {
        match (self.0 >> 6) & 0b1 {
            0 => SStatusEndianness::LittleEndian,
            1 => SStatusEndianness::LittleEndian,
            _ => panic!(),
        }
    }

    /// Set `Big-Endian Enable Bit` for `U-Mode` (`UBE`)
    pub fn set_ube(&mut self, value: SStatusEndianness) {
        self.0 &= !(0b1 << 6);
        self.0 |= ((value as u64) & 0b1) << 6;
    }

    /// Get `Global Preserved Privilege Level` for `S-Mode` (`SPP`)
    pub fn get_spp(&self) -> SStatusPrivLevel {
        match (self.0 >> 8) & 0b1 {
            0 => SStatusPrivLevel::UserMode,
            1 => SStatusPrivLevel::SupervisorMode,
            _ => panic!(),
        }
    }

    /// Set `Global Preserved Interrupt-Enable Bit` for `S-Mode` (`SPP`)
    pub fn set_spp(&mut self, value: SStatusPrivLevel) {
        self.0 &= !(0b1 << 8);
        self.0 |= ((value as u64) & 0b1) << 8;
    }

    /// Get `Vector Unit Extension Status`.
    pub fn get_vs(&self) -> SStatusUnitStatus {
        match (self.0 >> 9) & 0b11 {
            0b00 => SStatusUnitStatus::Off,
            0b01 => SStatusUnitStatus::Initial,
            0b10 => SStatusUnitStatus::Clean,
            0b11 => SStatusUnitStatus::Dirty,
            _ => panic!(),
        }
    }

    /// Get `Floating-Point Unit Extension Status`.
    pub fn get_fs(&self) -> SStatusUnitStatus {
        match (self.0 >> 13) & 0b11 {
            0b00 => SStatusUnitStatus::Off,
            0b01 => SStatusUnitStatus::Initial,
            0b10 => SStatusUnitStatus::Clean,
            0b11 => SStatusUnitStatus::Dirty,
            _ => panic!(),
        }
    }

    /// Get `Addtional User-Mode Unit Extension Status`.
    pub fn get_xs(&self) -> SStatusUnitStatus {
        match (self.0 >> 15) & 0b11 {
            0b00 => SStatusUnitStatus::Off,
            0b01 => SStatusUnitStatus::Initial,
            0b10 => SStatusUnitStatus::Clean,
            0b11 => SStatusUnitStatus::Dirty,
            _ => panic!(),
        }
    }

    /// Get `Supervisor User Memory Access Bit` (`SUM`).
    pub fn get_sum(&self) -> bool {
        ((self.0 >> 18) & 0b1) != 0
    }

    /// Set `Supervisor User Memory Access Bit` (`SUM`).
    pub fn set_sum(&mut self, value: bool) {
        self.0 &= !(0b1 << 18);
        if value {
            self.0 |= 0b1 << 18;
        }
    }

    /// Get `Make Executable Readable Bit` (`MXR`).
    pub fn get_mxr(&self) -> bool {
        ((self.0 >> 19) & 0b1) != 0
    }

    /// Set `Make Executable Readable Bit` (`MXR`).
    pub fn set_mxr(&mut self, value: bool) {
        self.0 &= !(0b1 << 19);
        if value {
            self.0 |= 0b1 << 19;
        }
    }

    /// Get `XLEN Configure` (`UXL`) for `U-Mode`.
    pub fn get_uxl(&self) -> u64 {
        (self.0 >> 32) & 0b11
    }

    /// Set `XLEN Configure` (`UXL`) for `U-Mode`.
    pub fn set_uxl(&mut self, value: u64) {
        self.0 &= !(0b11 << 32);
        self.0 |= (value & 0b11) << 32;
    }

    /// Get `Unit Extension Dirty Status`.
    pub fn get_sd(&self) -> bool {
        ((self.0 >> 63) & 0b1) != 0
    }
}

impl Display for SStatus {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:#018x}", self.0)
    }
}

/// Endiannes of `UBE` bits in `sstatus`.
///
/// #See
/// Section `4.1.1.3 Endianness Control in sstatus Register` of `Volume II: RISC-V Privileged Architectures`
#[derive(Debug, PartialEq, Eq)]
pub enum SStatusEndianness {
    /// Little endian.
    LittleEndian = 0b0,
    /// Big endian.
    BigEndian = 0b1,
}

impl Display for SStatusEndianness {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SStatusEndianness::LittleEndian => write!(f, "big-endian"),
            SStatusEndianness::BigEndian => write!(f, "little-endian"),
        }
    }
}

/// Privilege Level of `SPP` bit in `sstatus`.
///
/// #See
/// Section `8.6.2 Trap Entry` of `Volume II: RISC-V Privileged Architectures`
#[derive(Debug, PartialEq, Eq)]
pub enum SStatusPrivLevel {
    /// Trap was taken from user mode.
    UserMode = 0b0,
    /// Trap was taken from supervisor mode.
    SupervisorMode = 0b1,
}

impl Display for SStatusPrivLevel {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SStatusPrivLevel::UserMode => write!(f, "user-mode"),
            SStatusPrivLevel::SupervisorMode => write!(f, "supervisor-mode"),
        }
    }
}

/// Unit state of `FS`, `VS` and `XS` bit(s) in `sstatus`.
///
/// #See
/// Section `3.1.6.6 Extension Context Status in sstatus Register` of `Volume II: RISC-V Privileged Architectures`
#[derive(Debug, PartialEq, Eq)]
pub enum SStatusUnitStatus {
    /// Offline state.
    Off = 0b00,
    /// Initial state.
    Initial = 0b01,
    /// Clean state.
    Clean = 0b10,
    /// Dirty state.
    Dirty = 0b11,
}

impl Display for SStatusUnitStatus {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SStatusUnitStatus::Off => write!(f, "off"),
            SStatusUnitStatus::Initial => write!(f, "initial"),
            SStatusUnitStatus::Clean => write!(f, "clean"),
            SStatusUnitStatus::Dirty => write!(f, "dirty"),
        }
    }
}

/// Trap-Vector Base-Address Register
///
/// #See
/// Section `4.1.2 Supervisor Trap Vector Base Address Register (stvec)` of `Volume II: RISC-V Privileged Architectures`
#[derive(Debug)]
pub struct STVec(u64);

impl STVec {
    /// Create new, initialized `SupervisorTrapVectorBaseAddressRegister `.
    pub fn new() -> Self {
        let mut reg = STVec(0);
        reg.read();
        return reg;
    }

    /// Update value of `SupervisorTrapVectorBaseAddressRegister` based on underlying `stvec` register.
    pub fn read(&mut self) {
        let mut x: u64;
        unsafe {
            asm!(
                "csrr {x}, stvec",
                x = out(reg) x,
            );
        }
        self.0 = x;
    }

    /// Update `stvec` register based on value of `SupervisorTrapVectorBaseAddressRegister`.
    pub fn write(&self) {
        let x: u64 = self.0;
        unsafe {
            asm!(
                "csrw stvec, {x}",
                x = in(reg) x,
            );
        }
    }

    /// Get `Mode`.
    pub fn get_mode(&self) -> STVecMode {
        match self.0 & 0b11 {
            0 => STVecMode::Direct,
            1 => STVecMode::Vectored,
            _ => panic!(),
        }
    }

    /// Set `Mode`.
    pub fn set_mode(&mut self, mode: STVecMode) {
        self.0 &= !(0b11);
        self.0 |= (mode as u64) & 0b11;
    }

    /// Get `Base`.
    pub fn get_base(&self) -> u64 {
        self.0 >> 2
    }

    /// Set `Base`.
    pub fn set_base(&mut self, base: u64) {
        self.0 &= 0b11;
        self.0 |= base << 2;
    }
}

impl Display for STVec {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:#018x}", self.0)
    }
}

/// Mode of vector table.
#[derive(Debug, Eq, PartialEq)]
pub enum STVecMode {
    /// All exceptions set `pc` to `BASE`.
    Direct = 0,
    /// Asynchronous interrupts set `pc` to `BASE+4×cause`.
    Vectored = 1,
}

impl Display for STVecMode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            STVecMode::Direct => write!(f, "Direct"),
            STVecMode::Vectored => write!(f, "Vectored"),
        }
    }
}

/// Fine-grained Interrupt Enable Register
///
/// #See
/// Section `4.1.3 Supervisor Interrupt Registers (sip and sie)` of `Volume II: RISC-V Privileged Architectures`
#[derive(Debug)]
pub struct SIE(u64);

impl SIE {
    /// Create new, initialized `SupervisorInterruptEnable`.
    pub fn new() -> Self {
        let mut reg = SIE(0);
        reg.read();
        return reg;
    }

    /// Update value of `SupervisorInterruptEnable` based on underlying  `sie` register.
    pub fn read(&mut self) {
        let mut x: u64;
        unsafe {
            asm!(
                "csrr {x}, sie",
                x = out(reg) x,
            );
        }
        self.0 = x;
    }

    /// Update `sie` register based on value of `SupervisorInterruptEnable`.
    pub fn write(&self) {
        let x: u64 = self.0;
        unsafe {
            asm!(
                "csrw sie, {x}",
                x = in(reg) x,
            );
        }
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

/// Mask all interrupts (in `sie` register).
pub fn mask_all_interrupts() {
    let mut sie = SIE::new();
    sie.disable_all_interrupts();
}

/// Unmask all interrupts (in `sie` register).
pub fn unmask_all_interrupts() {
    let mut sie = SIE::new();
    sie.enable_all_interrupts();
}

/// Enable supervisor-mode interrupts (in `sstatus register).
pub unsafe fn enable_interrupts() {
    let mut sstatus = SStatus::new(0);
    sstatus.read();
    sstatus.set_sie(true);
    sstatus.write();
}

/// Disable supervisor-mode interrupts (in `sstatus register).
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
