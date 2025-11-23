use crate::boot::device_tree::header;
use crate::boot::device_tree::memory_reservation_block;
use crate::boot::device_tree::node;
use crate::boot::device_tree::property::PropertyValue;
use crate::boot::device_tree::structure_block;

use core::fmt::Display;

use core::mem;
use core::ptr;
use core::slice;
use core::str;

/// Flattened Devicetree Parser.
#[derive(Debug)]
pub struct Parser {
    /// Pointer to flattened device tree blob.
    dtb_ptr: *const u8,
    /// Size of flattened device tree blob in bytes.
    dtb_size: usize,

    /// Pointer to flattened device tree header.
    header: ptr::NonNull<header::FDTHeader>,
}

impl Parser {
    /// Create a new parser from flattened device tree blob.
    ///
    /// * `dtb_ptr`: Pointer to flattened device tree blob.
    /// * `dtb_size`: Size of flattened device tree blob in bytes.
    pub unsafe fn new(dtb_ptr: *const u8) -> Result<Self, ParserError> {
        /* Check required alignmnet */
        if dtb_ptr.align_offset(8) != 0 {
            return Err(ParserError::UnalignedAccess);
        }

        /* Process FDTHeader */
        let raw_fdt_header: *const header::FDTHeader = dtb_ptr.cast();
        let header = ptr::NonNull::new(raw_fdt_header.cast_mut())
            .expect("The devicetree blob pointer must not be NULL!");
        let dtb_size = header.as_ref().totalsize() as usize;
        if header.as_ref().magic() != header::FDT_HEADER_MAGIC {
            return Err(ParserError::InvalidMagicValue);
        }
        if header.as_ref().version() != header::FDT_HEADER_SUPPORTED_VERSION {
            return Err(ParserError::InvalidMagicValue);
        }

        return Ok(Self {
            dtb_ptr,
            dtb_size,
            header,
        });
    }

    /// Get magic value.
    pub fn magic(&self) -> u32 {
        assert!(self.check_access_dtb(self.header.as_ptr()));
        unsafe { self.header.as_ref().magic() }
    }

    /// Get total size in bytes of the devicetree data structure.
    pub fn totalsize(&self) -> u32 {
        assert!(self.check_access_dtb(self.header.as_ptr()));
        unsafe { self.header.as_ref().totalsize() }
    }

    /// Get Version of the devicetree data structure from raw `FDTHeader`.
    pub fn version(&self) -> u32 {
        assert!(self.check_access_dtb(self.header.as_ptr()));
        unsafe { self.header.as_ref().version() }
    }

    /// Get Lowest version of the devicetree data structure with which the version used is backwards compatible from raw `FDTHeader`.
    pub fn last_comp_version(&self) -> u32 {
        assert!(self.check_access_dtb(self.header.as_ptr()));
        unsafe { self.header.as_ref().last_comp_version() }
    }

    /// Get Physical ID of the system’s boot CPU from raw `FDTHeader`.
    pub fn boot_cpuid_phys(&self) -> u32 {
        assert!(self.check_access_dtb(self.header.as_ptr()));
        unsafe { self.header.as_ref().boot_cpuid_phys() }
    }

    /// Return iterator for memory reservation entries.
    pub fn mem_reservation_iter(&self) -> memory_reservation_block::MemoryReservationIter {
        /* Get a pointer to the memory reservation block */
        assert!(self.check_access_dtb(self.header.as_ptr()));
        let mem_rsvmap_offset = unsafe { self.header.as_ref().off_mem_rsvmap() };
        let ptr: *const u64 = unsafe { self.dtb_ptr.add(mem_rsvmap_offset as usize).cast() };

        return memory_reservation_block::MemoryReservationIter { parser: self, ptr };
    }

    /// Get a iterator for each node in the structure block.
    ///
    /// Returns an iterator representing the visited node of a depth-first search of the structure
    /// block within the flattened devicetree.
    pub fn node_iter(&self) -> impl Iterator<Item = node::Node> {
        let struct_block_iter = self.structure_block_iter();
        return struct_block_iter
            .filter(|e| match e {
                structure_block::StructureBlockEntry::Node(_) => true,
                structure_block::StructureBlockEntry::Property(_) => false,
            })
            .map(|e| match e {
                structure_block::StructureBlockEntry::Node(node) => node,
                structure_block::StructureBlockEntry::Property(_) => panic!(),
            });
    }

