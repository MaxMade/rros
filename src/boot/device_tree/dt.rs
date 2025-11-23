//! Abstraction of a device tree.

use core::cell::UnsafeCell;
use core::mem::MaybeUninit;

use crate::boot::device_tree::parser::Parser;
use crate::sync::level::LevelInitialization;

const INITIALIZED: UnsafeCell<bool> = UnsafeCell::new(false);
const DEVICE_TREE: UnsafeCell<MaybeUninit<DeviceTree>> = UnsafeCell::new(MaybeUninit::uninit());

#[derive(Debug)]
pub struct DeviceTree {
    parser: Parser,
}

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
        let parser = match Parser::new(dtb_ptr) {
            Ok(parser) => parser,
            Err(err) => {
                panic!("Unable to process device tree blob: {}", err);
            }
        };

        // # Safety
        // During the initialization phase (as indicated by `token`), no concurrent access is possible.
        DEVICE_TREE
            .get()
            .as_mut()
            .unwrap()
            .write(DeviceTree { parser });

        // # Safety
        // During the initialization phase (as indicated by `token`), no concurrent access is possible.
        return (DEVICE_TREE.get().as_ref().unwrap().assume_init_ref(), token);
    }

    /// Get an reference to the global device tree instance.
    pub fn get_dt() -> &'static Self {
        // # Safety
        //
        // Two cases can be observed:
        // - During the initialization phase, no concurrent access is possible. Therefore, either
        // write-access (using `initialization`) or read-access (every other method) is permitted.
        //
        // - After the initialization, only read-access (using every method except from
        // `initialize`) is permitted.
        unsafe { DEVICE_TREE.get().as_ref().unwrap().assume_init_ref() }
    }

    /// Get the number of enumerated CPUs within the device tree.
    pub fn get_cpu_count(&self) -> usize {
        self.parser
            .node_iter()
            .filter(|node| node.name().starts_with("cpu@"))
            .count()
    }
}
