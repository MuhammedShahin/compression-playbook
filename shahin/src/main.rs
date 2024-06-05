pub mod huffman;
use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::env;
use std::fs;

use huffman::Node;

fn main() {

    let args: Vec<String> = env::args().collect();
    let file_path = &args[1];

    println!("In file {}", file_path);

    let contents = fs::read_to_string(file_path)
        .expect("Should have been able to read the file");

    println!("With text:\n{contents}");

    let mut symboles_freq = HashMap::new();

    for character in contents.chars() {
        let count = symboles_freq.entry(character).or_insert(0);
        *count += 1;
    }
    
    let mut pq = BinaryHeap::new();
    for (key, value) in &symboles_freq {
        pq.push(huffman::Node { character: *key, freq: *value });
    }
    
    while let Some(node) = pq.pop() {
        println!("freq: {}, char: {}", node.freq, node.character);
    }

}


fn build_huffman_tree(symbols: BinaryHeap<Node>)  {
    /*
    * pop the first 2 nodes from the queue
    *
    * create parnt node with the sum of the 2 childs nodes
    *
    * insert the parnt node in the queue
    *
    * repeat until you have one node inside the queue
    */
}
