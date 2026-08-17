//! Byte I/O utilities for reading binary data from various sources.

use std::io::{Error, ErrorKind, SeekFrom};

pub use std::io::{Read, Result, Seek};

/// A trait that extends the standard `Read` trait with methods for reading integers in little-endian and big-endian formats.
pub trait ByteRead: Read {
    /// Reads a single byte in little-endian format from the input and returns it as a `u8`.
    fn le_u8(&mut self) -> Result<u8> {
        let mut buf = [0u8; 1];
        self.read_exact(&mut buf)?;
        Ok(buf[0])
    }
    /// Reads a single byte in big-endian format from the input and returns it as a `u8`.
    fn be_u8(&mut self) -> Result<u8> {
        self.le_u8()
    }

    /// Reads a 16-bit unsigned integer in little-endian format from the input and returns it as a `u16`.
    fn le_u16(&mut self) -> Result<u16> {
        let mut buf = [0u8; 2];
        self.read_exact(&mut buf)?;
        Ok(u16::from_le_bytes(buf))
    }
    /// Reads a 16-bit unsigned integer in big-endian format from the input and returns it as a `u16`.
    fn be_u16(&mut self) -> Result<u16> {
        let mut buf = [0u8; 2];
        self.read_exact(&mut buf)?;
        Ok(u16::from_be_bytes(buf))
    }

    /// Reads a 24-bit unsigned integer in little-endian format from the input and returns it as a `u32`.
    fn le_u24(&mut self) -> Result<u32> {
        let mut buf = [0u8; 4];
        self.read_exact(&mut buf[0..3])?;
        Ok(u32::from_le_bytes(buf))
    }
    /// Reads a 24-bit unsigned integer in big-endian format from the input and returns it as a `u32`.
    fn be_u24(&mut self) -> Result<u32> {
        let mut buf = [0u8; 4];
        self.read_exact(&mut buf[1..4])?;
        Ok(u32::from_be_bytes(buf))
    }

    /// Reads a 32-bit unsigned integer in little-endian format from the input and returns it as a `u32`.
    fn le_u32(&mut self) -> Result<u32> {
        let mut buf = [0u8; 4];
        self.read_exact(&mut buf)?;
        Ok(u32::from_le_bytes(buf))
    }
    /// Reads a 32-bit unsigned integer in big-endian format from the input and returns it as a `u32`.
    fn be_u32(&mut self) -> Result<u32> {
        let mut buf = [0u8; 4];
        self.read_exact(&mut buf)?;
        Ok(u32::from_be_bytes(buf))
    }

    /// Reads a 64-bit unsigned integer in little-endian format from the input and returns it as a `u64`.
    fn le_u64(&mut self) -> Result<u64> {
        let mut buf = [0u8; 8];
        self.read_exact(&mut buf)?;
        Ok(u64::from_le_bytes(buf))
    }
    /// Reads a 64-bit unsigned integer in big-endian format from the input and returns it as a `u64`.
    fn be_u64(&mut self) -> Result<u64> {
        let mut buf = [0u8; 8];
        self.read_exact(&mut buf)?;
        Ok(u64::from_be_bytes(buf))
    }
}

impl<T: Read> ByteRead for T {}

/// A reader that wraps a byte slice and provides `Read` and `Seek` functionality.
pub struct ByteReader<T: AsRef<[u8]> + ?Sized> {
    /// The current position in the byte slice.
    pos: usize,

    /// The underlying byte slice being read.
    inner: T,
}

impl<T: AsRef<[u8]>> ByteReader<T> {
    /// Creates a new `ByteReader` that wraps the given byte slice.
    pub fn new(inner: T) -> Self {
        Self { inner, pos: 0 }
    }
}

impl<T: AsRef<[u8]> + ?Sized> Read for ByteReader<T> {
    /// Reads bytes from the underlying byte slice into the provided buffer, returning the number of bytes read.
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let inner = self.inner.as_ref();
        let remaining = inner.len() - self.pos;
        let to_read = remaining.min(buf.len());
        buf[..to_read].copy_from_slice(&inner[self.pos..self.pos + to_read]);
        self.pos += to_read;
        Ok(to_read)
    }
}

impl<T: AsRef<[u8]> + ?Sized> Seek for ByteReader<T> {
    /// Seeks to a new position in the underlying byte slice based on the specified `SeekFrom` value.
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        let inner = self.inner.as_ref();

        let new_pos = match pos {
            SeekFrom::Start(offset) => offset as i64,
            SeekFrom::End(offset) => inner.len() as i64 + offset,
            SeekFrom::Current(offset) => self.pos as i64 + offset,
        };

        if new_pos < 0 || new_pos as usize > inner.len() {
            return Err(Error::new(ErrorKind::InvalidInput, "Invalid seek"));
        }

        self.pos = new_pos as usize;
        Ok(self.pos as u64)
    }
}
