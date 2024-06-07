pub mod huffman;
use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::env;
use std::fs;

use huffman::Node;

fn read_input_file() -> String {
    let args: Vec<String> = env::args().collect();
    let file_path = &args[1];

    println!("In file {}", file_path);

    let contents = fs::read_to_string(file_path).expect("Should have been able to read the file");

    return contents;
}


fn analyz_symboles(contents: String) -> BinaryHeap<Node> {
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


fn build_huffman_tree(mut symbols: BinaryHeap<Node>) -> Option<Node> {
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


fn build_huffman_lookup(node: &Node, prefix: String, hufmman_lookup: &mut HashMap<char, String>) {
    if let Some(character) = node.character {
        hufmman_lookup.insert(character, prefix);
    } else {
        if let Some(ref left_node) = node.left_node {
            build_huffman_lookup(left_node, format!("{}0", prefix), hufmman_lookup);
        }
        if let Some(ref right_node) = node.right_node {
            build_huffman_lookup(right_node, format!("{}1", prefix), hufmman_lookup);
        }
    }
}


fn main() {
    let contents = read_input_file();
    let symboles_queue = analyz_symboles(contents);

    let mut huffman_lookup = HashMap::new();
    if let Some(huffman_tree) = build_huffman_tree(symboles_queue) {
        build_huffman_lookup(&huffman_tree, String::new(), &mut huffman_lookup);
    }
    for (character, prefix) in huffman_lookup {
        println!("character: {:?}, prefix: {}", character, prefix);
    }
}
