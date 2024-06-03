pub mod huffman;
use std::collections::BinaryHeap;

fn main() {
    println!("Hello, world!");

    let mut pq = BinaryHeap::new();

    pq.push(huffman::Node{ character:'A', freq: 8});
    pq.push(huffman::Node{ character:'2', freq: 9});
    pq.push(huffman::Node{ character:'X', freq: 89});
    pq.push(huffman::Node{ character:'T', freq: 23});
    pq.push(huffman::Node{ character:'S', freq: 10});
    

    while let Some(node) = pq.pop() {
        println!("freq: {}, char: {}", node.freq, node.character);
    }
    
}
