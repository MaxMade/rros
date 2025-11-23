//! Kernel-Abstractions trap causes.

use core::fmt::Display;

use crate::arch::cpu::SCause;

/// Interrupt reasons.
///
/// For more details, see `Table 4.2` of `Volume II: RISC-V Privileged Architectures`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Interrupt {
    /// Supervisor software interrupt.
    SoftwareInterrupt,
    /// Supervisor timer interrupt.
    TimerInterrupt,
    /// Supervisor external interrupt.
    ExternalInterrupt,
    /// Supervisor generic interrupt.
    Interrupt(u64),
}

impl Into<usize> for Interrupt {
    fn into(self) -> usize {
        match self {
            Interrupt::SoftwareInterrupt => 1,
            Interrupt::TimerInterrupt => 5,
            Interrupt::ExternalInterrupt => 9,
            Interrupt::Interrupt(interrupt) => interrupt as usize,
        }
    }
}

impl From<usize> for Interrupt {
    fn from(value: usize) -> Self {
        match value {
            1 => Interrupt::SoftwareInterrupt,
            5 => Interrupt::TimerInterrupt,
            9 => Interrupt::ExternalInterrupt,
            interrupt => Interrupt::Interrupt(interrupt as u64),
        }
    }
}

impl Display for Interrupt {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Interrupt::SoftwareInterrupt => write!(f, "Supervisor software interrupt"),
            Interrupt::TimerInterrupt => write!(f, "Supervisor timer interrupt"),
            Interrupt::ExternalInterrupt => write!(f, "Supervisor external interrupt"),
            Interrupt::Interrupt(interrupt) => write!(f, "Supervisor Interrupt {:x}", interrupt),
        }
    }
}

/// Exception reasons.
///
/// For more details, see `Table 4.2` of `Volume II: RISC-V Privileged Architectures`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Exception {
    /// Instruction address misaligned.
    InstructionMisalignedAddr,
    /// Instruction access fault.
    InstructionAccessFault,
    /// Illegal instruction.
    IllegalInstruction,
    /// Breakpoint.
    Breakpoint,
    /// Load address misaligned.
    LoadMisalignedAddr,
    /// Load access fault.
    LoadAccessFault,
    /// Store/AMO address misaligned.
    StoreMisalignedAddr,
    /// Store/AMO access fault.
    StoreAccessFault,
    /// Environment call from U-mode.
    EnvCallUser,
    /// Environment call from S-mode.
    EnvCallSupervisor,
    /// Instruction page fault
    InstructionPageFault,
    /// Load page fault.
    LoadPageFault,
    /// Store page fault.
    StorePageFault,
    /// Generic exception.
    Exception(u64),
}

impl Into<usize> for Exception {
    fn into(self) -> usize {
        match self {
            Exception::InstructionMisalignedAddr => 0,
            Exception::InstructionAccessFault => 1,
            Exception::IllegalInstruction => 2,
            Exception::Breakpoint => 3,
            Exception::LoadMisalignedAddr => 4,
            Exception::LoadAccessFault => 5,
            Exception::StoreMisalignedAddr => 6,
            Exception::StoreAccessFault => 7,
            Exception::EnvCallUser => 8,
            Exception::EnvCallSupervisor => 9,
            Exception::InstructionPageFault => 12,
            Exception::LoadPageFault => 13,
            Exception::StorePageFault => 15,
            Exception::Exception(exception) => exception as usize,
        }
    }
}

impl From<usize> for Exception {
    fn from(value: usize) -> Self {
        match value {
            0 => Exception::InstructionMisalignedAddr,
            1 => Exception::InstructionAccessFault,
            2 => Exception::IllegalInstruction,
            3 => Exception::Breakpoint,
            4 => Exception::LoadMisalignedAddr,
            5 => Exception::LoadAccessFault,
            6 => Exception::StoreMisalignedAddr,
            7 => Exception::StoreAccessFault,
            8 => Exception::EnvCallUser,
            9 => Exception::EnvCallSupervisor,
            12 => Exception::InstructionPageFault,
            13 => Exception::LoadPageFault,
            15 => Exception::StorePageFault,
            exception => Exception::Exception(exception as u64),
        }
    }
}

