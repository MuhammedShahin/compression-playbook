use std::collections::binary_heap::BinaryHeap;
use thiserror::Error;

pub struct HuffmanTree {
    nodes: Vec<Node>,
    num_symbols: usize,
}

#[derive(Debug, Default)]
pub struct HuffmanTable {
    pub codes: Vec<PrefixCode>,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct PrefixCode {
    pub code: u32,
    pub length: u8,
}

#[derive(Debug, Clone, Copy)]
pub struct WalkIterator {
    code: PrefixCode,
    idx: usize,
}

#[derive(Debug, Error)]
pub enum WalkError {
    #[error("Trying to iterate past a leaf node")]
    PastTheEnd,

    #[error("Trying to iterate using an invalid index. This should never happen!")]
    InvalidIndex,
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
            "PrefixCode {{ code: {:b}, length: {} }}",
            self.code, self.length
        )
    }
}

impl HuffmanTree {
    pub fn build(freqs: &Vec<u32>) -> HuffmanTree {
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

            heap.push(std::cmp::Reverse(HeapEntry {
                freq: *freq,
                idx: idx as u32,
            }));
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
        }
    }

    pub fn walk(&self, iter: WalkIterator, bit: bool) -> Result<WalkIterator, WalkError> {
        if self.is_leaf_node(iter.idx) {
            return Err(WalkError::PastTheEnd);
        }

        Ok(WalkIterator {
            code: PrefixCode::update(iter.code, bit),
            idx: if bit {
                self.nodes[iter.idx].right.unwrap() as usize
            } else {
                self.nodes[iter.idx].left.unwrap() as usize
            },
        })
    }
}

impl HuffmanTable {
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

// TODO
#[allow(unused_variables)]
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
        let alloc_idx = nodes.len() - 2;

        for code in table.codes.iter() {
            let crawler_idx = root_idx;

            for bit_idx in 0..code.length {}
        }

        HuffmanTree { nodes, num_symbols }
    }
}
