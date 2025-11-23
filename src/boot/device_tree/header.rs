//! Flattened Devicetree Header.
//!
//! See Section 5.2 Header.

/// Flattened Devicetree Header.
///
/// See Section 5.2 Header.
#[derive(Debug)]
#[repr(packed)]
pub struct FDTHeader {
    /// Magic value.
    magic: u32,
    /// Total size in bytes of the devicetree data structure.
    totalsize: u32,
    /// Offset in bytes of the structure block from the beginning of the header.
    off_dt_struct: u32,
    /// Offset in bytes of the strings block from the beginning of the header.
    off_dt_strings: u32,
    /// Offset in bytes of the memory reservation block from the beginning of the header
    off_mem_rsvmap: u32,
    /// Version of the devicetree data structure.
    version: u32,
    /// Lowest version of the devicetree data structure with which the version used is backwards compatible.
    last_comp_version: u32,
    /// Physical ID of the system’s boot CPU.
    boot_cpuid_phys: u32,
    /// Length in bytes of the strings block section of the devicetree blob
    size_dt_strings: u32,
    /// Length in bytes of the structure block section of the devicetree blob.
    size_dt_struct: u32,
}

impl FDTHeader {
    /// Get magic value.
    pub fn magic(&self) -> u32 {
        u32::from_be(self.magic)
    }
    /// Get total size in bytes of the devicetree data structure.
    pub fn totalsize(&self) -> u32 {
        u32::from_be(self.totalsize)
    }
    /// Get Offset in bytes of the structure block from the beginning of the header from raw `FDTHeader`.
    pub fn off_dt_struct(&self) -> u32 {
        u32::from_be(self.off_dt_struct)
    }
    /// Get Offset in bytes of the strings block from the beginning of the header from raw `FDTHeader`.
    pub fn off_dt_strings(&self) -> u32 {
        u32::from_be(self.off_dt_strings)
    }
    /// Get Offset in bytes of the memory reservation block from the beginning of the heade from raw `FDTHeader`.
    pub fn off_mem_rsvmap(&self) -> u32 {
        u32::from_be(self.off_mem_rsvmap)
    }
    /// Get Version of the devicetree data structure from raw `FDTHeader`.
    pub fn version(&self) -> u32 {
        u32::from_be(self.version)
    }
    /// Get Lowest version of the devicetree data structure with which the version used is backwards compatible from raw `FDTHeader`.
    pub fn last_comp_version(&self) -> u32 {
        u32::from_be(self.last_comp_version)
    }
    /// Get Physical ID of the system’s boot CPU from raw `FDTHeader`.
    pub fn boot_cpuid_phys(&self) -> u32 {
        u32::from_be(self.boot_cpuid_phys)
    }
    /// Get Length in bytes of the strings block section of the devicetree blo from raw `FDTHeader`.
    pub fn size_dt_strings(&self) -> u32 {
        u32::from_be(self.size_dt_strings)
    }
    /// Get Length in bytes of the structure block section of the devicetree blob from raw `FDTHeader`.
    pub fn size_dt_struct(&self) -> u32 {
        u32::from_be(self.size_dt_struct)
    }
}

/// Magic value of flattened device tree header.
pub const FDT_HEADER_MAGIC: u32 = 0xd00dfeed;

/// Currently supported version of flattened device tree.
pub const FDT_HEADER_SUPPORTED_VERSION: u32 = 17;
