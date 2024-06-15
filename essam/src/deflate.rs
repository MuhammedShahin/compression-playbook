use crate::bitio::{BitReader, BitWriter};
use crate::huffman::{HuffmanTable, HuffmanTree};
use std::io::{Read, Seek, Write};

const NUM_LITERAL_SYMBOLS: usize = 286;
const NUM_DISTANCE_SYMBOLS: usize = 19;
const EOF: usize = 256;

const REPEAT_PREV_3_6_SYMBOL: u64 = 16;
const REPEAT_PREV_3_6_ARG_LEN: usize = 2;

const REPEAT_0_CODELEN_3_10_SYMBOL: u64 = 17;
const REPEAT_0_CODELEN_3_10_ARG_LEN: usize = 3;

const REPEAT_0_CODELEN_11_138_SYMBOL: u64 = 18;
const REPEAT_0_CODELEN_11_138_ARG_LEN: usize = 7;

const LENGTH_ORDER: [usize; 19] = [
    16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
];

const MAX_CODE_LENGTH: usize = 15;
const MAX_LENGTH_CODE_LENGTH: usize = 7;
const CODE_LENGTH_CODE_LENGTH_LEN: usize = 3; // Absolutely ridiculous

pub struct DeflateOptions {
    pub block_size: usize,
}

struct Block {
    symbols: Vec<u16>,
    literal_freqs: [u32; NUM_LITERAL_SYMBOLS],
    distance_freqs: [u32; NUM_DISTANCE_SYMBOLS],
}

#[derive(Debug)]
struct BlockCompressionInfo {
    num_literal_codes: usize,
    num_distance_codes: usize,
}

impl Default for DeflateOptions {
    fn default() -> Self {
        Self { block_size: 16384 }
    }
}

impl Default for Block {
    fn default() -> Self {
        Self {
            symbols: Vec::new(),
            literal_freqs: [0; NUM_LITERAL_SYMBOLS],
            distance_freqs: [0; NUM_DISTANCE_SYMBOLS],
        }
    }
}

pub fn compress(
    mut reader: impl Read + Seek,
    writer: impl Write,
    options: DeflateOptions,
) -> std::io::Result<()> {
    let mut bit_writer = BitWriter::new(writer);
    let mut block = Block::default();

    loop {
        match compress_block(&mut reader, &mut bit_writer, &mut block, &options) {
            Ok(true) => {
                break;
            }
            Ok(false) => {}
            Err(error) => {
                return Err(error);
            }
        }
    }

    bit_writer.flush()
}

fn compress_block<W: Write>(
    reader: &mut (impl Read + Seek),
    writer: &mut BitWriter<W>,
    block: &mut Block,
    options: &DeflateOptions,
) -> std::io::Result<bool> {
    let info = compress_block_gen_symbols(reader, block, options)?;

    let literal_table = {
        let literal_tree = HuffmanTree::build(
            &block.literal_freqs[0..info.num_literal_codes],
            MAX_CODE_LENGTH,
        );
        let mut literal_table = HuffmanTable::from(&literal_tree);
        literal_table.canonicalize();
        literal_table
    };

    let distance_table = {
        let distance_tree = HuffmanTree::build(
            &block.distance_freqs[0..info.num_distance_codes],
            MAX_CODE_LENGTH,
        );
        let mut distance_table = HuffmanTable::from(&distance_tree);
        distance_table.canonicalize();
        distance_table
    };

    // Check end of file
    // Not the best way to check end of file I guess.
    let mut buf = [0; 1];
    let bfinal = {
        let read_bytes = reader.read(&mut buf)?;
        if read_bytes == 1 {
            reader.seek_relative(-1)?;
            false
        } else {
            true
        }
    };
    writer.write_bits((bfinal as u64) | 0b100, 3)?; // Write BFINAL and BTYPE

    write_huffman_tables(writer, &literal_table, &distance_table, &info)?;

    for symbol in &block.symbols {
        let code = &literal_table.code(*symbol as usize);
        writer.write_bits(code.code.into(), code.length.into())?;
    }

    // Write EOF
    let eof_symbol = literal_table.code(EOF);
    writer.write_bits(eof_symbol.code.into(), eof_symbol.length.into())?;

    Ok(bfinal)
}

pub fn decompress(reader: impl Read + Seek, writer: impl Write) -> std::io::Result<()> {
    let mut bit_reader = BitReader::new(reader);
    let mut bit_writer = BitWriter::new(writer);

    loop {
        match decompress_block(&mut bit_reader, &mut bit_writer) {
            Ok(true) => {
                break;
            }
            Ok(false) => {}
            Err(error) => {
                return Err(error);
            }
        }
    }

    bit_writer.flush()
}

