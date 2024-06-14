use std::collections::binary_heap::BinaryHeap;

pub struct HuffmanTree {
    nodes: Vec<Node>,
    num_symbols: usize,
}

#[derive(Debug, Default)]
pub struct HuffmanTable {
    pub codes: Vec<PrefixCode>,
    pub lengths_count: [u32; 32],
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
    freq: u32,
    left: Option<u32>,
    right: Option<u32>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct HeapEntry {
    freq: u32,
    idx: u32,
}

impl PrefixCode {
    fn update(prefix_code: PrefixCode, bit: bool) -> PrefixCode {
        PrefixCode {
            code: prefix_code.code | ((bit as u32) << (prefix_code.length)),
            length: prefix_code.length + 1,
        }
    }
}

impl std::fmt::Display for PrefixCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "\nPrefixCode {{ code: {:b}, length: {} }}\n",
            self.code, self.length
        )
    }
}

impl std::fmt::Debug for PrefixCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "\nPrefixCode {{ code: {:b}, length: {} }}\n",
            self.code, self.length
        )
    }
}

impl HuffmanTree {
    pub fn build(freqs: &[u32]) -> HuffmanTree {
        let num_symbols = freqs.len();
        let capacity = 2 * num_symbols - 1;

        let mut nodes = Vec::<Node>::new();
        nodes.reserve(capacity);

        let mut heap = BinaryHeap::<std::cmp::Reverse<HeapEntry>>::new(); // reverse so that it
                                                                          // becomes a min heap
                                                                          // We first add the leaf nodes, and create binary heap with the nodes.
        for (idx, freq) in freqs.iter().enumerate() {
            nodes.push(Node {
                freq: *freq,
                left: None,
                right: None,
            });

            // Ignore symbols that have probability 0.
            if *freq != 0 {
                heap.push(std::cmp::Reverse(HeapEntry {
                    freq: *freq,
                    idx: idx as u32,
                }));
            }
        }

        // Until the heap is empty, we pop the two smallest elements, and create an internal node
        // from them.
        while heap.len() > 1 {
            let entry1 = heap.pop().unwrap(); // unwrap should always succeed here
            let entry2 = heap.pop().unwrap();

            let node1_idx = entry1.0.idx as usize;
            let node2_idx = entry2.0.idx as usize;

            // Create internal node with children being the least two nodes.
            let internal_node = Node {
                freq: nodes[node1_idx].freq + nodes[node2_idx].freq,
                left: Some(entry1.0.idx),
                right: Some(entry2.0.idx),
            };

            let internal_node_idx = nodes.len() as u32;
            heap.push(std::cmp::Reverse(HeapEntry {
                freq: internal_node.freq,
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

    pub fn walk(&self, iter: WalkIterator, bit: bool) -> WalkIterator {
        let idx = if bit {
            self.nodes[iter.idx].right.unwrap() as usize
        } else {
            self.nodes[iter.idx].left.unwrap() as usize
        };

        WalkIterator {
            code: PrefixCode::update(iter.code, bit),
            idx,
            leaf: self.is_leaf_node(idx),
        }
    }
}

impl HuffmanTable {
    pub fn from_lengths(lengths: &[u8]) -> Self {
        let mut lengths_count: [u32; 32] = [0; 32];
        let mut codes = Vec::new();
        codes.reserve(lengths.len());

        for length in lengths {
            assert!(*length < 32);
            lengths_count[*length as usize] += 1;

            codes.push(PrefixCode {
                code: 0, // Will be initialized later,
                length: *length,
            });
        }
        let mut table = Self {
            codes,
            lengths_count,
        };

        table.canonicalize();

        table
    }

    fn build_impl(tree: &HuffmanTree, idx: usize, code: PrefixCode, table: &mut HuffmanTable) {
        if tree.is_leaf_node(idx) {
            table.codes[idx] = code;
            return;
        }

        let node = &tree.nodes[idx];

        if let Some(left) = node.left {
            Self::build_impl(tree, left as usize, PrefixCode::update(code, false), table);
        }

        if let Some(right) = node.right {
            Self::build_impl(tree, right as usize, PrefixCode::update(code, true), table);
        }
    }

    pub fn canonicalize(&mut self) {
        // Same implementation as in RFC1951 (but MAX_BITS is extended a little bit)
        let mut next_code: [u32; 32] = [0; 32];

        let mut code = 0;
        for (idx, count) in self.lengths_count[1..].iter().enumerate() {
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
        let mut table = HuffmanTable {
            codes: Vec::new(),
            lengths_count: [0; 32],
        };
        // Is there a better way to directly initialize the vector in the above line?
        table.codes.resize(tree.num_symbols, PrefixCode::default());

        Self::build_impl(
            tree,
            tree.nodes.len() - 1,
            PrefixCode::default(),
            &mut table,
        );

        for code in &table.codes {
            table.lengths_count[code.length as usize] += 1;
        }

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
                freq: 0,
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

                            nodes[crawler_idx].left = Some(node_idx as u32);
                            crawler_idx = node_idx;
                        }
                        Some(idx) => {
                            assert!(bit_idx != code.length);
                            crawler_idx = idx as usize;
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

                            nodes[crawler_idx].right = Some(node_idx as u32);
                            crawler_idx = node_idx;
                        }
                        Some(idx) => {
                            assert!(bit_idx != code.length);
                            crawler_idx = idx as usize;
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
