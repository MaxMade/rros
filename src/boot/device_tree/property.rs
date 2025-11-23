use crate::boot::device_tree::node;

use core::mem;
use core::str;

use core::fmt::Display;

#[derive(Debug)]
pub struct Property<'a> {
    /// Reference to associated node.
    pub(crate) node: node::Node<'a>,
    /// Name of the name.
    pub(crate) name: &'a str,
    /// Raw value.
    pub(crate) value: &'a [u8],

    /// 0-based depth within devicetree.
    pub(crate) depth: usize,
}

impl<'a> Display for Property<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl<'a> Property<'a> {
    /// Get associated node.
    pub fn get_node(&self) -> &node::Node {
        return &self.node;
    }

    /// Get value.
    pub fn get_value(&self) -> PropertyValue {
        // Process empty values
        if self.value.len() == 0 {
            if self.name == "interrupt-controller"
                || self.name == "cache-unified"
                || self.name == "tlb-split"
                || self.name.starts_with("power-isa-")
                || self.name == "dma-coherent"
            {
                return PropertyValue::Empty;
            }
        }

        // Process U32 values
        if self.value.len() == 4 {
            if self.name == "phandle"
                || self.name == "#address-cells"
                || self.name == "#size-cells"
                || self.name == "virtual-reg"
                || self.name == "#interrupt-cells"
                || self.name == "cache-op-block-size"
                || self.name == "reservation-granule-size"
                || self.name == "tlb-size"
                || self.name == "tlb-sets"
                || self.name == "d-tlb-size"
                || self.name == "d-tlb-sets"
                || self.name == "i-tlb-size"
                || self.name == "i-tlb-sets"
                || self.name == "cache-size"
                || self.name == "cache-sets"
                || self.name == "cache-block-size"
                || self.name == "cache-line-size"
                || self.name == "i-cache-size"
                || self.name == "i-cache-sets"
                || self.name == "i-cache-block-size"
                || self.name == "i-cache-line-size"
                || self.name == "d-cache-size"
                || self.name == "d-cache-sets"
                || self.name == "d-cache-block-size"
                || self.name == "d-cache-line-size"
                || self.name == "next-level-cache"
                || self.name == "cache-level"
                || self.name == "reg-shift"
                || self.name == "clock-frequency"
                || self.name == "current-speed"
                || self.name == "clock-frequency"
                || self.name == "address-bits"
                || self.name == "max-frame-size"
                || self.name == "max-speed"
                || self.name == "riscv,ndev"
            {
                return PropertyValue::U32(
                    (self.value[0] as u32) << 24
                        | (self.value[1] as u32) << 16
                        | (self.value[2] as u32) << 8
                        | self.value[3] as u32,
                );
            }
        }

        // Process U64 values
        if self.value.len() == 8 {
            if self.name == "cpu-release-addr"
                || self.name == "clock-frequency"
                || self.name == "virtual-reg"
            {
                return PropertyValue::U64(
                    (self.value[0] as u64) << 56
                        | (self.value[1] as u64) << 48
                        | (self.value[2] as u64) << 40
                        | (self.value[3] as u64) << 32
                        | (self.value[4] as u64) << 24
                        | (self.value[5] as u64) << 16
                        | (self.value[6] as u64) << 8
                        | self.value[7] as u64,
                );
            }
        }

        // Process String values
        if self.name == "model"
            || self.name == "status"
            || self.name == "name"
            || self.name == "device_type"
            || self.name == "bootargs"
            || self.name == "stdout-path"
            || self.name == "stdin-path"
            || self.name == "power-isa-version"
            || self.name == "mmu-type"
            || self.name == "compatible"
            || self.name == "label"
            || self.name == "phy-connection-type"
        {
            let length = self.value.len() - 1;
            let value = str::from_utf8(&self.value[0..length]).unwrap();

            return PropertyValue::String(value);
        }

        // Process PropEncodedArray values
        if self.name == "reg" {
            // Sanity check: The value must consist of multiple u32 values!
            assert!(self.value.len() % mem::size_of::<u32>() == 0);

            let values: &[u32] = unsafe {
                core::slice::from_raw_parts(
                    self.value.as_ptr().cast(),
                    self.value.len() / mem::size_of::<u32>(),
                )
            };

            return PropertyValue::PropEncodedArray(values);
        }

        // Fallback: Return raw values
        return PropertyValue::Raw(self.value);
    }

    /// Return iterator for <address, length> pairs.
    ///
    /// The `reg` property defines a list of <address, length> pairs of the device’s resources
    /// within the address space defined by its parent bus.
    pub fn into_addr_length_iter(&self) -> AddrLengthArrayIter {
        assert!(self.name == "reg");

        let parent_node = match self.node.get_parent_node() {
            Some(node) => node,
            None => {
                return AddrLengthArrayIter {
                    value: self.value,
                    address_cells: 2,
                    size_cells: 1,
                    offset: 0,
                };
            }
        };

        let address_cells = match parent_node
            .property_iter()
            .find(|e| e.name == "#address-cells")
        {
            Some(cell) => cell,
            None => {
                return AddrLengthArrayIter {
                    value: self.value,
                    address_cells: 2,
                    size_cells: 1,
                    offset: 0,
                };
            }
        };
        let address_cells = match address_cells.get_value() {
            PropertyValue::U32(cells) => cells,
            _ => panic!("Each node with a 'reg' property must have a parent node with the associated '#address-cells' (U32) property!"),
        };

        let size_cells = match parent_node
            .property_iter()
            .find(|e| e.name == "#size-cells")
        {
            Some(cell) => cell,
            None => {
                return AddrLengthArrayIter {
                    value: self.value,
                    address_cells: 2,
                    size_cells: 1,
                    offset: 0,
                };
            }
        };
        let size_cells = match size_cells.get_value() {
            PropertyValue::U32(cells) => cells,
            _ => panic!("Each node with a 'reg' property must have a parent node with the associated '#size-cells' (U32) property!"),
        };

        return AddrLengthArrayIter {
            value: self.value,
            address_cells,
            size_cells,
            offset: 0,
        };
    }
}

