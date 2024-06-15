use crate::deflate::{
    compress as deflate_compress, decompress as deflate_decompress, DeflateOptions,
};
use crc::{Crc, CRC_32_ISO_HDLC};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Read, Seek, Write};
use std::path::Path;

pub fn compress(input_path: String, output_path: String) -> std::io::Result<()> {
    let input_file = File::open(&input_path)?;
    let output_file = File::create(&output_path)?;

    let mut buf_reader = BufReader::new(input_file);
    let mut buf_writer = BufWriter::new(output_file);

    const ID: u16 = 0x8b1f;
    const DEFLATE_CM: u8 = 8;

    buf_writer.write(&ID.to_le_bytes())?;
    buf_writer.write(&DEFLATE_CM.to_le_bytes())?;

    // TODO
    let flags: u8 = 0b00001000;
    buf_writer.write(&flags.to_le_bytes())?;

    // TODO
    let mtime: u32 = 0;
    buf_writer.write(&mtime.to_le_bytes())?;

    // TODO
    let xfl: u8 = 4;
    buf_writer.write(&xfl.to_le_bytes())?;

    // TODO
    let os: u8 = 255;
    buf_writer.write(&os.to_le_bytes())?;

    // YUCK FIXME
    let filename = Path::new(&input_path)
        .file_name()
        .unwrap()
        .to_str()
        .unwrap();
    buf_writer.write(filename.as_bytes())?;
    buf_writer.write(&(0 as u8).to_le_bytes())?; // Write null terminator

    deflate_compress(&mut buf_reader, &mut buf_writer, DeflateOptions::default())?;

    // FIXME: This is inefficient. Maybe calculate the crc while we're compressing using deflate.
    buf_reader.rewind()?;

    let (crc, size) = compute_crc_and_size(&mut buf_reader);
    buf_writer.write(&crc.to_le_bytes())?;
    buf_writer.write(&size.to_le_bytes())?;

    buf_writer.flush()
}

pub fn decompress(input_path: String, output_path: String) -> std::io::Result<()> {
    const FHCRC_MASK: u8 = 0b00000010;
    const FEXTRA_MASK: u8 = 0b00000100;
    const FNAME_MASK: u8 = 0b00001000;
    const FCOMMENT_MASK: u8 = 0b00010000;

    let input_file = File::open(&input_path)?;
    let output_file = File::create(&output_path)?;

    let mut buf_reader = BufReader::new(input_file);
    let mut buf_writer = BufWriter::new(output_file);

    // FIXME
    let mut buffer: [u8; 10] = [0; 10];

    // Read id, flags, modification time, extra flags, and os
    buf_reader.read_exact(&mut buffer[0..10])?;

    assert!(buffer[0] == 0x1f);
    assert!(buffer[1] == 0x8b);

    let flags = buffer[3];

    // FIXME
    if flags & FEXTRA_MASK != 0 {
        buf_reader.read_exact(&mut buffer[0..2])?;
        let xlen = u16::from_le_bytes([buffer[0], buffer[1]]);

        // Ignore extra field.
        buf_reader.seek_relative(xlen as i64)?;
    }

    // FIXME
    if flags & FNAME_MASK != 0 {
        // Read file name
        let mut name = Vec::new();
        buf_reader.read_until(0, &mut name)?;
    }

    // FIXME
    if flags & FCOMMENT_MASK != 0 {
        // Read comment
        let mut comment = Vec::new();
        buf_reader.read_until(0, &mut comment)?;
    }

    // FIXME
    if flags & FHCRC_MASK != 0 {
        // Skip CRC
        buf_reader.seek_relative(2)?;
    }

    deflate_decompress(&mut buf_reader, &mut buf_writer)?;

    buf_reader.read_exact(&mut buffer[0..8])?;

    // TODO
    // let crc = u32::from_le_bytes(buffer[0..4].try_into().unwrap());
    // let size = u32::from_le_bytes(buffer[4..8].try_into().unwrap());

    Ok(())
}

fn compute_crc_and_size(reader: &mut impl Read) -> (u32, u32) {
    let crc_obj = Crc::<u32>::new(&CRC_32_ISO_HDLC);
    let mut digest = crc_obj.digest();

    let mut tot_size = 0;

    let mut buffer: [u8; 512] = [0; 512];
    while let Ok(read_bytes) = reader.read(&mut buffer) {
        if read_bytes == 0 {
            break;
        }
        tot_size += read_bytes;
        digest.update(&buffer[0..read_bytes]);
    }

    (digest.finalize(), tot_size as u32)
}
