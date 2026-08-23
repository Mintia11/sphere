use std::io;

use crate::{bit_io::BitReader, byte_io::ByteRead};

pub struct HuffmanTable {
    max_code_len: u32,
    lookup: Vec<Option<(u8, u16)>>,
}

impl HuffmanTable {
    pub fn build(lens: &[u8], codes: &[u32]) -> Self {
        assert_eq!(lens.len(), codes.len());
        let max_code_len = *lens.iter().max().unwrap() as u32;
        let table_size = 1usize << max_code_len;
        let mut lookup = vec![None; table_size];

        for (symbol_idx, (&len, &code)) in lens.iter().zip(codes.iter()).enumerate() {
            let len = len as u32;
            let shift = max_code_len - len;
            let base = (code as usize) << shift;
            let count = 1usize << shift;
            for suffix in 0..count {
                lookup[base + suffix] = Some((len as u8, symbol_idx as u16));
            }
        }

        Self {
            max_code_len,
            lookup,
        }
    }

    pub fn decode<T: ByteRead>(&self, reader: &mut BitReader<T>) -> Result<u16, Error> {
        let peeked = reader.peek_bits(self.max_code_len)?;
        if let Some((len, symbol)) = self.lookup[peeked as usize] {
            reader.consume_bits(len as u32);
            Ok(symbol)
        } else {
            Err(Error::InvalidHuffmanCode)
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O error occurred: {0}")]
    IoError(#[from] io::Error),
    #[error("Invalid Huffman code encountered")]
    InvalidHuffmanCode,
}
