//! Abstraction of a device tree.

use core::ffi::c_void;

use crate::boot::device_tree::parser::Parser;
use crate::kernel::address::{Address, PhysicalAddress};
use crate::mm::mapping::KERNEL_VIRTUAL_MEMORY_SYSTEM;
use crate::sync::init_cell::InitCell;
use crate::sync::level::LevelInitialization;

use crate::boot::device_tree::node::Node;
use crate::boot::device_tree::property::PropertyValue;

static DEVICE_TREE: InitCell<DeviceTree> = InitCell::new();

/// Abstraction of a device tree.
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
        dtb_size: u32,
        token: LevelInitialization,
    ) -> (&'static Self, LevelInitialization) {
        // Map device tree
        let (virt_address, token) = KERNEL_VIRTUAL_MEMORY_SYSTEM
            .as_ref()
            .early_create_dev(
                PhysicalAddress::new(dtb_ptr as *mut c_void),
                dtb_size as usize,
                token,
            )
            .unwrap();

        // Parse device tree blob
        let parser = match Parser::new(virt_address.as_ptr() as *const u8) {
            Ok(parser) => parser,
            Err(err) => {
                panic!("Unable to process device tree blob: {}", err);
            }
        };

        // Update DEVICE_TREE
        let mut device_tree = DEVICE_TREE.get_mut(token);
        *device_tree = DeviceTree { parser };
        let token = device_tree.destroy();

        // Finalize InitCell
        // # Safety
        // During the initialization phase (as indicated by `token`), no concurrent access is possible.
        let token = unsafe { DEVICE_TREE.finanlize(token) };

        return (DEVICE_TREE.as_ref(), token);
    }

    /// Get initialize device tree (parser).
    pub fn get_dt(token: LevelInitialization) -> (&'static Self, LevelInitialization) {
        (DEVICE_TREE.as_ref(), token)
    }

    /// Get the number of enumerated CPUs within the device tree.
    pub fn get_cpu_count(&self) -> usize {
        self.parser
            .node_iter()
            .filter(|node| node.name().starts_with("cpu@"))
            .count()
    }

    /// Get node by matching `compatible` property
    pub fn get_node_by_compatible_property(&self, compatible: &str) -> Option<Node> {
        for node in self.parser.node_iter() {
            if let Some(property) = node.property_iter().find(|p| p.name == "compatible") {
                if let PropertyValue::String(value) = property.get_value() {
                    if value.contains(compatible) {
                        return Some(node);
                    }
                }
            }
        }

        return None;
    }
}
