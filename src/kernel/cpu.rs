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
pub struct ThreadPointer(u64);

impl ThreadPointer {
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
pub struct SupervisorStatusRegister(u64);

impl SupervisorStatusRegister {
    /// Create new, initialized `SupervisorStatusRegister`.
    pub fn new() -> Self {
        let mut reg = SupervisorStatusRegister(0);
        reg.read();
        return reg;
    }

    /// Update value of `SupervisorStatusRegister` based on underlying  `sstatus` register.
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

    /// Update `sstatus` register based on value of `SupervisorStatusRegister`.
    pub fn write(&self) {
        let x: u64 = self.0;
        unsafe {
            asm!(
                "csrw sstatus, {x}",
                x = in(reg) x,
            );
        }
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
    /// let mut sstatus = SupervisorStatusRegister::new();
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
    pub fn get_ube(&self) -> SupervisorStatusRegisterEndiannes {
        match (self.0 >> 6) & 0b1 {
            0 => SupervisorStatusRegisterEndiannes::LittleEndian,
            1 => SupervisorStatusRegisterEndiannes::LittleEndian,
            _ => panic!(),
        }
    }

    /// Set `Big-Endian Enable Bit` for `U-Mode` (`UBE`)
    pub fn set_ube(&mut self, value: SupervisorStatusRegisterEndiannes) {
        self.0 &= !(0b1 << 6);
        self.0 |= ((value as u64) & 0b1) << 6;
    }

    /// Get `Global Preserved Privilege Level` for `S-Mode` (`SPP`)
    pub fn get_spp(&self) -> SupervisorStatusRegisterPrivilegeLevel {
        match (self.0 >> 8) & 0b1 {
            0 => SupervisorStatusRegisterPrivilegeLevel::UserMode,
            1 => SupervisorStatusRegisterPrivilegeLevel::SupervisorMode,
            _ => panic!(),
        }
    }

    /// Set `Global Preserved Interrupt-Enable Bit` for `S-Mode` (`SPP`)
    pub fn set_spp(&mut self, value: SupervisorStatusRegisterPrivilegeLevel) {
        self.0 &= !(0b1 << 8);
        self.0 |= ((value as u64) & 0b1) << 8;
    }

    /// Get `Vector Unit Extension Status`.
    pub fn get_vs(&self) -> SupervisorStatusRegisterUnitStatus {
        match (self.0 >> 9) & 0b11 {
            0b00 => SupervisorStatusRegisterUnitStatus::Off,
            0b01 => SupervisorStatusRegisterUnitStatus::Initial,
            0b10 => SupervisorStatusRegisterUnitStatus::Clean,
            0b11 => SupervisorStatusRegisterUnitStatus::Dirty,
            _ => panic!(),
        }
    }

    /// Get `Floating-Point Unit Extension Status`.
    pub fn get_fs(&self) -> SupervisorStatusRegisterUnitStatus {
        match (self.0 >> 13) & 0b11 {
            0b00 => SupervisorStatusRegisterUnitStatus::Off,
            0b01 => SupervisorStatusRegisterUnitStatus::Initial,
            0b10 => SupervisorStatusRegisterUnitStatus::Clean,
            0b11 => SupervisorStatusRegisterUnitStatus::Dirty,
            _ => panic!(),
        }
    }

    /// Get `Addtional User-Mode Unit Extension Status`.
    pub fn get_xs(&self) -> SupervisorStatusRegisterUnitStatus {
        match (self.0 >> 15) & 0b11 {
            0b00 => SupervisorStatusRegisterUnitStatus::Off,
            0b01 => SupervisorStatusRegisterUnitStatus::Initial,
            0b10 => SupervisorStatusRegisterUnitStatus::Clean,
            0b11 => SupervisorStatusRegisterUnitStatus::Dirty,
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

impl Display for SupervisorStatusRegister {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:#066x}", self.0)
    }
}

impl From<u64> for SupervisorStatusRegister {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<usize> for SupervisorStatusRegister {
    fn from(value: usize) -> Self {
        Self(value.try_into().unwrap())
    }
}

impl From<SupervisorStatusRegister> for u64 {
    fn from(value: SupervisorStatusRegister) -> Self {
        value.0
    }
}

impl From<SupervisorStatusRegister> for usize {
    fn from(value: SupervisorStatusRegister) -> Self {
        value.0.try_into().unwrap()
    }
}

/// Endiannes of `UBE` bits in `sstatus`.
///
/// #See
/// Section `4.1.1.3 Endianness Control in sstatus Register` of `Volume II: RISC-V Privileged Architectures`
#[derive(Debug, PartialEq, Eq)]
pub enum SupervisorStatusRegisterEndiannes {
    /// Little endian.
    LittleEndian = 0b0,
    /// Big endian.
    BigEndian = 0b1,
}

impl Display for SupervisorStatusRegisterEndiannes {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SupervisorStatusRegisterEndiannes::LittleEndian => write!(f, "big-endian"),
            SupervisorStatusRegisterEndiannes::BigEndian => write!(f, "little-endian"),
        }
    }
}

/// Privilege Level of `SPP` bit in `sstatus`.
///
/// #See
/// Section `8.6.2 Trap Entry` of `Volume II: RISC-V Privileged Architectures`
#[derive(Debug, PartialEq, Eq)]
pub enum SupervisorStatusRegisterPrivilegeLevel {
    /// Trap was taken from user mode.
    UserMode = 0b0,
    /// Trap was taken from supervisor mode.
    SupervisorMode = 0b1,
}

impl Display for SupervisorStatusRegisterPrivilegeLevel {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SupervisorStatusRegisterPrivilegeLevel::UserMode => write!(f, "user-mode"),
            SupervisorStatusRegisterPrivilegeLevel::SupervisorMode => write!(f, "supervisor-mode"),
        }
    }
}

/// Unit state of `FS`, `VS` and `XS` bit(s) in `sstatus`.
///
/// #See
/// Section `3.1.6.6 Extension Context Status in sstatus Register` of `Volume II: RISC-V Privileged Architectures`
#[derive(Debug, PartialEq, Eq)]
pub enum SupervisorStatusRegisterUnitStatus {
    /// Offline state.
    Off = 0b00,
    /// Initial state.
    Initial = 0b01,
    /// Clean state.
    Clean = 0b10,
    /// Dirty state.
    Dirty = 0b11,
}

impl Display for SupervisorStatusRegisterUnitStatus {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SupervisorStatusRegisterUnitStatus::Off => write!(f, "off"),
            SupervisorStatusRegisterUnitStatus::Initial => write!(f, "initial"),
            SupervisorStatusRegisterUnitStatus::Clean => write!(f, "clean"),
            SupervisorStatusRegisterUnitStatus::Dirty => write!(f, "dirty"),
        }
    }
}

