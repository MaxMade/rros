//! Abstraction of the *S*upervisor *B*inary *I*nterface (*SBI*) -  interface between the
//! *S*upervisor *E*xecution *E*nvironment (*SEE*) and the supervisor.

use core::arch::asm;
use core::error::Error;
use core::fmt::Display;

use crate::kernel;
use crate::kernel::address::Address;
use crate::kernel::sbi::SBIFunctionID::BaseExtension;
use crate::kernel::sbi::SBIFunctionID::HartStateManagementExtension;

/// Perform `ECALL` for OpenSBI firmware without any arguments.
///
/// * `eid`: Extension ID.
/// * `fid`: Function ID.
fn sbi_ecall_0(eid: SBIExtensionID, fid: SBIFunctionID) -> Result<isize, SBIError> {
    /* Perform ecall */
    let mut error = 0;
    let mut value = 0;
    unsafe {
        asm!(
            "ecall",
            in("a7") isize::from(eid),
            in("a6") isize::from(fid),
            out("a0") error,
            out("a1") value,
        );
    }

    if error != 0 {
        return Err(SBIError::from(error));
    }

    return Ok(value);
}

/// Perform `ECALL` for OpenSBI firmware with a single argument.
///
/// * `eid`: Extension ID.
/// * `fid`: Function ID.
/// * `arg0`: First argument.
fn sbi_ecall_1(eid: SBIExtensionID, fid: SBIFunctionID, arg0: isize) -> Result<isize, SBIError> {
    /* Perform ecall */
    let mut error = arg0 as isize;
    let mut value = 0isize;
    unsafe {
        asm!(
            "ecall",
            inout("a0") error,
            in("a7") isize::from(eid),
            in("a6") isize::from(fid),
            out("a1") value,
        );
    }

    if error != 0 {
        return Err(SBIError::from(error));
    }

    return Ok(value as isize);
}

/// Perform `ECALL` for OpenSBI firmware with a three arguments.
///
/// * `eid`: Extension ID.
/// * `fid`: Function ID.
/// * `arg0`: First argument.
/// * `arg1`: Second argument.
/// * `arg2`: Third argument.
fn sbi_ecall_3(
    eid: SBIExtensionID,
    fid: SBIFunctionID,
    arg0: isize,
    arg1: isize,
    arg2: isize,
) -> Result<isize, SBIError> {
    /* Perform ecall */
    let mut error = arg0 as isize;
    let mut value = arg1 as isize;
    unsafe {
        asm!(
            "ecall",
            inout("a0") error,
            inout("a1") value,
            in("a2") arg2,
            in("a7") isize::from(eid),
            in("a6") isize::from(fid),
        );
    }

    if error != 0 {
        return Err(SBIError::from(error));
    }

    return Ok(value as isize);
}

/// SBI Errors
///
/// # See
/// Section `Binary Encoding` of `RISC-V Supervisor Binary Interface Specification`
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SBIError {
    Failed = -1,
    NotSupported = -2,
    InvalidParam = -3,
    Denied = -4,
    InvalidAddress = -5,
    AlreadyAvailable = -6,
    AlreadyStarted = -7,
    AlreadyStopped = -8,
}

impl Error for SBIError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }

    fn description(&self) -> &str {
        "description() is deprecated; use Display"
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.source()
    }
}

impl Display for SBIError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SBIError::Failed => write!(f, "SBI_ERR_FAILED"),
            SBIError::NotSupported => write!(f, "SBI_ERR_NOT_SUPPORTED"),
            SBIError::InvalidParam => write!(f, "SBI_ERR_INVALID_PARAM"),
            SBIError::Denied => write!(f, "SBI_ERR_DENIED"),
            SBIError::InvalidAddress => write!(f, "SBI_ERR_INVALID_ADDRESS"),
            SBIError::AlreadyAvailable => write!(f, "SBI_ERR_ALREADY_AVAILABLE"),
            SBIError::AlreadyStarted => write!(f, "SBI_ERR_ALREADY_STARTED"),
            SBIError::AlreadyStopped => write!(f, "SBI_ERR_ALREADY_STOPPED"),
        }
    }
}

impl From<isize> for SBIError {
    fn from(value: isize) -> Self {
        match value {
            x if x == Self::Failed as isize => Self::Failed,
            x if x == Self::NotSupported as isize => Self::NotSupported,
            x if x == Self::InvalidParam as isize => Self::InvalidParam,
            x if x == Self::Denied as isize => Self::Denied,
            x if x == Self::InvalidAddress as isize => Self::InvalidAddress,
            x if x == Self::AlreadyAvailable as isize => Self::AlreadyAvailable,
            x if x == Self::AlreadyStarted as isize => Self::AlreadyStarted,
            x if x == Self::AlreadyStopped as isize => Self::AlreadyStopped,
            _ => panic!("Unsupported SBI error: {:x}", value),
        }
    }
}

