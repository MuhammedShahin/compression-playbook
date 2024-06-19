use crate::nonmax::NonMaxU16;
use crate::package_merge::{package_merge, PackageMergeError};

use std::collections::binary_heap::BinaryHeap;

pub struct HuffmanTree {
    nodes: Vec<Node>,
    num_symbols: usize,
}

#[derive(Debug, Default)]
pub struct HuffmanTable {
    pub codes: Vec<PrefixCode>,
}

#[derive(Default, Clone, Copy)]
pub struct PrefixCode {
    pub code: u32,
    pub length: u8,
}

#[derive(Debug, Clone, Copy)]
pub struct WalkIterator {
    pub code: PrefixCode,
    pub idx: usize,
    pub leaf: bool,
}

#[derive(Copy, Clone)]
struct Node {
    left: Option<NonMaxU16>,
    right: Option<NonMaxU16>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct HeapEntry {
    freq: std::cell::Cell<u32>,
    idx: u16,
}

impl PrefixCode {
    fn update(prefix_code: PrefixCode, bit: bool) -> PrefixCode {
        PrefixCode {
            code: prefix_code.code | ((bit as u32) << (prefix_code.length)),
            length: prefix_code.length + 1,
        }
    }
}

impl std::fmt::Debug for PrefixCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for i in 0..self.length {
            write!(f, "{}", (self.code & ((1 as u32) << i)) >> i)?;
        }
        Ok(())
    }
}

impl HuffmanTree {
    pub fn build(freqs: &[u32]) -> HuffmanTree {
        let num_symbols = freqs.len();
        let capacity = 2 * num_symbols - 1;
        assert!(capacity <= (std::u16::MAX - 1).into());

        let mut nodes = Vec::<Node>::new();

        nodes.reserve(capacity);

        // Reverse so that it becomes a min heap.
        let mut heap = BinaryHeap::<std::cmp::Reverse<HeapEntry>>::new();

        for (idx, freq) in freqs.iter().enumerate() {
            nodes.push(Node {
                left: None,
                right: None,
            });

            // Ignore symbols that have probability 0.
            if *freq != 0 {
                heap.push(std::cmp::Reverse(HeapEntry {
                    freq: (*freq).into(),
                    idx: idx as u16,
                }));
            }
        }

        // Until the heap is empty, we pop the two smallest elements, and create an internal node
        // from them.
        while heap.len() > 1 {
            let entry1 = heap.pop().unwrap(); // unwrap should always succeed here
            let entry2 = heap.pop().unwrap();

            // Create internal node with children being the least two nodes.
            let internal_node = Node {
                left: NonMaxU16::new(entry1.0.idx),
                right: NonMaxU16::new(entry2.0.idx),
            };

            let internal_node_idx = nodes.len() as u16;
            heap.push(std::cmp::Reverse(HeapEntry {
                freq: (entry1.0.freq.get() + entry2.0.freq.get()).into(),
                idx: internal_node_idx,
            }));
            nodes.push(internal_node);
        }

        HuffmanTree { nodes, num_symbols }
    }

    fn is_leaf_node(&self, idx: usize) -> bool {
        (idx as usize) < self.num_symbols
    }

    pub fn create_walk_iter(&self) -> WalkIterator {
        WalkIterator {
            idx: self.nodes.len() - 1,
            code: PrefixCode::default(),
            leaf: false,
        }
    }

    pub fn walk(&self, iter: WalkIterator, bit: bool) -> Option<WalkIterator> {
        let idx;

        if bit {
            if let Some(right) = self.nodes[iter.idx].right {
                idx = right.get().into();
            } else {
                return None;
            }
        } else {
            if let Some(left) = self.nodes[iter.idx].left {
                idx = left.get().into();
            } else {
                return None;
            }
        };

        Some(WalkIterator {
            code: PrefixCode::update(iter.code, bit),
            idx,
            leaf: self.is_leaf_node(idx),
        })
    }
}

impl HuffmanTable {
    pub fn build_length_limited(
        freqs: &[u32],
        max_length: usize,
    ) -> Result<Self, PackageMergeError> {
        let lengths = package_merge(&freqs, max_length)?;

        let table = Self::from_lengths(&lengths);

        // Assert that we achieved the required lengths.
        debug_assert!(table
            .codes
            .iter()
            .all(|&code| code.length as usize <= max_length));

        Ok(table)
    }

