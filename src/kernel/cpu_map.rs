//! CPU map

use core::cell::UnsafeCell;
use core::fmt::Display;

use crate::config;
use crate::kernel::cpu::HartID;
use crate::sync::level::LevelInitialization;

/// Logical CPU ID.
///
/// Logical CPU IDs implement another way to address hardware threads (aka. CPUs). Hereby, these
/// IDs are assigned sequentially, and thus must be in range `[0, MAX_CPU_NUM]`.
pub struct LogicalCPUID(u64);

impl LogicalCPUID {
    /// Create new `LogicalCPUID` from fixed integer value.
    pub fn new(value: u64) -> Self {
        Self { 0: value }
    }
}

impl Display for LogicalCPUID {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

const CPU_MAP_IDX: UnsafeCell<usize> = UnsafeCell::new(0);
const CPU_MAP: UnsafeCell<[HartID; config::MAX_CPU_NUM]> =
    UnsafeCell::new([HartID::new(0); config::MAX_CPU_NUM]);

/// Register hart at CPU map.
///
/// # Panics
///
/// The internal CPU map is capable of managing at most [`config::MAX_CPU_NUM`] entries. If this
/// limit is exceeded, `register_hart` will `panic`.
pub fn register_hart(
    hart_id: HartID,
    token: LevelInitialization,
) -> (LogicalCPUID, LevelInitialization) {
    // Fetch CPU map index
    //
    // # Safety
    // During the initialization phase (as indicated by `token`), no concurrent access is possible.
    let cpu_map_idx = unsafe { CPU_MAP_IDX.get().as_mut().unwrap() };

    // Check if maximum number of supported harts is reached
    if *cpu_map_idx >= config::MAX_CPU_NUM {
        panic!("Unable to register hart: Maximum number of supported Logical IDs reached!");
    }

    // Update CPU map
    //
    // # Safety
    // During the initialization phase (as indicated by `token`), no concurrent access is possible.
    unsafe {
        let cpu_map = CPU_MAP.get().as_mut().unwrap();
        cpu_map[*cpu_map_idx] = hart_id;
    }

    // Fetch logical ID
    let logical_id = LogicalCPUID::new(u64::try_from(*cpu_map_idx).unwrap());

    // Update CPU map index
    *cpu_map_idx += 1;

    (logical_id, token)
}

/// Lookup [`LogicalCPUID`] for corresponding [`HartID`].
///
/// # Panics
/// If no corresponding `HartID` is found, `panic` will be called.
pub fn lookup_logical_id(hart_id: HartID) -> LogicalCPUID {
    // # Safety
    // Two cases can be observed:
    // - During the initialization phase, no concurrent access is possible. Therefore, either
    // write-access (using `register_hart`) or read-access (using `lookup_hart_id`/`lookup_logical_id`) is permitted.
    //
    // - After the initialization, only read-access (using `lookup_hart_id`/`lookup_logical_id`) is
    // permitted.
    let cpu_map = unsafe { CPU_MAP.get().as_ref().unwrap() };

    for (curr_logical_id, curr_hart_id) in cpu_map.iter().enumerate() {
        if *curr_hart_id == hart_id {
            return LogicalCPUID::new(u64::try_from(curr_logical_id).unwrap());
        }
    }

    panic!(
        "Unable to lookup corresponding logical ID for hart ID {}",
        hart_id
    );
}

/// Lookup [`HartID`] for corresponding [`LogicalCPUID`].
///
/// # Panics
/// If no corresponding `LogicalCPUID` is found, `panic` will be called.
pub fn lookup_hart_id(logical_id: LogicalCPUID) -> HartID {
    // # Safety
    // Two cases can be observed:
    // - During the initialization phase, no concurrent access is possible. Therefore, either
    // write-access (using `register_hart`) or read-access (using `lookup_hart_id`/`lookup_logical_id`) is permitted.
    //
    // - After the initialization, only read-access (using `lookup_hart_id`/`lookup_logical_id`) is
    // permitted.
    let cpu_map = unsafe { CPU_MAP.get().as_ref().unwrap() };
    let cpu_map_idx = unsafe { *CPU_MAP_IDX.get().as_ref().unwrap() };
    let logical_id = usize::try_from(logical_id.0).unwrap();

    if logical_id >= cpu_map_idx {
        panic!(
            "Unable to lookup corresponding hart ID for logical ID {}",
            logical_id
        );
    }

    cpu_map[usize::try_from(logical_id).unwrap()]
}
