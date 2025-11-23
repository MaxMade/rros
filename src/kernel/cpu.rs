//! Convienient helper to access/modify CPU state.

use core::arch::asm;
use core::fmt::Display;

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
    // Create zeroed abstraction of `tp` register.
    pub fn new(value: u64) -> Self {
        Self { 0: value }
    }

    /// Load current value from `tp` register.
    pub fn read(&mut self) {
        let mut x: u64 = 0;
        unsafe {
            asm!(
                "mv {x}, tp",
                x = out(reg) _,
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

/// Current operating status of hart.
///
/// #See
/// Section `4.1.1 Supervisor Status Register (sstatus)` of `Volume II: RISC-V Privileged Architectures`
#[derive(Debug)]
pub struct SStatus(u64);

impl SStatus {
    /// Create new, initialized `SStatus`.
    pub fn new() -> Self {
        let mut reg = SStatus(0);
        reg.read();
        return reg;
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

