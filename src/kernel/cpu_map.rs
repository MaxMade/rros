//! CPU map

use core::fmt::Display;

use crate::boot::device_tree::dt::DeviceTree;
use crate::config;
use crate::kernel::cpu::HartID;
use crate::sync::init_cell::InitCell;
use crate::sync::level::LevelInitialization;

use super::sbi;

/// Logical CPU ID.
///
/// Logical CPU IDs implement another way to address hardware threads (aka. CPUs). Hereby, these
/// IDs are assigned sequentially, and thus must be in range `[0, MAX_CPU_NUM]`.
pub struct LogicalCPUID(usize);

impl LogicalCPUID {
    /// Create new `LogicalCPUID` from fixed integer value.
    pub const fn new(value: usize) -> Self {
        Self { 0: value }
    }

    /// Get raw inner value.
    pub const fn raw(self) -> usize {
        self.0
    }
}

impl Display for LogicalCPUID {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Lookup map between [`LogicalCPUID`]s and [`HartID`]s.
#[derive(Debug)]
pub struct CPUMap {
    idx: usize,
    map: [HartID; config::MAX_CPU_NUM],
}

static CPU_MAP: InitCell<CPUMap> = InitCell::new();

/// Initialize CPU map using device tree and SBI information.
///
/// # Panics
///
/// The internal CPU map is capable of managing at most [`config::MAX_CPU_NUM`] entries. If this
/// limit is exceeded, `initialize` will `panic`.
pub fn initialize(token: LevelInitialization) -> LevelInitialization {
    // Get maximum supported number of CPUs as indicated by device tree.
    let dt = DeviceTree::get_dt();
    let max_cpu_num = dt.get_cpu_count();
    if max_cpu_num > config::MAX_CPU_NUM {
        panic!("Maximum number of supported CPUs exceeded!");
    }

    // Initialzie CPU_MAP with sane defaults
    let (cpu_map, token) = CPU_MAP.as_mut(token);
    cpu_map.idx = 0;

    // Try to lookup associated hart ID using SBI.
    for i in 0..usize::MAX {
        /* Success: All CPUs were correctly identified and registered */
        if cpu_map.idx >= max_cpu_num {
            break;
        }

        /* Query state of hart */
        let hart_id = HartID::new(u64::try_from(i).unwrap());
        match sbi::status_hart(hart_id) {
            Ok(_) => {
                /* Identified hart: Update map */
                cpu_map.map[cpu_map.idx] = hart_id;

                /* Increment number of identified harts */
                cpu_map.idx += 1;
            }
            Err(error) => {
                panic!("Unable to query state of hart: {}", error);
            }
        }
    }

    // Signal finished initialization
    //
    // # Safety
    // Initialization is done at this point, thus it is safe to make CPU_MAP read-only.
    let token = unsafe { CPU_MAP.finanlize(token) };

    token
}

/// Lookup [`LogicalCPUID`] for corresponding [`HartID`].
///
/// # Panics
/// If no corresponding `HartID` is found, `panic` will be called.
pub fn lookup_logical_id(hart_id: HartID) -> LogicalCPUID {
    let cpu_map = CPU_MAP.as_ref();
    for (curr_logical_id, curr_hart_id) in cpu_map.map.iter().enumerate() {
        if *curr_hart_id == hart_id {
            return LogicalCPUID::new(curr_logical_id);
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
    let cpu_map = CPU_MAP.as_ref();
    if logical_id.0 >= cpu_map.idx {
        panic!(
            "Unable to lookup corresponding hart ID for logical ID {}",
            logical_id
        );
    }

    cpu_map.map[usize::try_from(logical_id.0).unwrap()]
}
