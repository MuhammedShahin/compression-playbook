use std::io::{self, Read, Write};


pub(crate) struct BitWriter<W: Write> {
    writer: W,
    buffer: u8,
    buffer_lenght: u8
}

impl<W: Write> BitWriter<W> {
    pub fn new(writer: W) -> Self {
        return BitWriter {
            writer,
            buffer: 0,
            buffer_lenght: 0
        }
    }

    pub fn write_bit(&mut self, bit: u8) -> io::Result<()> {
        self.buffer = (self.buffer << 1) | (bit & 1);
        self.buffer_lenght += 1;

        if self.buffer_lenght == 8 {
            self.flush_buffer()?;
        }
        
        Ok(())
    }

    pub fn flush_buffer(&mut self) -> io::Result<()> {
        if self.buffer_lenght > 0 {
            self.buffer <<= 8 - self.buffer_lenght;
            self.writer.write_all(&[self.buffer])?;
            self.buffer = 0;
            self.buffer_lenght = 0;
        }

        Ok(())
    }

    pub fn write_bits(&mut self, bits: u8, num_bits: u8) -> io::Result<()> {
        for i in (0..num_bits).rev() {
            self.write_bit((bits >> i) & 1)?;
        }

        Ok(())
    }
}

impl<W: Write> Drop for BitWriter<W> {
    fn drop(&mut self) {
        let _ = self.flush_buffer();
    }
}


pub(crate) struct BitReader<R: Read> {
    reader: R,
    buffer: u8,
    buffer_length: u8
}

impl<R: Read> BitReader<R> {
    pub fn new(reader: R) -> Self {
        return BitReader { 
            reader,
            buffer: 0,
            buffer_length: 0
        }
    } 

    pub fn read_bit(&mut self) -> io::Result<Option<u8>> {
        if self.buffer_length == 0 {
            let mut byte = [0];
            let bytes_read = self.reader.read(&mut byte)?;

            if bytes_read == 0 {
                return Ok(None);
            }

            self.buffer = byte[0];
            self.buffer_length = 8;
        }

        self.buffer_length -= 1;

        Ok(Some((self.buffer >> self.buffer_length) & 1))
    }

    pub fn read_bits(&mut self, num_bits: u8) -> io::Result<Option<u8>> {
        let mut result = 0;

        for _ in 0..num_bits {
            match self.read_bit()? {
                Some(bit) => result = (result << 1) | bit,
                None => return Ok(None),
            }
        }
        Ok(Some(result))
    }
}
