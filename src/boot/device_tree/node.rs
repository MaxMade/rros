use crate::boot::device_tree::parser;
use crate::boot::device_tree::property;
use crate::boot::device_tree::structure_block;

use core::ptr;

use core::fmt::Display;

/// Device tree node.
///
/// A devicetree is a tree data structure with nodes that describe the devices in a system. Each
/// node has property/value pairs that describe the characteristics of the device being
/// represented. Each node has exactly one parent except for the root node, which has no parent.
#[derive(Debug, Clone)]
pub struct Node<'a> {
    /// Reference to parser.
    pub(crate) parser: &'a parser::Parser,
    /// Name of the node.
    pub(crate) name: &'a str,

    /// Current token (pointer) within structure block.
    pub(crate) curr_token: ptr::NonNull<u32>,
    /// 0-based depth within devicetree.
    pub(crate) depth: usize,
}

impl<'a> Display for Node<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl<'a> Node<'a> {
    /// Get name of node.
    pub fn name(&self) -> &'a str {
        return self.name;
    }

    /// Try to get parent node.
    ///
    /// Returns the preceding node if possible. In case of the root node ("/") `None` will be
    /// returned.
    pub fn get_parent_node(&self) -> Option<Node<'a>> {
        for entry in self.parser.structure_block_iter() {
            if let structure_block::StructureBlockEntry::Node(node) = entry {
                if node
                    .children_node_iter()
                    .find(|e| e.curr_token == self.curr_token)
                    .is_some()
                {
                    return Some(node);
                }
            }
        }
        return None;
    }

    /// Get iterator for properties associated with node.
    pub fn property_iter(&self) -> PropertyIter {
        /* Align current token pointer */
        let alignment_offset = self.curr_token.as_ptr().cast::<u8>().align_offset(4);
        let curr_token = unsafe {
            ptr::NonNull::new(
                self.curr_token
                    .as_ptr()
                    .cast::<u8>()
                    .add(alignment_offset)
                    .cast(),
            )
            .unwrap()
        };

        /* Return wrapper for StructureBlockIter */
        let structure_block_iter = structure_block::StructureBlockIter {
            parser: self.parser,
            curr_token,
            curr_node: Some(self.clone()),
            depth: self.depth,
        };
        let property_iter = PropertyIter {
            structure_block_iter,
            depth: self.depth,
        };
        return property_iter;
    }

    /// Get iterator for (direct) children associated with given node.
    pub fn children_node_iter(&self) -> ChildNodeIter {
        /* Align current token pointer */
        let alignment_offset = self.curr_token.as_ptr().cast::<u8>().align_offset(4);
        let curr_token = unsafe {
            ptr::NonNull::new(
                self.curr_token
                    .as_ptr()
                    .cast::<u8>()
                    .add(alignment_offset)
                    .cast(),
            )
            .unwrap()
        };

        /* Return wrapper for ChildNodeIter */
        let structure_block_iter = structure_block::StructureBlockIter {
            parser: self.parser,
            curr_token,
            curr_node: Some(self.clone()),
            depth: self.depth,
        };
        let children_node_iter = ChildNodeIter {
            structure_block_iter,
            depth: self.depth,
        };
        return children_node_iter;
    }
}

/// Property iterator of node entry.
///
/// The `PropertyIter` will enumerate each property of the associated node within the flattened
/// devicetree in sequential order. Hereby, it will make use of the raw `StructureBlockIter`.
pub struct PropertyIter<'a> {
    /// Underlying raw iterator.
    pub(crate) structure_block_iter: structure_block::StructureBlockIter<'a>,
    /// 0-based depth within devicetree.
    pub(crate) depth: usize,
}

impl<'a> Iterator for PropertyIter<'a> {
    type Item = property::Property<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        /* Perform depth-first search regarding current subtree */
        while self.depth <= self.structure_block_iter.depth {
            /* Try to get next node/property */
            let next = match self.structure_block_iter.next() {
                Some(next) => next,
                None => return None,
            };

            /* Early out if non-child node encountered */
            if let structure_block::StructureBlockEntry::Node(node) = &next {
                if node.depth <= self.depth {
                    return None;
                }
            };

            /* Return property if it is at the same level */
            if let structure_block::StructureBlockEntry::Property(property) = next {
                if property.depth == self.depth {
                    return Some(property);
                }
            };
        }

        return None;
    }
}

/// Iterator for (direct) children node.
///
/// The `ChildNodeIter` will enumerate each (direct) child node of the associated node within the
/// flattened devicetree in sequential order. Hereby, it will make use of the raw
/// `StructureBlockIter`.
pub struct ChildNodeIter<'a> {
    /// Underlying raw iterator.
    pub(crate) structure_block_iter: structure_block::StructureBlockIter<'a>,
    /// 0-based depth within devicetree.
    pub(crate) depth: usize,
}

impl<'a> Iterator for ChildNodeIter<'a> {
    type Item = Node<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        /* Perform depth-first search regarding current subtree */
        while self.depth <= self.structure_block_iter.depth {
            /* Try to get next node/property */
            let next = match self.structure_block_iter.next() {
                Some(next) => next,
                None => return None,
            };

            /* Early out if non-child node encountered */
            if let structure_block::StructureBlockEntry::Node(node) = &next {
                if node.depth <= self.depth {
                    return None;
                }
            };

            /* Return property if it is at the same level */
            if let structure_block::StructureBlockEntry::Node(node) = next {
                if node.depth == self.depth + 1 {
                    return Some(node);
                }
            };
        }

        return None;
    }
}
