pub mod bitio;
pub mod huffman;
use std::env;
use std::fs;
use std::io;

use huffman::Huffman;

fn read_input_file() -> String {
    let args: Vec<String> = env::args().collect();
    let file_path = &args[1];

    println!("In file {}", file_path);

    let contents = fs::read_to_string(file_path).expect("Should have been able to read the file");

    return contents;
}

fn test_write_bits() -> io::Result<()> {

    let file = fs::File::create("output.txt")?;
    let mut bit_writer = bitio::BitWriter::new(file);

    bit_writer.write_bits('A' as u8 , 8)?;
    bit_writer.flush_buffer()?;

    Ok(())
}

fn test_read_bits() -> io::Result<()> {

    let file = fs::File::open("output.txt")?;
    let mut bit_reader = bitio::BitReader::new(file);

    if let Some(bits) = bit_reader.read_bits(8)? {
        println!("bits: {:?}", bits as char);
    }

    Ok(())
}

fn main() -> io::Result<()> {
    let contents = read_input_file();
    let huffman_encoding = Huffman{};
    let _ = huffman_encoding.encode(contents);

    let _ = huffman_encoding.decode();

    Ok(())
}