fn decompress_block<R: Read + Seek, W: Write>(
    reader: &mut BitReader<R>,
    writer: &mut BitWriter<W>,
) -> std::io::Result<bool> {
    // Read BFINAL and BTYPE
    let bfinal = reader.read_bits(1)?;
    let btype = reader.read_bits(2)?;

    assert!(btype == 0b10);

    let table = read_huffman_table(reader)?;
    let tree = HuffmanTree::from(&table);

    let mut iter = tree.create_walk_iter();

    loop {
        while !iter.leaf {
            let bit = reader.read_bits(1)? != 0;
            iter = tree.walk(iter, bit);
        }

        let symbol = iter.idx;

        if symbol == EOF {
            break;
        }

        writer.write(&symbol.to_le_bytes()[0..1])?;
        iter = tree.create_walk_iter();
    }

    Ok(bfinal != 0)
}

fn compress_block_gen_symbols(
    bit_reader: &mut impl Read,
    block: &mut Block,
    options: &DeflateOptions,
) -> std::io::Result<BlockCompressionInfo> {
    // Reset block
    block.symbols.clear();
    block.literal_freqs.fill(0);
    block.distance_freqs.fill(0);

    // Single EOF symbol at the last of the block.
    block.literal_freqs[EOF] = 1;

    let mut buffer = [0; 256];
    let mut tot_read_bytes = 0;
    let mut bytes_to_read = buffer.len().min(options.block_size);

    // TODO: Implement LZ77
    loop {
        let num_read_bytes = bit_reader.read(&mut buffer[0..bytes_to_read])?;

        for byte in &buffer[0..num_read_bytes] {
            block.symbols.push(*byte as u16);
            block.literal_freqs[*byte as usize] += 1;
        }

        tot_read_bytes += num_read_bytes;
        let remaining_bytes = options.block_size - tot_read_bytes;

        if num_read_bytes == 0 || remaining_bytes <= 0 {
            break;
        }

        bytes_to_read = buffer.len().min(remaining_bytes);
    }

    assert!(tot_read_bytes > 0);

    Ok(BlockCompressionInfo {
        num_literal_codes: 257,
        num_distance_codes: 1,
    })
}

fn write_huffman_tables<W: Write>(
    writer: &mut BitWriter<W>,
    literal_table: &HuffmanTable,
    distance_table: &HuffmanTable,
    info: &BlockCompressionInfo,
) -> std::io::Result<()> {
    // Write HLIT (number of literals - 257)
    writer.write_bits((info.num_literal_codes - 257) as u64, 5)?;
    // Write HDIST (number of distant codes - 1)
    writer.write_bits((info.num_distance_codes - 1) as u64, 5)?;

    let mut lengths_freqs: [u32; 19] = [0; 19];

    let literal_table_lengths_symbols =
        compress_huffman_table_gen_symbols(literal_table, &mut lengths_freqs);
    let distance_table_lengths_symbols =
        compress_huffman_table_gen_symbols(distance_table, &mut lengths_freqs);

    let num_code_length_codes = {
        let mut result = 4;
        for i in (4..19).rev() {
            if lengths_freqs[LENGTH_ORDER[i]] != 0 {
                result = i + 1;
                break;
            }
        }
        result
    };

    // Write HCLEN (number of code length codes - 4)
    writer.write_bits((num_code_length_codes - 4) as u64, 4)?;

    let length_table = {
        let mut table =
            HuffmanTable::from(&HuffmanTree::build(&lengths_freqs, MAX_LENGTH_CODE_LENGTH));
        table.canonicalize();
        table
    };

    // Write code lengths for the code lengths alphabet
    for idx in 0..num_code_length_codes {
        writer.write_bits(
            length_table.code(LENGTH_ORDER[idx]).length as u64,
            CODE_LENGTH_CODE_LENGTH_LEN,
        )?;
    }

    // Write code lengths for the literal/length alphabet.
    write_huffman_length_symbols(writer, &literal_table_lengths_symbols, &length_table)?;

    // Write code lengths for the distance alphabet.
    write_huffman_length_symbols(writer, &distance_table_lengths_symbols, &length_table)?;

    Ok(())
}

