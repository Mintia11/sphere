use std::io::Result;

use crate::{bit_io::BitReader, byte_io::ByteRead};

impl<T: ByteRead> BitReader<T> {
    pub fn read_exp(&mut self) -> Result<u64> {
        let mut leading_zeroes = 1;
        while !self.read_bit()? {
            leading_zeroes += 1;
        }
        self.read_bits(leading_zeroes)
    }

    pub fn read_exp_signed(&mut self) -> Result<i64> {
        let val = self.read_exp()?;
        let ret = val.div_ceil(2) as i64;
        if val % 2 == 0 { Ok(-ret) } else { Ok(ret) }
    }
}