/// SBI Extension ID (`EID`)
///
/// # See
/// - Section `Chapter 3. Binary Encoding` of `RISC-V Supervisor Binary Interface Specification`
/// - Section `Chapter 4. Base Extension (EID #0x10)` of `RISC-V Supervisor Binary Interface Specification`
/// - Section `Chapter 9. Hart State Management Extension (EID #0x48534D "HSM")` of `RISC-V Supervisor Binary Interface Specification`
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SBIExtensionID {
    /// Functionality for probing availability/version of SBI extensions.
    BaseExtension = 0x10,
    /// Functionality for requesting hart state changes.
    HartStateManagement = 0x48534d,
}

impl Display for SBIExtensionID {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SBIExtensionID::BaseExtension => write!(f, "Base Extension"),
            SBIExtensionID::HartStateManagement => write!(f, "Hart State Management Extension"),
        }
    }
}

impl From<SBIExtensionID> for isize {
    fn from(value: SBIExtensionID) -> Self {
        value as isize
    }
}

/// SBI Function ID (`FID`)
///
/// # See
/// Section `Chapter 4. Base Extension (EID #0x10)` of `RISC-V Supervisor Binary Interface Specification`
#[derive(Debug, Copy, Clone)]
pub enum SBIFunctionID {
    BaseExtension(SBIBaseFunctionID),
    HartStateManagementExtension(SBIHSMFunctionID),
}

impl Display for SBIFunctionID {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SBIFunctionID::BaseExtension(id) => write!(f, "{}", id),
            SBIFunctionID::HartStateManagementExtension(id) => write!(f, "{}", id),
        }
    }
}

impl From<SBIFunctionID> for isize {
    fn from(value: SBIFunctionID) -> Self {
        match value {
            BaseExtension(extension) => isize::from(extension),
            HartStateManagementExtension(extension) => isize::from(extension),
        }
    }
}

/// SBI Function ID (`FID`) for Base Extension
///
/// # See
/// Section `Chapter 4. Base Extension (EID #0x10)` of `RISC-V Supervisor Binary Interface Specification`
#[derive(Debug, Copy, Clone)]
pub enum SBIBaseFunctionID {
    /// SBI specification version.
    SpecificationVersion = 0x00,
    /// SBI probe extension.
    ProbeExtension = 0x03,
}

impl Display for SBIBaseFunctionID {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SBIBaseFunctionID::SpecificationVersion => write!(f, "SBI Specification Version"),
            SBIBaseFunctionID::ProbeExtension => write!(f, "SBI Probe Extension"),
        }
    }
}

impl From<SBIBaseFunctionID> for isize {
    fn from(value: SBIBaseFunctionID) -> Self {
        value as isize
    }
}

/// SBI Function ID (`FID`) for Hart State Management Extension
///
/// # See
/// Section `Chapter 9. Hart State Management Extension (EID #0x48534D "HSM")` of `RISC-V Supervisor Binary Interface Specification`
#[derive(Debug, Copy, Clone)]
pub enum SBIHSMFunctionID {
    /// Request the SBI implementation to start executing the target hart in supervisor-mode.
    HartStart = 0x00,
    /// Get the current status of the given hart.
    HartStatus = 0x02,
}

impl Display for SBIHSMFunctionID {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SBIHSMFunctionID::HartStart => write!(f, "HART Start"),
            SBIHSMFunctionID::HartStatus => write!(f, "HART Status"),
        }
    }
}

impl From<SBIHSMFunctionID> for isize {
    fn from(value: SBIHSMFunctionID) -> Self {
        value as isize
    }
}

/// SBI Minor Number.
#[derive(Debug)]
pub struct SBIMinorNumber(u32);

impl From<SBIMinorNumber> for u32 {
    fn from(value: SBIMinorNumber) -> Self {
        value.0
    }
}

impl Display for SBIMinorNumber {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:x}", self.0)
    }
}

/// SBI Major Number.
#[derive(Debug)]
pub struct SBIMajorNumber(u32);

impl From<SBIMajorNumber> for u32 {
    fn from(value: SBIMajorNumber) -> Self {
        value.0
    }
}

impl Display for SBIMajorNumber {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:x}", self.0)
    }
}

/// Query version of OpenSBI.
pub fn specification_version() -> Result<(SBIMajorNumber, SBIMinorNumber), SBIError> {
    match sbi_ecall_0(
        SBIExtensionID::BaseExtension,
        SBIFunctionID::BaseExtension(SBIBaseFunctionID::SpecificationVersion),
    ) {
        Ok(result) => {
            let minor = (result as u32) & 0xffffff;
            let major = ((result as u32) >> 24) & 0x7f;
            return Ok((SBIMajorNumber(major), SBIMinorNumber(minor)));
        }
        Err(err) => {
            return Err(err);
        }
    };
}

