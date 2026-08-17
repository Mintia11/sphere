use std::io::{Seek, SeekFrom};

use common::byte_io::ByteRead;

use crate::embl::{EBMLElement, EBMLMasterElement};

/// A trait for reading EBML (Extensible Binary Meta Language) data from a byte stream.
pub trait EBMLRead: ByteRead {
    /// Reads an EBML ID from the byte stream.
    fn id(&mut self) -> Result<u32, Error> {
        let first = self.le_u8()?;
        let length = (first.leading_zeros() + 1) as usize;
        if length > 4 {
            return Err(Error::InvalidIDLenght(length));
        }
        let mut buf = [0u8; 4];
        buf[4 - length] = first;
        self.read_exact(&mut buf[4 - length + 1..4])?;
        Ok(u32::from_be_bytes(buf))
    }

    /// Reads an EBML VINT (Variable Integer) from the byte stream.
    fn vint(&mut self) -> Result<u64, Error> {
        let first = self.le_u8()?;
        let length = (first.leading_zeros() + 1) as usize;
        if length > 8 {
            return Err(Error::InvalidVINTLenght(length));
        }
        let mask = 0xFFu8.unbounded_shr(length as u32);
        let mut buf = [0u8; 8];
        buf[8 - length] = first & mask;
        self.read_exact(&mut buf[8 - length + 1..8])?;
        Ok(u64::from_be_bytes(buf))
    }

    /// Reads an unsigned integer of the specified size from the byte stream.
    fn uinteger(&mut self, size: usize) -> Result<u64, Error> {
        if size > 8 {
            return Err(Error::InvalidIntegerLenght(size));
        }
        let mut buf = [0u8; 8];
        self.read_exact(&mut buf[8 - size..8])?;
        Ok(u64::from_be_bytes(buf))
    }

    /// Reads a signed integer of the specified size from the byte stream.
    fn integer(&mut self, size: usize) -> Result<i64, Error> {
        if size > 8 {
            return Err(Error::InvalidIntegerLenght(size));
        }
        let mut buf = [0u8; 8];
        self.read_exact(&mut buf[8 - size..8])?;

        // sign-extend the read integer
        if size > 0 && size < 8 && buf[8 - size] & 0x80 != 0 {
            for b in &mut buf[..8 - size] {
                *b = 0xFF;
            }
        }

        Ok(i64::from_be_bytes(buf))
    }

    /// Reads a string of the specified size from the byte stream, trimming any trailing null bytes.
    fn string(&mut self, size: usize) -> Result<String, Error> {
        let mut buf = vec![0u8; size];
        self.read_exact(&mut buf)?;
        while buf.last() == Some(&0) {
            buf.pop();
        }
        Ok(String::from_utf8(buf)?)
    }

    /// Reads a float of the specified size (4 or 8 bytes) from the byte stream.
    fn float(&mut self, size: usize) -> Result<f64, Error> {
        match size {
            0 => Ok(0.0),
            4 => {
                let mut buf = [0u8; 4];
                self.read_exact(&mut buf)?;
                Ok(f32::from_be_bytes(buf) as f64)
            }
            8 => {
                let mut buf = [0u8; 8];
                self.read_exact(&mut buf)?;
                Ok(f64::from_be_bytes(buf))
            }
            _ => Err(Error::InvalidData(format!(
                "Invalid float size: got {} expected 4 or 8",
                size
            ))),
        }
    }

    /// Reads a binary blob of the specified size from the byte stream.
    fn binary(&mut self, size: usize) -> Result<Vec<u8>, Error> {
        let mut buf = vec![0u8; size];
        self.read_exact(&mut buf)?;
        Ok(buf)
    }

    /// Reads a master element of type `T` from the byte stream
    /// If `element` is `Some(...)` use it instead of reading an EBMLElement from the bytestream
    fn master_element<T: EBMLMasterElement<Self>>(
        &mut self,
        element: Option<EBMLElement>,
    ) -> Result<T, Error>
    where
        Self: Seek + Sized,
    {
        let element = match element {
            Some(e) => e,
            None => EBMLElement::read(&mut *self)?,
        };
        if element.id != T::ID {
            return Err(Error::InvalidData(format!(
                "Expected master element with ID {:X}, got {:X}",
                T::ID,
                element.id
            )));
        }

        let mut instance = T::default();
        element.for_each_child(&mut *self, |sub_element, reader| {
            instance.visit_child(sub_element, reader)
        })?;
        instance.validate()?;
        self.seek(SeekFrom::Start(element.data_end))?;

        Ok(instance)
    }
}

impl<T: ByteRead> EBMLRead for T {}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid data encountered: {0}")]
    InvalidData(String),

    #[error("Invalid EBML ID length: got {0} expected 1-4")]
    InvalidIDLenght(usize),

    #[error("Invalid EBML VINT length: got {0} expected 1-8")]
    InvalidVINTLenght(usize),

    #[error("integer wider than 8 bytes: got {0}")]
    InvalidIntegerLenght(usize),

    #[error("Encountered error while decoding UTF-8 string: {0}")]
    FromUTF8(#[from] std::string::FromUtf8Error),
}
