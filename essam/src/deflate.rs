use crate::bitio::{BitReader, BitWriter};
use crate::huffman::{HuffmanTable, HuffmanTree};
use std::io::{Read, Seek, Write};

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

const MAX_CODE_LENGTH: u32 = 15;

pub fn compress(reader: impl Read + Seek, writer: impl Write) -> std::io::Result<()> {
    let mut bit_reader = BitReader::new(reader);
    let mut bit_writer = BitWriter::new(writer);

    let stats = create_stats(&mut bit_reader)?;
    bit_reader.rewind()?;

    let tree = HuffmanTree::build(&stats, MAX_CODE_LENGTH);
    let mut table = HuffmanTable::from(&tree);
    table.canonicalize();

    let eof_symbol = table.code(EOF);

    // TODO: Segment the file into multiple blocks.
    bit_writer.write_bits(0b101, 3)?; // Write BFINAL and BTYPE

    write_huffman_table(&mut bit_writer, &table)?;

    let mut byte = [0; 1];
    while let Ok(num_read_bytes) = bit_reader.read(&mut byte) {
        if num_read_bytes == 0 {
            break;
        }
        let code = &table.code(byte[0] as usize);
        bit_writer.write_bits(code.code.into(), code.length.into())?;
    }
    bit_writer.write_bits(eof_symbol.code.into(), eof_symbol.length.into())?;

    bit_writer.flush()?;

    Ok(())
}

pub fn decompress(reader: impl Read + Seek, writer: impl Write) -> std::io::Result<()> {
    let mut buf_reader = BitReader::new(reader);
    let mut buf_writer = BitWriter::new(writer);

    // TODO
    buf_reader.read_bits(3)?; // Write BFINAL and BTYPE

    let table = read_huffman_table(&mut buf_reader)?;
    let tree = HuffmanTree::from(&table);

    let mut iter = tree.create_walk_iter();

    while let Ok(bit) = buf_reader.read_bits(1) {
        iter = tree.walk(iter, bit != 0);
        if iter.leaf {
            let symbol = iter.idx;

            if symbol == EOF {
                break;
            }

            buf_writer.write(&symbol.to_le_bytes()[0..1])?;
            iter = tree.create_walk_iter();
        }
    }

    Ok(())
}

fn create_stats(bit_reader: &mut impl Read) -> std::io::Result<Vec<u32>> {
    let mut stats = Vec::new();
    stats.resize(257, 0);
    stats[EOF] = 1;

    let mut buffer = [0; 256];

    while let Ok(num_read_bytes) = bit_reader.read(&mut buffer) {
        for byte in &buffer[0..num_read_bytes] {
            stats[*byte as usize] += 1;
        }

        if num_read_bytes == 0 {
            break;
        }
    }

    Ok(stats)
}

fn write_huffman_table<W: Write>(
    writer: &mut BitWriter<W>,
    huffman_table: &HuffmanTable,
) -> std::io::Result<()> {
    // Write HLIT (number of literals - 257)
    writer.write_bits(0, 5)?;
    // Write HDIST (number of distant codes - 1)
    writer.write_bits(0, 5)?;
    // Write HCLEN (number of code length codes - 4)
    writer.write_bits(15, 4)?;

    let mut lengths_freqs: [u32; 19] = [0; 19];

    let mut symbols = Vec::<u16>::new();
    symbols.reserve(huffman_table.codes.len());

    let mut i: usize = 0;
    while i < huffman_table.codes.len() {
        let code = &huffman_table.code(i);

        // Check if the length is repeated
        let mut j = i + 1;
        while j < huffman_table.codes.len() && huffman_table.codes[j].length == code.length {
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

    // For the code length of the distance alphabet
    lengths_freqs[0] += 1;

    let length_huffman_table = {
        let mut table = HuffmanTable::from(&HuffmanTree::build(&lengths_freqs, MAX_CODE_LENGTH));
        table.canonicalize();
        table
    };

    // Write code lengths for the code lengths alphabet
    for idx in LENGTH_ORDER {
        writer.write_bits(length_huffman_table.codes[idx].length as u64, 3)?;
    }

    // Write code lengths for the literal/length alphabet.
    let mut i = 0;
    while i < symbols.len() {
        let symbol = symbols[i];
        let code = length_huffman_table.code(symbol as usize);
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

    // Write code lengths for the distance alphabet.
    let zero_length_code = length_huffman_table.code(0);
    writer.write_bits(
        zero_length_code.code as u64,
        zero_length_code.length as usize,
    )?;

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

    let length_huffman_table = HuffmanTable::from_lengths(&lengths);
    let length_huffman_tree = HuffmanTree::from(&length_huffman_table);

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
