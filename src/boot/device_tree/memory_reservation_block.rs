use crate::boot::device_tree::parser;

/// Iterator for memory reservation entries.
///
/// The memory reservation block provides the client program with a list of areas in physical
/// memory which are reserved and must not be used for general memory allocations.
#[derive(Debug)]
pub struct MemoryReservationIter<'a> {
    pub(crate) parser: &'a parser::Parser,
    pub(crate) ptr: *const u64,
}

impl<'a> Iterator for MemoryReservationIter<'a> {
    type Item = (u64, u64);

    fn next(&mut self) -> Option<Self::Item> {
        let mut ptr: *const u64 = self.ptr.cast();

        /* Try to load address */
        assert!(self.parser.check_access_dtb(ptr));
        let address = unsafe { u64::from_be(*ptr) };

        /* Temporary increase pointer */
        ptr = unsafe { ptr.add(1) };

        /* Try to load size */
        assert!(self.parser.check_access_dtb(ptr));
        let size = unsafe { u64::from_be(*ptr) };

        if address == 0 && size == 0 {
            return None;
        }

        /* Update pointer */
        self.ptr = unsafe { self.ptr.add(2) };

        return Some((address, size));
    }
}
