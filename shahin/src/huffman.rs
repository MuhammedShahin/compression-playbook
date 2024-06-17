use std::{cmp::Ordering, collections::{BinaryHeap, HashMap}};

#[derive(Eq, PartialEq, Clone)]
pub struct Node {
    symbole: Option<u8>,
    freq: u32,
    left_node: Option<Box<Node>>,
    right_node: Option<Box<Node>>,
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other.freq.cmp(&self.freq)
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}


#[derive(Clone, Copy)]
pub struct PrefixCode {
    code: u8,
    length: u8
}

impl PrefixCode {
    pub fn update(prefix_code: PrefixCode, bit: u8) -> PrefixCode {
        if bit == 1 {
            return PrefixCode {
                code: (prefix_code.code << 1) + 1,
                length: prefix_code.length + 1
            }
        }
        else {
            return PrefixCode {
                code: prefix_code.code << 1,
                length: prefix_code.length + 1
            }
        }
    }
}


pub struct Huffman {

}

impl Huffman {

    pub fn encode(&self, contents: String) {

        let symboles_queue = self.analyz_symboles(contents);

        let mut huffman_lookup = HashMap::new();
        if let Some(huffman_tree) = self.build_huffman_tree(symboles_queue) {
            self.build_huffman_lookup(&huffman_tree, PrefixCode {code: 0, length: 0}, &mut huffman_lookup);
        }
        for (character, prefix) in huffman_lookup {
            println!("character: {:?}, prefix: {:08b}, length: {}", character as char, prefix.code, prefix.length);
        }

    }
    
    fn analyz_symboles(&self, contents: String) -> BinaryHeap<Node> {
        let mut symboles_freq = HashMap::new();

        for character in contents.into_bytes() {
            let count = symboles_freq.entry(character).or_insert(0);
            *count += 1;
        }

        let mut symboles_queue = BinaryHeap::new();
        for (key, value) in &symboles_freq {
            symboles_queue.push(Node {
                symbole: Some(*key),
                freq: *value,
                left_node: None,
                right_node: None,
            });
        }

        return symboles_queue;
    }

    fn build_huffman_tree(&self, mut symbols: BinaryHeap<Node>) -> Option<Node> {
        while symbols.len() > 1 {
            let left_node = symbols.pop().unwrap();
            let right_node = symbols.pop().unwrap();

            let parent_node = Node {
                symbole: None,
                freq: left_node.freq + right_node.freq,
                left_node: Some(Box::new(left_node)),
                right_node: Some(Box::new(right_node)),
            };

            symbols.push(parent_node);
        }

        return symbols.pop();
    }

    fn build_huffman_lookup(&self, node: &Node, prefix: PrefixCode, hufmman_lookup: &mut HashMap<u8, PrefixCode>) {
        if let Some(symbole) = node.symbole {
            hufmman_lookup.insert(symbole, prefix);
        } else {
            if let Some(ref left_node) = node.left_node {
                self.build_huffman_lookup(left_node, PrefixCode::update(prefix, 0), hufmman_lookup);
            }
            if let Some(ref right_node) = node.right_node {
                self.build_huffman_lookup(right_node, PrefixCode::update(prefix, 1), hufmman_lookup);
            }
        }
    }

}
