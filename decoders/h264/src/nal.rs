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

        let _ref_idc = reader.read_bits(3)?;
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

    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

#[derive(Debug, TryFromPrimitive, Clone, Copy)]
#[repr(u64)]
pub enum NalType {
    Sps = 0xE,
    Pps = 0x10,
}
