pub mod huffman;
use std::env;
use std::fs;

use huffman::Huffman;

fn read_input_file() -> String {
    let args: Vec<String> = env::args().collect();
    let file_path = &args[1];

    println!("In file {}", file_path);

    let contents = fs::read_to_string(file_path).expect("Should have been able to read the file");

    return contents;
}

fn main() {
    let contents = read_input_file();
    
    let huffman_encoding = Huffman{};
    huffman_encoding.encode(contents);

}
