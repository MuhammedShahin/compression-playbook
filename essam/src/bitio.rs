use std::io::{Read, Seek, Write};

pub struct BitWriter<W: Write> {
    writer: W,
    buffer: u64,
    length: usize,
}

pub struct BitReader<R: Read> {
    reader: R,
    buffer: u64,
    length: usize,
}

impl<W: Write> Write for BitWriter<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.writer.write(buf)
    }

    fn write_vectored(&mut self, bufs: &[std::io::IoSlice<'_>]) -> std::io::Result<usize> {
        self.writer.write_vectored(bufs)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        if self.length > 0 {
            let bytes = self.buffer.to_le_bytes();
            let num_bytes = (self.length + 7) / 8;

            self.write(&bytes[0..num_bytes])?;
        }

        self.writer.flush()
    }

    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        self.writer.write_all(buf)
    }

    fn write_fmt(&mut self, fmt: std::fmt::Arguments<'_>) -> std::io::Result<()> {
        self.writer.write_fmt(fmt)
    }
}

impl<W: Write> BitWriter<W> {
    const BUF_NBITS: usize = 64;

    pub fn new(writer: W) -> Self {
        Self {
            writer,
            buffer: 0,
            length: 0,
        }
    }

    pub fn write_bits(&mut self, data: u64, length: usize) -> std::io::Result<()> {
        assert!(length <= Self::BUF_NBITS);

        if self.length + length < Self::BUF_NBITS {
            self.buffer = self.buffer | (data << self.length);
            self.length += length;
        } else {
            let concatenated_data = self.buffer | data.overflowing_shl(self.length as u32).0;
            self.write(&concatenated_data.to_le_bytes())?;

            self.buffer = data
                .overflowing_shr((Self::BUF_NBITS - self.length) as u32)
                .0;
            self.length = length + self.length - Self::BUF_NBITS;
        }

        Ok(())
    }
}

impl<R: Read> Read for BitReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.reader.read(buf)
    }

    fn read_vectored(&mut self, bufs: &mut [std::io::IoSliceMut<'_>]) -> std::io::Result<usize> {
        self.reader.read_vectored(bufs)
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> std::io::Result<usize> {
        self.reader.read_to_end(buf)
    }

    fn read_to_string(&mut self, buf: &mut String) -> std::io::Result<usize> {
        self.reader.read_to_string(buf)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> std::io::Result<()> {
        self.reader.read_exact(buf)
    }
}

impl<R: Read> BitReader<R> {
    const BUF_NBITS: usize = 64;

    pub fn new(reader: R) -> Self {
        Self {
            reader,
            buffer: 0,
            length: 0,
        }
    }

    pub fn read_bits(&mut self, length: usize) -> std::io::Result<u64> {
        assert!(length <= Self::BUF_NBITS);

        let mask = (!(0 as u64))
            .overflowing_shr((Self::BUF_NBITS - length) as u32)
            .0;

        if length < self.length {
            let return_value = self.buffer & mask;

            self.length -= length;
            self.buffer = self.buffer.overflowing_shr(length as u32).0;

            Ok(return_value)
        } else {
            let mut buffer_arr: [u8; 8] = [0; 8];
            let read_bytes = self.read(&mut buffer_arr)?;

            let buffer = u64::from_le_bytes(buffer_arr);
            let read_bits = 8 * read_bytes;

            if length > self.length + length {
                return Err(std::io::Error::from(std::io::ErrorKind::UnexpectedEof));
            }

            let result = (self.buffer | (buffer.overflowing_shl(self.length as u32).0)) & mask;

            self.buffer = buffer.overflowing_shr((length - self.length) as u32).0;
            self.length = self.length + read_bits - length;

            Ok(result)
        }
    }

    pub fn put_back_extra(&mut self) -> std::io::Result<()>
    where
        R: Seek,
    {
        // ignore the byte we've already taken bits from.
        let nbytes = (self.length / 8) as i64;

        self.length = 0;
        self.buffer = 0;
        self.seek_relative(-nbytes)
    }
}

impl<R: Read + Seek> Seek for BitReader<R> {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        self.buffer = 0;
        self.length = 0;

        self.reader.seek(pos)
    }

    fn rewind(&mut self) -> std::io::Result<()> {
        self.buffer = 0;
        self.length = 0;

        self.reader.rewind()
    }

    fn stream_position(&mut self) -> std::io::Result<u64> {
        self.reader.stream_position()
    }
}