fn compress_huffman_table_gen_symbols(
    table: &HuffmanTable,
    lengths_freqs: &mut [u32; 19],
) -> Vec<u16> {
    if table.codes.is_empty() {
        // This is because HDIST has to be at least 1, so we increment
        // the frequency for the zero symbol so that this singular element
        // has length 0
        lengths_freqs[0] += 1;
        return [0].into();
    }

    let mut symbols = Vec::<u16>::new();
    symbols.reserve(table.codes.len());

    let mut i: usize = 0;
    while i < table.codes.len() {
        let code = &table.code(i);

        // Check if the length is repeated
        let mut j = i + 1;
        while j < table.codes.len() && table.codes[j].length == code.length {
            j += 1;
        }

        // The number of times this length is repeated consecutively
        let mut num_repeated = j - i;

        // If code length is repeated > 3, use specific code for repeated code lengths.
        if code.length == 0 && num_repeated >= 3 {
            if num_repeated <= 10 {
                lengths_freqs[REPEAT_0_CODELEN_3_10_SYMBOL as usize] += 1;

                num_repeated = num_repeated.min(10);
                symbols.push(REPEAT_0_CODELEN_3_10_SYMBOL as u16);
                symbols.push((num_repeated - 3) as u16);
            } else {
                lengths_freqs[REPEAT_0_CODELEN_11_138_SYMBOL as usize] += 1;

                num_repeated = num_repeated.min(138);
                symbols.push(REPEAT_0_CODELEN_11_138_SYMBOL as u16);
                symbols.push((num_repeated - 11) as u16);
            }
        } else {
            // Write the symbol itself
            lengths_freqs[code.length as usize] += 1;
            symbols.push(code.length as u16);

            if num_repeated >= 4 {
                lengths_freqs[REPEAT_PREV_3_6_SYMBOL as usize] += 1;

                num_repeated = num_repeated.min(7);
                symbols.push(REPEAT_PREV_3_6_SYMBOL as u16);
                symbols.push((num_repeated - 4) as u16);
            } else {
                num_repeated = 1;
            }
        }

        // Update i
        i += num_repeated;
    }

    symbols
}

fn write_huffman_length_symbols<W: Write>(
    writer: &mut BitWriter<W>,
    symbols: &Vec<u16>,
    length_table: &HuffmanTable,
) -> std::io::Result<()> {
    // Write code lengths for the literal/length alphabet.
    let mut i = 0;
    while i < symbols.len() {
        let symbol = symbols[i];

        let code = length_table.code(symbol as usize);
        writer.write_bits(code.code as u64, code.length as usize)?;

        match symbol as u64 {
            REPEAT_PREV_3_6_SYMBOL => {
                i += 1;
                writer.write_bits(symbols[i] as u64, REPEAT_PREV_3_6_ARG_LEN)?;
            }
            REPEAT_0_CODELEN_3_10_SYMBOL => {
                i += 1;
                writer.write_bits(symbols[i] as u64, REPEAT_0_CODELEN_3_10_ARG_LEN)?;
            }
            REPEAT_0_CODELEN_11_138_SYMBOL => {
                i += 1;
                writer.write_bits(symbols[i] as u64, REPEAT_0_CODELEN_11_138_ARG_LEN)?;
            }
            _ => {}
        }

        i += 1;
    }

    Ok(())
}

fn read_huffman_table<R: Read>(reader: &mut BitReader<R>) -> std::io::Result<HuffmanTable> {
    let num_literals = reader.read_bits(5)? + 257; // HLIT
    let num_distance_codes = reader.read_bits(5)? + 1; // HDIST
    let num_code_length_codes = reader.read_bits(4)? + 4; // HCLEN

    // Read the table for the alphabet lengths.
    let mut lengths = [0; 19];

    for idx in 0..num_code_length_codes {
        lengths[LENGTH_ORDER[idx as usize]] = reader.read_bits(3)? as u8;
    }

    let length_table = HuffmanTable::from_lengths(&lengths);
    let length_huffman_tree = HuffmanTree::from(&length_table);

    // Read the table for the alphabet.
    let mut lengths = Vec::new();
    lengths.reserve((num_literals) as usize);

    let mut literal_idx = 0;
    while literal_idx < num_literals {
        let mut iter = length_huffman_tree.create_walk_iter();

        while !iter.leaf {
            let bit = reader.read_bits(1)? != 0;
            iter = length_huffman_tree.walk(iter, bit);
        }

        let code_length = iter.idx as u64;
        if code_length < 16 {
            lengths.push(code_length as u8);
            literal_idx += 1;
        } else if code_length == REPEAT_PREV_3_6_SYMBOL {
            let num_repeated = reader.read_bits(REPEAT_PREV_3_6_ARG_LEN)? + 3;
            let prev_length = *lengths.last().unwrap();

            for _ in 0..num_repeated {
                lengths.push(prev_length as u8)
            }

            literal_idx += num_repeated;
        } else if code_length == REPEAT_0_CODELEN_3_10_SYMBOL {
            let num_repeated = reader.read_bits(REPEAT_0_CODELEN_3_10_ARG_LEN)? + 3;
            for _ in 0..num_repeated {
                lengths.push(0)
            }

            literal_idx += num_repeated;
        } else if code_length == REPEAT_0_CODELEN_11_138_SYMBOL {
            let num_repeated = reader.read_bits(REPEAT_0_CODELEN_11_138_ARG_LEN)? + 11;
            for _ in 0..num_repeated {
                lengths.push(0)
            }

            literal_idx += num_repeated;
        }
    }

    // TODO: Not supported yet
    for _ in 0..num_distance_codes {
        let mut iter = length_huffman_tree.create_walk_iter();

        while !iter.leaf {
            let bit = reader.read_bits(1)? != 0;
            iter = length_huffman_tree.walk(iter, bit);
        }

        // TODO
    }

    Ok(HuffmanTable::from_lengths(&lengths))
}
