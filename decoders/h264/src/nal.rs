use bytes::Bytes;
use common::{bit_io::BitReader, byte_io::ByteReader, packet::Error};
use derive_try_from_primitive::TryFromPrimitive;

#[derive(Debug)]
pub struct RawNal {
    typ: NalType,
    data: Bytes,
}

impl RawNal {
    pub fn parse(bytes: &Bytes) -> Result<Self, Error> {
        let first_byte = [bytes[0]];
        let reader = ByteReader::new(first_byte);
        let mut reader = BitReader::new(reader);

        let forbidden_zero = reader.read_bit()?;
        if forbidden_zero {
            return Err(Error::InvalidData(
                "nal's forbidden_zero_bit was not zero".to_string(),
            ));
        }

        let _ref_idc = reader.read_bits(2)?;
        let nal_unit_type = reader.read_bits(5)?;
        let typ = NalType::try_from(nal_unit_type)
            .map_err(|e| Error::InvalidData(format!("Invalid nal type: {e:#x}")))?;

        Ok(RawNal {
            typ,
            data: bytes.slice(1..),
        })
    }

    pub fn typ(&self) -> NalType {
        self.typ
    }

    pub fn strip_emulation_prevention(&self) -> Vec<u8> {
        let mut rbsp = Vec::with_capacity(self.data.len());
        let mut i = 0;

        while i < self.data.len() {
            if i + 2 < self.data.len()
                && self.data[i] == 0x00
                && self.data[i + 1] == 0x00
                && self.data[i + 2] == 0x03
            {
                rbsp.push(0x00);
                rbsp.push(0x00);
                i += 3;
            } else {
                rbsp.push(self.data[i]);
                i += 1;
            }
        }

        rbsp
    }
}

#[derive(Debug, TryFromPrimitive, Clone, Copy)]
#[repr(u64)]
pub enum NalType {
    Sps = 0x7,
    Pps = 0x8,
}
