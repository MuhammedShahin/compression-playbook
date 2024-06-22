use std::{cmp::Ordering, collections::{BinaryHeap, HashMap}, io::Write, usize};
use std::fs;
use std::io;

use crate::bitio;

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

    pub fn encode(&self, contents: String) -> io::Result<()>{

        let symboles_queue = self.analyz_symboles(&contents);

        let mut huffman_lookup: [PrefixCode; 256] = [PrefixCode {code: 0, length: 0}; 256];
        if let Some(huffman_tree) = self.build_huffman_tree(symboles_queue) {
            self.build_huffman_lookup(&huffman_tree, PrefixCode {code: 0, length: 0}, &mut huffman_lookup);
            println!("Encoded Tree:");
            self.print_tree(&huffman_tree, "", true);
        }

        let file = fs::File::create("output.txt")?;
        let mut bit_writer = bitio::BitWriter::new(file);

        for i in 0..256 {
            // println!("character: {:?}, prefix: {:08b}, length: {}", (i as u8) as char, huffman_lookup[i].code, huffman_lookup[i].length);
            bit_writer.write_bits(huffman_lookup[i].length, 8)?;
            bit_writer.write_bits(huffman_lookup[i].code, huffman_lookup[i].length)?;
        }

        for symbols in contents.into_bytes() {
            bit_writer.write_bits(huffman_lookup[symbols as usize].code, huffman_lookup[symbols as usize].length)?;
        }

        bit_writer.flush_buffer()?;

        Ok(())

    }

    pub fn decode(&self) -> io::Result<()> {
        let file = fs::File::open("output.txt")?;
        let mut bit_reader = bitio::BitReader::new(file);

        let mut huffman_tree = Node {
            symbole: None,
            freq: 0,
            left_node: None,
            right_node: None
        };

        let mut huffman_lookup: [PrefixCode; 256] = [PrefixCode {code: 0, length: 0}; 256];

        for i in 0..256 {
            if let Some(length) = bit_reader.read_bits(8)? {
                if let Some(code) = bit_reader.read_bits(length)? {
                    huffman_lookup[i] = PrefixCode {code, length};
                    
                    if length != 0 {
                        self.insert_leaf(&mut huffman_tree, huffman_lookup[i], i as u8);
                    }
                }
            }
        }

        println!("Decoded Tree:");
        self.print_tree(&huffman_tree, "", true);

        let mut walking_node = &huffman_tree;
        let mut content: String = String::new();
        loop {
            if let Some(bit) = bit_reader.read_bit()? {
                if walking_node.right_node.is_none() && walking_node.left_node.is_none() {
                    if let Some(value) = walking_node.symbole {
                        content.push(value as char);
                        walking_node = &huffman_tree;
                    }
                }
                else if bit == 1 {
                    if let Some(ref right_node) = walking_node.right_node {
                        walking_node = right_node;
                    }
                }
                else {
                    if let Some(ref left_node) = walking_node.left_node {
                        walking_node = left_node;
                    }
                }
            }
            else {
                break;
            }
        }

        let mut decoded_file = fs::File::create("output1.txt")?;

        decoded_file.write_all(content.as_bytes())?;

        Ok(())
    }
    
    fn analyz_symboles(&self, contents: &String) -> BinaryHeap<Node> {
        let mut symboles_freq = HashMap::new();

        for character in contents.clone().into_bytes() {
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

    fn build_huffman_lookup(&self, node: &Node, prefix: PrefixCode, hufmman_lookup: &mut [PrefixCode; 256]) {
        if let Some(symbole) = node.symbole {
            hufmman_lookup[symbole as usize] = prefix;
        } else {
            if let Some(ref left_node) = node.left_node {
                self.build_huffman_lookup(left_node, PrefixCode::update(prefix, 0), hufmman_lookup);
            }
            if let Some(ref right_node) = node.right_node {
                self.build_huffman_lookup(right_node, PrefixCode::update(prefix, 1), hufmman_lookup);
            }
        }
    }

    fn insert_leaf(&self, root: &mut Node, prefix: PrefixCode, symbole: u8) {
        let mut currnet_node = root;

        for i in (0..prefix.length).rev() {
            let bit = (prefix.code >> i) & 1;

            if bit == 1 {
                if currnet_node.right_node.is_none() {
                    currnet_node.right_node = Some(Box::new(Node {
                        freq: 0,
                        symbole: None,
                        left_node: None,
                        right_node: None
                    }))
                }

                if let Some(ref mut right_node) = currnet_node.right_node {
                    currnet_node = right_node;
                }
            }
            else {
                if currnet_node.left_node.is_none() {
                    currnet_node.left_node = Some(Box::new(Node {
                        freq: 0,
                        symbole: None,
                        left_node: None,
                        right_node: None
                    }))
                }

                if let Some(ref mut left_node) = currnet_node.left_node {
                    currnet_node = left_node;
                }
            }
        }
        *currnet_node = Node {
            symbole: Some(symbole),
            freq: 0,
            left_node: None,
            right_node: None
        }
    }

    fn print_tree(&self, node: &Node, prefix: &str, is_left: bool) {
        if let Some(symbol) = node.symbole {
            println!("{}{}- {:?}", prefix, if is_left { "├──" } else { "└──" }, symbol as char);
        } else {
            println!("{}{}", prefix, if is_left { "├──" } else { "└──" });

            if let Some(ref left) = node.left_node {
                self.print_tree(left, &format!("{}{}", prefix, if is_left { "│   " } else { "    " }), true);
            }
            if let Some(ref right) = node.right_node {
                self.print_tree(right, &format!("{}{}", prefix, if is_left { "│   " } else { "    " }), false);
            }
        }
    }
}