/// Probe availability of extension for given SBI extension.
pub fn probe_extension(eid: SBIExtensionID) -> Result<bool, SBIError> {
    match sbi_ecall_1(
        SBIExtensionID::BaseExtension,
        SBIFunctionID::BaseExtension(SBIBaseFunctionID::ProbeExtension),
        isize::from(eid),
    ) {
        Ok(value) => return Ok(value == 1),
        Err(error) => return Err(error),
    };
}

/// Execution state of hart.
///
/// # See
/// Section `Chapter 9. Hart State Management Extension (EID #0x48534D "HSM")` of `RISC-V Supervisor Binary Interface Specification`
#[derive(Debug)]
pub enum SBIHartState {
    ///The hart is physically powered-up and executing normally.
    Started = 0,
    // The hart is not executing in supervisor-mode or any lower privilege mode.
    Stopped = 1,
    /// Some other hart has requested to start (or power-up) the hart from the STOPPED state.
    StartPending = 2,
    /// The hart has requested to stop (or power-down) itself from the STARTED state and the SBI implementation is still working to get the hart in the STOPPED state.
    StopPending = 3,
    /// This hart is in a platform specific suspend (or low power) state.
    Suspended = 4,
    /// The hart has requested to put itself in a platform specific low power state from the STARTED state.
    Suspendpending = 5,
    /// An interrupt or platform specific hardware event has caused the hart to resume normal execution from the SUSPENDED state.
    ResumePending = 6,
}

impl Display for SBIHartState {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SBIHartState::Started => write!(f, "Started"),
            SBIHartState::Stopped => write!(f, "Stopped"),
            SBIHartState::StartPending => write!(f, "Start pending"),
            SBIHartState::StopPending => write!(f, "Stop pending"),
            SBIHartState::Suspended => write!(f, "Suspended"),
            SBIHartState::Suspendpending => write!(f, "Suspend pending"),
            SBIHartState::ResumePending => write!(f, "Resume Pending"),
        }
    }
}

impl From<SBIHartState> for isize {
    fn from(value: SBIHartState) -> Self {
        value as isize
    }
}

impl From<isize> for SBIHartState {
    fn from(value: isize) -> Self {
        match value {
            x if x == Self::Started as isize => Self::Started,
            x if x == Self::Stopped as isize => Self::Stopped,
            x if x == Self::StartPending as isize => Self::StartPending,
            x if x == Self::StopPending as isize => Self::StopPending,
            x if x == Self::Suspended as isize => Self::Suspended,
            x if x == Self::Suspendpending as isize => Self::Suspendpending,
            x if x == Self::ResumePending as isize => Self::ResumePending,
            _ => panic!("Unable to convert value to SBHHartState"),
        }
    }
}

/// Request the SBI implementation to start executing the target hart in supervisor-mode at address specified by `start_addr` parameter.
///
/// * `hart_id`: Target hart ID.
/// * `start_addr`: Start address
/// * `arg`: Opaque arguemnt
///
/// # Initial Register State
/// | Register Name | Register Value |
/// +:-------------:|:--------------:|
/// | `SATP`        | 0              |
/// | `SSTATUS.SIE` | 0              |
/// | `a0`          | `hard_id`      |
/// | `a1`          | `arg`          |
pub fn start_hart(
    hart_id: kernel::cpu::HartID,
    start_addr: kernel::address::PhysicalAddress<unsafe extern "C" fn(isize, isize)>,
    arg: isize,
) -> Result<(), SBIError> {
    let hart_id: u64 = hart_id.into();
    match sbi_ecall_3(
        SBIExtensionID::HartStateManagement,
        SBIFunctionID::HartStateManagementExtension(SBIHSMFunctionID::HartStart),
        hart_id as isize,
        start_addr.addr() as isize,
        arg,
    ) {
        Ok(_) => {
            return Ok(());
        }

        Err(err) => {
            return Err(err);
        }
    };
}

/// Get the current hart status.
///
/// * `hart_id`: Target hart ID.
pub fn status_hart(hart_id: kernel::cpu::HartID) -> Result<SBIHartState, SBIError> {
    let hart_id: u64 = hart_id.into();
    match sbi_ecall_1(
        SBIExtensionID::HartStateManagement,
        SBIFunctionID::HartStateManagementExtension(SBIHSMFunctionID::HartStatus),
        isize::try_from(hart_id).unwrap(),
    ) {
        Ok(status) => {
            return Ok(SBIHartState::from(status));
        }

        Err(err) => {
            return Err(err);
        }
    };
}