impl Display for Exception {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Exception::InstructionMisalignedAddr => write!(f, "Instruction address misaligned"),
            Exception::InstructionAccessFault => write!(f, "Instruction access fault"),
            Exception::IllegalInstruction => write!(f, "Illegal instruction"),
            Exception::Breakpoint => write!(f, "Breakpoint"),
            Exception::LoadMisalignedAddr => write!(f, "Load address misasligned"),
            Exception::LoadAccessFault => write!(f, "Load access fault"),
            Exception::StoreMisalignedAddr => write!(f, "Store/AMO address misaligned"),
            Exception::StoreAccessFault => write!(f, "Store/AMO access fault"),
            Exception::EnvCallUser => write!(f, "Environment call from U-mode"),
            Exception::EnvCallSupervisor => write!(f, "Environment call from S-mode"),
            Exception::InstructionPageFault => write!(f, "Instruction page fault"),
            Exception::LoadPageFault => write!(f, "Load page fault"),
            Exception::StorePageFault => write!(f, "Store page fault"),
            Exception::Exception(exception) => write!(f, "Exception {:x}", exception),
        }
    }
}

/// Trap reasons.
///
/// For more details, see `Table 4.2` of `Volume II: RISC-V Privileged Architectures`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trap {
    /// [`Interrupt`] source.
    Interrupt(Interrupt),
    /// [`Exception`] source.
    Exception(Exception),
}

impl Trap {
    /// Check if pending trap is an [`Interrupt`]
    pub const fn is_interrupt(&self) -> bool {
        match self {
            Trap::Interrupt(_) => true,
            Trap::Exception(_) => false,
        }
    }

    /// Check if pending trap is an [`Exception`]
    pub const fn is_exception(&self) -> bool {
        match self {
            Trap::Interrupt(_) => false,
            Trap::Exception(_) => true,
        }
    }
}

impl Display for Trap {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Trap::Interrupt(interrupt) => write!(f, "{}", interrupt),
            Trap::Exception(exception) => write!(f, "{}", exception),
        }
    }
}

impl From<SCause> for Trap {
    fn from(value: SCause) -> Self {
        const INTERRUPT_MASK: u64 = 1u64 << 63;
        let is_interrupt = (value.raw() & INTERRUPT_MASK) != 0;

        if is_interrupt {
            let trap = match value.raw() & !INTERRUPT_MASK {
                1 => Trap::Interrupt(Interrupt::SoftwareInterrupt),
                5 => Trap::Interrupt(Interrupt::TimerInterrupt),
                9 => Trap::Interrupt(Interrupt::ExternalInterrupt),
                interrupt => Trap::Interrupt(Interrupt::Interrupt(interrupt)),
            };
            return trap;
        } else {
            let trap = match value.raw() & !INTERRUPT_MASK {
                0 => Trap::Exception(Exception::InstructionMisalignedAddr),
                1 => Trap::Exception(Exception::InstructionAccessFault),
                2 => Trap::Exception(Exception::IllegalInstruction),
                3 => Trap::Exception(Exception::Breakpoint),
                4 => Trap::Exception(Exception::LoadMisalignedAddr),
                5 => Trap::Exception(Exception::LoadAccessFault),
                6 => Trap::Exception(Exception::StoreMisalignedAddr),
                7 => Trap::Exception(Exception::StoreAccessFault),
                8 => Trap::Exception(Exception::EnvCallUser),
                9 => Trap::Exception(Exception::EnvCallSupervisor),
                12 => Trap::Exception(Exception::InstructionPageFault),
                13 => Trap::Exception(Exception::LoadPageFault),
                15 => Trap::Exception(Exception::StorePageFault),
                exception => Trap::Exception(Exception::Exception(exception)),
            };
            return trap;
        }
    }
}