    /// Get root node in the structure block.
    pub fn root_node(&self) -> Option<node::Node> {
        return self.node_iter().next();
    }

    /// Get node by phandle.
    pub fn node_by_phandle(&self, phandle: u32) -> Option<node::Node> {
        for node in self.node_iter() {
            for property in node.property_iter() {
                if property.name == "phandle" {
                    if let PropertyValue::U32(handle) = property.get_value() {
                        if handle == phandle {
                            return Some(node);
                        }
                    }
                }
            }
        }

        return None;
    }

    /// Return a iterator for each node and property in the structure block.
    ///
    /// Returns an iterator representing the visited node/property of a depth-first search of the
    /// structure block within the flattened devicetree.
    pub(crate) fn structure_block_iter(&self) -> structure_block::StructureBlockIter {
        /* Get a pointer to the beginning of the structure block */
        assert!(self.check_access_dtb(self.header.as_ptr()));
        let structure_block_offset = unsafe { self.header.as_ref().off_dt_struct() };
        let curr_token: *const u32 =
            unsafe { self.dtb_ptr.add(structure_block_offset as usize).cast() };

        /* Sanity checks */
        if curr_token.align_offset(4) != 0 {
            panic!(
                "Unable to process structure block: {}",
                ParserError::UnalignedAccess
            );
        }
        if !self.check_access_structure_block(curr_token) {
            panic!(
                "Unable to process structure block: {}",
                ParserError::OutOfBoundsAccess
            );
        }

        return structure_block::StructureBlockIter {
            parser: self,
            curr_token: ptr::NonNull::new(curr_token.cast_mut())
                .expect("The structure block pointer must not be NULL!"),
            curr_node: None,
            depth: 0,
        };
    }

    /// Perform manual bounds check.
    ///
    /// Check whether the objected pointed by `ptr` of type `T` fits within the given bounds
    /// (`[mem_start, mem_start + mem_end{`).
    ///
    /// * `ptr`: Object pointer.
    /// * `mem_ptr`: Pointer to start of memory region.
    /// * `mem_size`: Size (in bytes) of memory region.
    pub(crate) fn check_access<T, U>(ptr: *const T, mem_ptr: *const U, mem_size: usize) -> bool {
        let mem_start: *const u8 = mem_ptr.cast();
        let mem_end = unsafe { mem_start.add(mem_size) };
        let ptr_start: *const u8 = ptr.cast();
        let ptr_end = unsafe { ptr_start.add(mem::size_of::<T>()) };

        assert!(mem_start <= mem_end);
        assert!(ptr_start <= ptr_end);

        if ptr_start < mem_start || ptr_start >= mem_end {
            return false;
        }

        if ptr_end < mem_start || ptr_end > mem_end {
            return false;
        }

        return true;
    }

    /// Perform manual bounds check within the flattened devicetree.
    ///
    /// Check whether the objected pointed by `ptr` of type `T` fits within the provided flattened
    /// devicetree blob.
    ///
    /// * `ptr`: Object pointer.
    pub(crate) fn check_access_dtb<T>(&self, ptr: *const T) -> bool {
        return Self::check_access(ptr, self.dtb_ptr, self.dtb_size);
    }

    /// Perform manual bounds check within the structure block of the flattened devicetree.
    ///
    /// Check whether the objected pointed by `ptr` of type `T` fits within the structure block of provided flattened
    /// devicetree blob.
    ///
    /// * `ptr`: Object pointer.
    pub(crate) fn check_access_structure_block<T>(&self, ptr: *const T) -> bool {
        /* Get bounds of structure block within provided flattened devicetree */
        assert!(self.check_access_dtb(self.header.as_ptr()));
        let structure_block_offset = unsafe { self.header.as_ref().off_dt_struct() };
        let structure_block_size = unsafe { self.header.as_ref().size_dt_struct() };

        if (structure_block_offset + structure_block_size) as usize > self.dtb_size {
            return false;
        }
        let structure_block_start = unsafe { self.dtb_ptr.add(structure_block_offset as usize) };

        return Self::check_access(ptr, structure_block_start, structure_block_size as usize);
    }

