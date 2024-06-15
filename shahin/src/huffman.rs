use std::{cmp::Ordering, collections::{BinaryHeap, HashMap}};

#[derive(Eq, PartialEq)]
pub struct Node {
    character: Option<char>,
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


pub struct Huffman {

}


impl Huffman {

    pub fn encode(&self, contents: String) {

        let symboles_queue = self.analyz_symboles(contents);

        let mut huffman_lookup = HashMap::new();
        if let Some(huffman_tree) = self.build_huffman_tree(symboles_queue) {
            self.build_huffman_lookup(&huffman_tree, String::new(), &mut huffman_lookup);
        }
        for (character, prefix) in huffman_lookup {
            println!("character: {:?}, prefix: {}, length: {}", character, prefix, prefix.len());
        }

    }
    
    fn analyz_symboles(&self, contents: String) -> BinaryHeap<Node> {
        let mut symboles_freq = HashMap::new();

        for character in contents.chars() {
            let count = symboles_freq.entry(character).or_insert(0);
            *count += 1;
        }

        let mut symboles_queue = BinaryHeap::new();
        for (key, value) in &symboles_freq {
            symboles_queue.push(Node {
                character: Some(*key),
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
                character: None,
                freq: left_node.freq + right_node.freq,
                left_node: Some(Box::new(left_node)),
                right_node: Some(Box::new(right_node)),
            };

            symbols.push(parent_node);
        }

        return symbols.pop();
    }

    fn build_huffman_lookup(&self, node: &Node, prefix: String, hufmman_lookup: &mut HashMap<char, String>) {
        if let Some(character) = node.character {
            hufmman_lookup.insert(character, prefix);
        } else {
            if let Some(ref left_node) = node.left_node {
                self.build_huffman_lookup(left_node, format!("{}0", prefix), hufmman_lookup);
            }
            if let Some(ref right_node) = node.right_node {
                self.build_huffman_lookup(right_node, format!("{}1", prefix), hufmman_lookup);
            }
        }
    }

}
