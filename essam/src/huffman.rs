use std::collections::binary_heap::BinaryHeap;
use thiserror::Error;

pub struct HuffmanTree {
    nodes: Vec<Node>,
    num_symbols: usize,
}

pub struct HuffmanTable {
    codes: Vec<PrefixCode>,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct PrefixCode {
    code: u64,
    length: u8,
}

#[derive(Debug, Clone, Copy)]
pub struct WalkIterator {
    code: PrefixCode,
    idx: u32,
}

#[derive(Debug, Error)]
pub enum WalkError {
    #[error("Trying to iterate past a leaf node")]
    PastTheEnd,

    #[error("Trying to iterate using an invalid index. This should never happen!")]
    InvalidIndex,
}

struct Node {
    freq: u32,
    left: Option<u32>,
    right: Option<u32>,
    parent: Option<u32>, // We might not need it
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct HeapEntry {
    freq: u32,
    idx: u32,
}

impl PrefixCode {
    fn update(prefix_code: PrefixCode, bit: bool) -> PrefixCode {
        PrefixCode {
            code: (prefix_code.code << 1) | (bit as u64),
            length: prefix_code.length + 1,
        }
    }
}

impl HuffmanTree {
    pub fn build(freqs: Vec<u32>) -> HuffmanTree {
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
                parent: None,
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
                right: Some(entry1.0.idx),
                parent: None,
            };

            let internal_node_idx = nodes.len() as u32;
            heap.push(std::cmp::Reverse(HeapEntry {
                freq: internal_node.freq,
                idx: internal_node_idx,
            }));
            nodes.push(internal_node);

            // Set the parent of the least two nodes to the new created internal node.
            nodes[node1_idx].parent = Some(internal_node_idx);
            nodes[node2_idx].parent = Some(internal_node_idx);
        }

        HuffmanTree { nodes, num_symbols }
    }

    fn is_leaf_node(&self, idx: u32) -> bool {
        (idx as usize) < self.num_symbols
    }

    pub fn create_walk_iter(&self) -> WalkIterator {
        WalkIterator {
            idx: (self.nodes.len() - 1) as u32,
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
                self.nodes[iter.idx as usize].right.unwrap()
            } else {
                self.nodes[iter.idx as usize].left.unwrap()
            },
        })
    }
}

impl HuffmanTable {

}
