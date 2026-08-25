use bytes::{Buf, Bytes};
use common::{bit_io::BitReader, byte_io::ByteReader, packet::Error};
use derive_try_from_primitive::TryFromPrimitive;

#[derive(Debug)]
pub struct RawNal {
    typ: NalType,
    raw_hdr: u8,
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
            raw_hdr: bytes[0],
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

pub struct LenghtPrefixedNal {
    inner: RawNal,
}

impl LenghtPrefixedNal {
    pub fn parse(mut bytes: Bytes, length_size: usize) -> Result<Vec<LenghtPrefixedNal>, Error> {
        let mut nals = Vec::new();

        while !bytes.is_empty() {
            if bytes.len() < length_size {
                return Err(Error::InvalidData("Truncated length prefix".into()));
            }

            let mut length = 0;
            for &byte in &bytes[..length_size] {
                length = (length << 8) | (byte as usize);
            }

            bytes.advance(length_size);

            if bytes.len() < length {
                return Err(Error::InvalidData("Truncated NAL payload".into()));
            }

            let nal_bytes = bytes.split_to(length);
            let nal = RawNal::parse(&nal_bytes)?;
            nals.push(LenghtPrefixedNal { inner: nal });
        }

        Ok(nals)
    }

    pub fn into_annex_b(self) -> Vec<u8> {
        let mut data = Vec::with_capacity(4 + 1 + self.inner.data.len());
        data.extend_from_slice(&[0x00, 0x00, 0x00, 0x01, self.inner.raw_hdr]);
        data.extend_from_slice(&self.inner.data);
        data
    }
}
