use std::io::{self, BufRead, BufReader, Read, Seek};

use crate::{RevBufRead, RevRead};

use self::buffer::Buffer;

mod buffer;

// Bare metal platforms usually have very small amounts of RAM
// (in the order of hundreds of KB)
pub const DEFAULT_BUF_SIZE: usize = if cfg!(target_os = "espidf") {
    512
} else {
    8 * 1024
};

/// # Use case
/// Use this struct, if:
///   - you read back and forth in a limited section
///
/// # Non use case
/// Don't use this struct, if:
///   - you are reading a lot in only one direction (either back or forth). Use [`std::io::BufReader`] or [RevBufReader] for this
///     since they will buffer more from their reading direction
pub struct BiBufReader<R> {
    buf: Buffer,
    inner: R,
}

impl<R> BiBufReader<R> {
    pub fn capacity(&self) -> usize {
        self.buf.capacity()
    }

    pub fn get_mut(&mut self) -> &mut R {
        &mut self.inner
    }

    pub fn get_ref(&self) -> &R {
        &self.inner
    }

    pub fn into_inner(self) -> R
    where
        R: Sized,
    {
        self.inner
    }

    pub fn new(inner: R) -> Self {
        Self::with_capacity(DEFAULT_BUF_SIZE, inner)
    }

    pub fn with_capacity(capacity: usize, inner: R) -> Self {
        Self {
            buf: Buffer::with_capacity(capacity),
            inner,
        }
    }
}

impl<R: Seek> BiBufReader<R> {
    pub fn seek_relative(&mut self, offset: i64) -> io::Result<()> {
        todo!()
    }
}

impl<R: Read> Read for BiBufReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let nothing_buffered = self.buf.pos() == self.buf.filled();
        let buf_exceeds_internal_buffer = buf.len() >= self.capacity();

        if nothing_buffered && buf_exceeds_internal_buffer {
            self.buf.discard_buffer();
            return self.inner.read(buf);
        }

        let mut added_content = self.fill_buf()?;
        let amount_read = added_content.read(buf)?;
        Ok(amount_read)
    }
}

impl<R: Read> BufRead for BiBufReader<R> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        self.buf.fill_buf(&mut self.inner)
    }

    fn consume(&mut self, amt: usize) {
        self.buf.consume(amt);
    }
}

impl<R: Seek> RevRead for BiBufReader<R> {
    fn rev_read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        todo!()
    }
}

impl<R: Seek> RevBufRead for BiBufReader<R> {
    fn rev_fill_buf(&mut self) -> io::Result<&[u8]> {
        todo!()
    }

    fn rev_consume(&mut self, amt: usize) {
        todo!()
    }
}

impl<R: Seek> Seek for BiBufReader<R> {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DATA: [u8; 10] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];

    #[test]
    fn read() {
        let inner = io::Cursor::new(&DATA);
        let mut reader = BiBufReader::new(inner);
        let mut buffer = [0, 0, 0];

        assert_eq!(reader.read(&mut buffer).ok(), Some(3));
        assert_eq!(buffer, [0, 1, 2]);
    }

    #[test]
    fn rev_read() {
        let inner = io::Cursor::new(&DATA);
        let mut reader = BiBufReader::new(inner);
        reader.seek(io::SeekFrom::End(0)).unwrap();
        let mut buffer = [0, 0, 0];

        assert_eq!(reader.rev_read(&mut buffer).ok(), Some(3));
        assert_eq!(buffer, [7, 8, 9]);
    }
}
