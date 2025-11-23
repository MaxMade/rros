use crate::boot::device_tree::node;
use crate::boot::device_tree::parser;
use crate::boot::device_tree::property;

use core::ptr;
use core::slice;

use core::fmt::Display;

/// Token within the structure block.
///
/// The structure block is composed of a sequence of pieces, each beginning with a token, that is,
/// a big-endian 4-byte integer. All tokens shall be aligned on a 4-byte boundary.
///
/// See Section 5.4.1 Lexical structure.
#[derive(Debug, PartialEq, Eq)]
pub enum Token {
    /// Marker indicating the beginning of a node’s representation.
    FDTBeginNode = 0x00000001,
    /// Marker indicating the end of a node’s representation
    FDTEndNode = 0x00000002,
    /// Marker indicating the beginning of the representation of one property in the devicetree.
    FDTProp = 0x00000003,
    /// NOP token, which will be ignored.
    FDTNop = 0x00000004,
    /// Marker indicating the end of the structure block.
    FDTEnd = 0x00000009,
}

impl TryFrom<u32> for Token {
    type Error = parser::ParserError;

    /// Convert value (host-endianness) to token.
    ///
    /// * `value`: Input value.
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if value == 0x00000001 {
            return Ok(Self::FDTBeginNode);
        } else if value == 0x00000002 {
            return Ok(Self::FDTEndNode);
        } else if value == 0x00000003 {
            return Ok(Self::FDTProp);
        } else if value == 0x00000004 {
            return Ok(Self::FDTNop);
        } else if value == 0x00000009 {
            return Ok(Self::FDTEnd);
        } else {
            return Err(parser::ParserError::InvalidStructureBlockToken);
        }
    }
}

/// Entry within structure block.
#[derive(Debug)]
pub enum StructureBlockEntry<'a> {
    /// Node.
    Node(node::Node<'a>),
    /// Property (within node).
    Property(property::Property<'a>),
}

impl<'a> Display for StructureBlockEntry<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            StructureBlockEntry::Node(node) => write!(f, "{}", node),
            StructureBlockEntry::Property(property) => write!(f, "{}", property),
        }
    }
}

/// Raw iterator over structure block entries.
///
/// The `StructureBlockIter` will enumerate each node and property within the flattened devicetree in
/// sequential order.
pub struct StructureBlockIter<'a> {
    /// Reference to parser.
    pub(crate) parser: &'a parser::Parser,
    /// Current token (pointer) within structure block.
    pub(crate) curr_token: ptr::NonNull<u32>,
    /// Currently parsed node within structure block.
    pub(crate) curr_node: Option<node::Node<'a>>,
    /// 0-based depth within devicetree.
    pub(crate) depth: usize,
}

impl<'a> Iterator for StructureBlockIter<'a> {
    type Item = StructureBlockEntry<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            /* Check current token pointer */
            assert!(self.curr_token.as_ptr().align_offset(4) == 0);
            assert!(self
                .parser
                .check_access_structure_block(self.curr_token.as_ptr()));

            /* Load token */
            let raw_token = u32::from_be(unsafe { self.curr_token.as_ptr().read() });
            let token = match Token::try_from(raw_token) {
                Ok(token) => token,
                Err(error) => panic!(
                    "Unable to process next token within structure block: {}",
                    error
                ),
            };

            /* Update current token pointer */
            self.curr_token =
                unsafe { ptr::NonNull::new(self.curr_token.as_ptr().add(1)).unwrap() };

            /* Process next token */
            let entry = match token {
                Token::FDTBeginNode => {
                    /* Increase depth */
                    self.depth += 1;

                    /* Try to parse name */
                    let mut name = match self
                        .parser
                        .get_str_from_structure_block(self.curr_token.as_ptr().cast())
                    {
                        Ok(name) => name,
                        Err(error) => panic!(
                            "Unable to process name of node within structure block: {}",
                            error
                        ),
                    };

                    /* Update current token pointer */
                    self.curr_token = unsafe {
                        ptr::NonNull::new(
                            self.curr_token
                                .as_ptr()
                                .cast::<u8>()
                                .add(name.len() + 1)
                                .cast(),
                        )
                        .unwrap()
                    };

                    /* XXX: Root node ("/") uses an empty string as its name! */
                    if name.len() == 0 {
                        name = "/".into();
                    }

                    /* Create node */
                    let node = node::Node {
                        parser: self.parser,
                        name,
                        curr_token: self.curr_token,
                        depth: self.depth,
                    };

                    self.curr_node = Some(node.clone());

                    /* Return node */
                    StructureBlockEntry::Node(node)
                }
                Token::FDTEndNode => {
                    /* Decrease depth */
                    self.depth -= 1;

                    continue;
                }
                Token::FDTProp => {
                    /* Try to parse property length */
                    assert!(self
                        .parser
                        .check_access_structure_block(self.curr_token.as_ptr()));

                    /* Load token */
                    let length = u32::from_be(unsafe { self.curr_token.as_ptr().read() });

                    /* Update current token pointer */
                    self.curr_token =
                        unsafe { ptr::NonNull::new(self.curr_token.as_ptr().add(1)).unwrap() };

                    /* Try to parse property name offset */
                    assert!(self
                        .parser
                        .check_access_structure_block(self.curr_token.as_ptr()));

                    /* Load token */
                    let name_offset = u32::from_be(unsafe { self.curr_token.as_ptr().read() });

                    /* Update current token pointer */
                    self.curr_token =
                        unsafe { ptr::NonNull::new(self.curr_token.as_ptr().add(1)).unwrap() };

                    /* Get name */
                    let name = match self.parser.get_str_from_strings_block(name_offset) {
                        Ok(name) => name,
                        Err(error) => panic!(
                            "Unable to process name of proptery within structure block: {}",
                            error
                        ),
                    };

                    /* Get value */
                    let value = unsafe {
                        slice::from_raw_parts(
                            self.curr_token.as_ptr().cast::<u8>(),
                            length as usize,
                        )
                    };

                    /* Update current token pointer */
                    self.curr_token = unsafe {
                        ptr::NonNull::new(
                            self.curr_token
                                .as_ptr()
                                .cast::<u8>()
                                .add(length as usize)
                                .cast(),
                        )
                        .unwrap()
                    };

                    StructureBlockEntry::Property(property::Property {
                        node: self.curr_node.clone().unwrap(),
                        name,
                        value,
                        depth: self.depth,
                    })
                }
                Token::FDTNop => {
                    /* Nothing to do here */
                    continue;
                }
                Token::FDTEnd => {
                    /* Reached end of token stream, nothing left to do */
                    return None;
                }
            };

            /* Align current token pointer */
            let alignment_offset = self.curr_token.as_ptr().cast::<u8>().align_offset(4);
            self.curr_token = unsafe {
                ptr::NonNull::new(
                    self.curr_token
                        .as_ptr()
                        .cast::<u8>()
                        .add(alignment_offset)
                        .cast(),
                )
                .unwrap()
            };

            return Some(entry);
        }
    }
}
