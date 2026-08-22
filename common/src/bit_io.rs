//! Bit-level I/O utilities

use crate::byte_io::ByteRead;

use std::io::{Error, ErrorKind, Result};

/// A reader that allows reading bits from an underlying byte-oriented reader.
pub struct BitReader<T: ByteRead> {
    /// The underlying byte-oriented reader from which bits are read.
    reader: T,

    /// A buffer that holds bits read from the underlying reader. The bits are stored in the higher-order bits of this `u64`.
    bit_buf: u64,

    /// The number of bits currently available in the `bit_buf`.
    /// This indicates how many bits can be read from the buffer before needing to read more bytes from the underlying reader.
    bits_left: u32,

    /// A flag indicating whether the end of the underlying reader has been reached.
    eof: bool,
}

impl<T: ByteRead> BitReader<T> {
    /// Creates a new `BitReader` that wraps the given byte-oriented reader.
    pub fn new(reader: T) -> Self {
        Self {
            reader,
            bit_buf: 0,
            bits_left: 0,
            eof: false,
        }
    }

    /// Fills the bit buffer by reading bytes from the underlying reader until there are at least 56 bits available or the end of the reader is reached.
    fn fill(&mut self) -> Result<()> {
        while self.bits_left <= 56 && !self.eof {
            match self.reader.le_u8() {
                Ok(byte) => {
                    self.bit_buf |= (byte as u64) << (56 - self.bits_left);
                    self.bits_left += 8;
                }
                Err(e) if e.kind() == ErrorKind::UnexpectedEof => {
                    self.eof = true;
                }
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    /// Peeks at the specified number of bits from the bit buffer without consuming them.
    pub fn peek_bits(&mut self, num_bits: u32) -> Result<u64> {
        assert!(
            num_bits <= 57,
            "peek_bits supports at most 57 bits at a time"
        );
        self.fill()?;
        Ok(self.bit_buf >> (64 - num_bits))
    }

    /// Consumes the specified number of bits from the bit buffer, effectively discarding them.
    pub fn consume_bits(&mut self, num_bits: u32) {
        self.bit_buf <<= num_bits;
        self.bits_left = self.bits_left.saturating_sub(num_bits);
    }

    /// Reads the specified number of bits from the bit buffer, consuming them in the process, and returns the value as a `u64`.
    pub fn read_bits(&mut self, num_bits: u32) -> Result<u64> {
        if num_bits == 0 || num_bits > 57 {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!("invalid number of bits to read: {num_bits}"),
            ));
        }
        let val = self.peek_bits(num_bits)?;
        self.consume_bits(num_bits);
        Ok(val)
    }

    /// Reads a single bit from the bit buffer, consuming it in the process, and returns the value as a `bool`.
    pub fn read_bit(&mut self) -> Result<bool> {
        Ok(self.read_bits(1)? != 0)
    }

    /// Returns the current bit position within the current byte, which is useful for determining alignment and for debugging purposes.
    pub fn bit_position_in_byte(&self) -> u32 {
        (8 - self.bits_left % 8) % 8
    }

    /// Aligns the bit reader to the next byte boundary by consuming any remaining bits in the current byte.
    pub fn byte_align(&mut self) {
        let rem = self.bits_left % 8;
        if rem != 0 {
            self.consume_bits(rem);
        }
    }

    /// Returns the number of bits left in the bit buffer, which indicates how many bits can be read before needing to fill the buffer again.
    pub fn bits_left(&self) -> u32 {
        self.bits_left
    }

    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.reader
    }

    pub fn eof(&self) -> bool {
        self.eof
    }
}
