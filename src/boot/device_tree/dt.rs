//! Abstraction of a device tree.

use crate::boot::device_tree::parser::Parser;
use crate::sync::init_cell::InitCell;
use crate::sync::level::LevelInitialization;

static DEVICE_TREE: InitCell<DeviceTree> = InitCell::new();

#[derive(Debug)]
pub struct DeviceTree {
    parser: Parser,
}

unsafe impl Sync for DeviceTree {}
unsafe impl Send for DeviceTree {}

impl DeviceTree {
    /// Initialize device tree.
    ///
    /// # Safety
    /// Parsing the device tree blob requires raw memory accesses to `dtb_ptr` and internal
    /// pointers. If the `dtb_ptr` and the referenced blob are valid, this function can be
    /// considered safe.
    pub unsafe fn initialize(
        dtb_ptr: *const u8,
        token: LevelInitialization,
    ) -> (&'static Self, LevelInitialization) {
        // Parse device tree blob
        let parser = match Parser::new(dtb_ptr) {
            Ok(parser) => parser,
            Err(err) => {
                panic!("Unable to process device tree blob: {}", err);
            }
        };

        // Update DEVICE_TREE
        let (device_tree, token) = DEVICE_TREE.as_mut(token);
        *device_tree = DeviceTree { parser };

        // Finalize InitCell
        // # Safety
        // During the initialization phase (as indicated by `token`), no concurrent access is possible.
        let token = unsafe { DEVICE_TREE.finanlize(token) };

        return (DEVICE_TREE.as_ref(), token);
    }

    /// Get initialize device tree (parser).
    pub fn get_dt() -> &'static Self {
        DEVICE_TREE.as_ref()
    }

    /// Get the number of enumerated CPUs within the device tree.
    pub fn get_cpu_count(&self) -> usize {
        self.parser
            .node_iter()
            .filter(|node| node.name().starts_with("cpu@"))
            .count()
    }
}