    /// Perform manual bounds check within the strings block of the flattened devicetree.
    ///
    /// Check whether the objected pointed by `ptr` of type `T` fits within the strings block of provided flattened
    /// devicetree blob.
    ///
    /// * `ptr`: Object pointer.
    pub(crate) fn check_access_strings_block<T>(&self, ptr: *const T) -> bool {
        /* Get bounds of strings block within provided flattened devicetree */
        assert!(self.check_access_dtb(self.header.as_ptr()));
        let strings_block_offset = unsafe { self.header.as_ref().off_dt_strings() };
        let strings_block_size = unsafe { self.header.as_ref().size_dt_strings() };

        if (strings_block_offset + strings_block_size) as usize > self.dtb_size {
            return false;
        }
        let strings_block_start = unsafe { self.dtb_ptr.add(strings_block_offset as usize) };

        return Self::check_access(ptr, strings_block_start, strings_block_size as usize);
    }

    pub(crate) fn get_str_from_structure_block(&self, ptr: *const u8) -> Result<&str, ParserError> {
        let start = ptr;
        let mut end = ptr;

        /* Search end of string */
        loop {
            /* Check access */
            if !self.check_access_structure_block(end) {
                return Err(ParserError::OutOfBoundsAccess);
            }

            /* Load character */
            let character = unsafe { end.read() };

            /* Check for null byte */
            if character == 0 {
                break;
            }

            /* Otherwise, increment pointer */
            end = unsafe { end.add(1) };
        }

        /* Create str from poitners */
        let length: usize = unsafe { end.offset_from(start).try_into().unwrap() };
        let slice = unsafe { slice::from_raw_parts(start, length) };
        return Ok(str::from_utf8(slice).unwrap());
    }

    pub(crate) fn get_str_from_strings_block(&self, offset: u32) -> Result<&str, ParserError> {
        /* Get pointer to start of string */
        assert!(self.check_access_dtb(self.header.as_ptr()));
        let strings_block_offset = unsafe { self.header.as_ref().off_dt_strings() };
        let start = unsafe {
            self.dtb_ptr
                .add(strings_block_offset as usize)
                .add(offset as usize)
        };
        let mut end = start;

        /* Search end of string */
        loop {
            /* Check access */
            if !self.check_access_strings_block(end) {
                return Err(ParserError::OutOfBoundsAccess);
            }

            /* Load character */
            let character = unsafe { end.read() };

            /* Check for null byte */
            if character == 0 {
                break;
            }

            /* Otherwise, increment pointer */
            end = unsafe { end.add(1) };
        }

        /* Create str from poitners */
        let length: usize = unsafe { end.offset_from(start).try_into().unwrap() };
        let slice = unsafe { slice::from_raw_parts(start, length) };
        return Ok(str::from_utf8(slice).unwrap());
    }
}

/// Error codes for Flattend Devicetree Parser.
#[derive(Debug, PartialEq, Eq)]
pub enum ParserError {
    /// Potential unaligned access.
    ///
    /// The devicetree specification defines a set of minimal alignment requirements, including:
    /// * reservation block (aligned to 8-byte boundary)
    /// * structure block (aligned to 4-byte boundary)
    /// * strcuture block token (aligned to 4-byte boundary)
    /// If one of this is not satifies, the `UnalignedAccess` error will be returned.
    ///
    /// See:
    /// * Section 5.4.1 Lexical structure.
    /// * Section 5.6 Alignment.
    UnalignedAccess,
    /// Potential Out-of-bounds access.
    OutOfBoundsAccess,
    /// Unexpected magic value within `header::FDTHeader`.
    InvalidMagicValue,
    /// Unsupported flattend devicetree version (Currently, only `header::FDT_HEADER_SUPPORTED_VERSION` is supported).
    UnsupportedVersion,
    /// Unsupported token within structure block.
    ///
    /// See Section 5.4.1 Lexical structure.
    InvalidStructureBlockToken,
}

impl Display for ParserError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            parser_unaligned_access => write!(f, "Misalinged access"),
            parser_out_of_bounds_access => write!(f, "Out of bounds accecss"),
            parser_invalid_magic_value => write!(f, "Unexpected magic value"),
            parser_unsupported_version => write!(f, "Unsupported version"),
            parser_invalid_structure_block_token => {
                write!(f, "Unexpected strcuture block token")
            }
        }
    }
}