    pub fn from_lengths(lengths: &[u8]) -> Self {
        let mut lengths_count: [u32; 32] = [0; 32];
        let mut codes = Vec::new();
        codes.reserve(lengths.len());

        for length in lengths {
            lengths_count[*length as usize] += 1;

            codes.push(PrefixCode {
                code: 0, // Will be initialized later,
                length: *length,
            });
        }
        let mut table = Self { codes };

        table.canonicalize_impl(&lengths_count);

        table
    }

    fn build_impl(tree: &HuffmanTree, idx: usize, code: PrefixCode, table: &mut HuffmanTable) {
        if tree.is_leaf_node(idx) {
            table.codes[idx] = code;
            return;
        }

        let node = &tree.nodes[idx];

        if let Some(left) = node.left {
            Self::build_impl(
                tree,
                left.get().into(),
                PrefixCode::update(code, false),
                table,
            );
        }

        if let Some(right) = node.right {
            Self::build_impl(
                tree,
                right.get().into(),
                PrefixCode::update(code, true),
                table,
            );
        }
    }

    pub fn canonicalize(&mut self) {
        let mut lengths_count: [u32; 32] = [0; 32];
        for code in &self.codes {
            lengths_count[code.length as usize] += 1;
        }

        self.canonicalize_impl(&lengths_count)
    }

    fn canonicalize_impl(&mut self, lengths_count: &[u32; 32]) {
        // Same implementation as in RFC1951 (but MAX_BITS is extended a little bit)
        let mut next_code: [u32; 32] = [0; 32];

        let mut code = 0;
        for (idx, count) in lengths_count[1..].iter().enumerate() {
            next_code[idx + 1] = code;
            code = (code + count) << 1;
        }

        for code in self.codes.iter_mut() {
            if code.length != 0 {
                code.code = next_code[code.length as usize].reverse_bits() >> (32 - code.length);
                next_code[code.length as usize] += 1;
            }
        }
    }

    pub fn code(&self, symbol: usize) -> &PrefixCode {
        &self.codes[symbol]
    }
}

impl From<&HuffmanTree> for HuffmanTable {
    fn from(tree: &HuffmanTree) -> Self {
        let mut table = HuffmanTable { codes: Vec::new() };
        // Is there a better way to directly initialize the vector in the above line?
        table.codes.resize(tree.num_symbols, PrefixCode::default());

        Self::build_impl(
            tree,
            tree.nodes.len() - 1,
            PrefixCode::default(),
            &mut table,
        );

        return table;
    }
}

impl From<&HuffmanTable> for HuffmanTree {
    fn from(table: &HuffmanTable) -> Self {
        let num_symbols = table.codes.len();
        let capacity = 2 * num_symbols - 1;

        let mut nodes = Vec::<Node>::new();
        nodes.resize(
            capacity,
            Node {
                left: None,
                right: None,
            },
        );

        //  The last node is the root
        let root_idx = nodes.len() - 1;
        let mut alloc_idx = nodes.len() - 1;

        for (idx, code) in table.codes.iter().enumerate() {
            let mut crawler_idx = root_idx;

            if code.length == 0 {
                continue;
            }
            for bit_idx in 0..code.length {
                let bit = code.code & ((0b1 as u32) << bit_idx);
                if bit == 0 {
                    match nodes[crawler_idx].left {
                        None => {
                            let node_idx = if bit_idx == code.length - 1 {
                                idx
                            } else {
                                alloc_idx -= 1;
                                alloc_idx
                            };

                            nodes[crawler_idx].left = NonMaxU16::new(node_idx as u16);
                            crawler_idx = node_idx;
                        }
                        Some(idx) => {
                            assert!(bit_idx != code.length);
                            crawler_idx = idx.get().into();
                        }
                    }
                } else {
                    match nodes[crawler_idx].right {
                        None => {
                            let node_idx = if bit_idx == code.length - 1 {
                                idx
                            } else {
                                alloc_idx -= 1;
                                alloc_idx
                            };

                            nodes[crawler_idx].right = NonMaxU16::new(node_idx as u16);
                            crawler_idx = node_idx;
                        }
                        Some(idx) => {
                            assert!(bit_idx != code.length);
                            crawler_idx = idx.get().into();
                        }
                    }
                }
            }

            assert!(nodes[crawler_idx].left.is_none());
            assert!(nodes[crawler_idx].right.is_none());
        }

        HuffmanTree { nodes, num_symbols }
    }
}