/// Interpretation of property value.
#[derive(Debug)]
pub enum PropertyValue<'a> {
    /// Value used for conveying boolean information, when the presence or absence of
    /// the property itself is sufficiently descriptive.
    Empty,
    /// A 32-bit integer (in host endianess).
    U32(u32),
    /// A 64-bit integer (in host endianess).
    U64(u64),
    /// A string value.
    String(&'a str),
    /// Raw (uninterpreted) values encoded as array of 32-bit big-endian values.
    PropEncodedArray(&'a [u32]),
    /// Raw (uninterpreted) value (used as fallback) as big-endian values.
    Raw(&'a [u8]),
}

impl<'a> Display for PropertyValue<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            PropertyValue::Empty => {
                write!(f, "")
            }
            PropertyValue::U32(value) => {
                write!(f, "<{:#x}>", value)
            }
            PropertyValue::U64(value) => {
                write!(f, "<{:#x}>", value)
            }
            PropertyValue::String(value) => {
                write!(f, "\"")?;
                for character in value.chars() {
                    match character {
                        '\0' => write!(f, "\\0")?,
                        _ => write!(f, "{}", character)?,
                    }
                }
                write!(f, "\"")
            }
            PropertyValue::PropEncodedArray(values) => {
                write!(f, "<")?;
                for (i, value) in values.iter().enumerate() {
                    write!(f, "{:#04x}", u32::from_be(*value))?;
                    if i != values.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, ">")
            }
            PropertyValue::Raw(value) => {
                write!(f, "[")?;
                for (i, byte) in value.iter().enumerate() {
                    write!(f, "{:02x}", u8::from_be(*byte))?;
                    if i != value.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, "]")
            }
        }
    }
}

/// Iterator for <address, length> pairs of the device’s resources within the address space defined by its parent bus.
#[derive(Debug, Clone)]
pub struct AddrLengthArrayIter<'a> {
    /// Raw value.
    value: &'a [u8],
    /// Number of `u32` cells required to specify the address (specified by `#address-cells`properties in the parent of the device node).
    address_cells: u32,
    /// Number of `u32` cells required to specify the length (specified by `#size-cells` properties in the parent of the device node).
    size_cells: u32,
    /// Current offset within `value` member.
    offset: usize,
}

impl<'a> Iterator for AddrLengthArrayIter<'a> {
    type Item = (usize, usize);

    fn next(&mut self) -> Option<Self::Item> {
        let address_bytes = mem::size_of::<u32>() * self.address_cells as usize;
        let size_bytes = mem::size_of::<u32>() * self.size_cells as usize;

        /* Check bounds */
        if self.offset + address_bytes + size_bytes > self.value.len() {
            return None;
        }

        /* Sanity-check: usize should be able to represent any given address/length */
        assert!(mem::size_of::<usize>() >= address_bytes);
        assert!(mem::size_of::<usize>() >= size_bytes);

        /* Calculate address */
        let mut address = 0usize;
        for i in 0..address_bytes {
            let mut chunk = self.value[self.offset + i] as usize;
            chunk = chunk << ((u8::BITS as usize) * (address_bytes - i - 1));
            address |= chunk;
        }

        /* Update offset */
        self.offset += address_bytes;

        /* Calculate length */
        let mut length = 0usize;
        for i in 0..size_bytes {
            let mut chunk = self.value[self.offset + i] as usize;
            chunk = chunk << ((u8::BITS as usize) * (size_bytes - i - 1));
            length |= chunk;
        }

        /* Update offset */
        self.offset += size_bytes;

        return Some((address, length));
    }
}
