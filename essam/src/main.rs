use byteorder::{ByteOrder, LittleEndian, ReadBytesExt, WriteBytesExt};
use clap::Parser;
use essam::huffman::{HuffmanTable, HuffmanTree, PrefixCode};
use std::fs::File;
use std::io::Seek;
use std::io::{BufReader, BufWriter};
use std::io::{Read, Write};

#[derive(Debug, Clone, clap::Args)]
struct OperationArgs {
    input_path: String,
    output_path: String,
}

#[derive(Debug, Clone, clap::Subcommand)]
enum Operation {
    Compress(OperationArgs),
    Decompress(OperationArgs),
}

#[derive(Debug, clap::Parser)]
struct Args {
    #[command(subcommand)]
    op: Operation,
}

const EOF: usize = 256;

fn create_stats(buf_reader: &mut impl Read) -> anyhow::Result<Vec<u32>> {
    let mut stats = Vec::new();
    stats.resize(257, 0);
    stats[EOF] = 1;

    let mut buffer = [0; 256];

    while let Ok(num_read_bytes) = buf_reader.read(&mut buffer) {
        for byte in &buffer[0..num_read_bytes] {
            stats[*byte as usize] += 1;
        }

        if num_read_bytes == 0 {
            break;
        }
    }

    Ok(stats)
}

fn write_huffman_table(
    writer: &mut impl Write,
    huffman_table: &HuffmanTable,
) -> anyhow::Result<()> {
    writer.write_u16::<LittleEndian>(huffman_table.codes.len() as u16)?;

    // This can be more efficient by writing only the required bits.
    for code in huffman_table.codes.iter() {
        writer.write_u8(code.length)?;
        writer.write_u32::<LittleEndian>(code.code)?;
    }

    Ok(())
}

fn read_huffman_table(reader: &mut impl Read) -> anyhow::Result<HuffmanTable> {
    let mut table = HuffmanTable::default();

    let table_len = reader.read_u16::<LittleEndian>()? as usize;
    table.codes.resize(table_len, PrefixCode::default());

    for code in table.codes.iter_mut() {
        code.length = reader.read_u8()?;
        code.code = reader.read_u32::<LittleEndian>()?;
    }

    Ok(table)
}

fn compress(input_path: String, output_path: String) -> anyhow::Result<()> {
    let input_file = File::open(input_path)?;
    let output_file = File::create(output_path)?;

    let mut buf_reader = BufReader::new(input_file);
    let mut buf_writer = BufWriter::new(output_file);

    let stats = create_stats(&mut buf_reader)?;
    buf_reader.rewind()?;

    let tree = HuffmanTree::build(&stats);
    let table = HuffmanTable::from(&tree);

    write_huffman_table(&mut buf_writer, &table)?;

    let mut byte = [0; 1];

    let mut pending: u32 = 0;
    let mut pending_length: u8 = 0;

    let mut write_symbol = |symbol: usize| -> anyhow::Result<()> {
        let prefix_code = table.code(symbol);

        if pending_length + prefix_code.length < 32 {
            pending = pending | (prefix_code.code << pending_length);
            pending_length = pending_length + prefix_code.length;
        } else {
            let to_write = pending | (prefix_code.code.overflowing_shl(pending_length as u32)).0;
            buf_writer.write_u32::<LittleEndian>(to_write)?;

            pending_length = (pending_length + prefix_code.length) - 32;
            pending = prefix_code.code >> (prefix_code.length - pending_length);
        }

        Ok(())
    };

    while let Ok(num_read_bytes) = buf_reader.read(&mut byte) {
        if num_read_bytes == 0 {
            break;
        }
        write_symbol(byte[0] as usize)?;
    }
    write_symbol(EOF)?;

    // Write the last few
    if pending_length != 0 {
        let mut bytes = [0; 4];
        LittleEndian::write_u32(&mut bytes, pending);

        let num_bytes = ((pending_length + 7) / 8) as usize;
        buf_writer.write(&bytes[0..num_bytes])?;
    }

    Ok(())
}

fn match_prefix_code(code: u32, table: &HuffmanTable) -> Option<usize> {
    for (idx, prefix_code) in table.codes.iter().enumerate() {
        let (mask, _) = (!(0 as u32)).overflowing_shr(32 - prefix_code.length as u32);
        if (code & mask) == prefix_code.code {
            return Some(idx);
        }
    }

    None
}

fn decompress(input_path: String, output_path: String) -> anyhow::Result<()> {
    let input_file = File::open(input_path)?;
    let output_file = File::create(output_path)?;

    let mut buf_reader = BufReader::new(input_file);
    let mut buf_writer = BufWriter::new(output_file);

    let table = read_huffman_table(&mut buf_reader)?;

    let mut pending: u64 = 0;
    let mut pending_length: u8 = 0;

    let mut byte = [0; 1];
    let mut done = false;

    while let Ok(num_read_bytes) = buf_reader.read(&mut byte) {
        if num_read_bytes == 0 {
            break;
        }

        pending = pending | ((byte[0] as u64) << pending_length);
        pending_length = pending_length + 8;

        while let Some(symbol_idx) = match_prefix_code(pending as u32, &table) {
            if table.codes[symbol_idx].length > pending_length {
                break;
            }
            if symbol_idx == EOF {
                done = true;
                break;
            }

            buf_writer.write_u8(symbol_idx as u8)?;

            pending_length -= table.codes[symbol_idx].length;
            pending = pending >> table.codes[symbol_idx].length;
        }

        if done {
            break;
        }
    }

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    match args.op {
        Operation::Compress(args) => compress(args.input_path, args.output_path),
        Operation::Decompress(args) => decompress(args.input_path, args.output_path),
    }
}
