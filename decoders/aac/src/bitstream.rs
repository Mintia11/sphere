use bytes::Bytes;
use common::{bit_io::BitReader, byte_io::ByteRead, packet::Error};

use crate::{config::Config, ics::Ics};

pub enum SyntaxElement {
    SingleChannel { ics: Ics },
    Fill { extension: Option<ExtensionPayload> },
}

impl SyntaxElement {
    pub fn parse_all<T: ByteRead>(
        reader: &mut BitReader<T>,
        config: &Config,
    ) -> Result<Vec<Self>, Error> {
        let mut elements = Vec::new();

        while !reader.eof() {
            match reader.read_bits(3)? {
                0 => {
                    // ID_SCE
                    let _tag = reader.read_bits(4)?;
                    let ics = Ics::parse(reader, config, None)?;
                    elements.push(SyntaxElement::SingleChannel { ics });
                }
                6 => {
                    // ID_FIL
                    let count = reader.read_bits(4)?;
                    let count = if count == 15 {
                        reader.read_bits(8)? - 1
                    } else {
                        count
                    };

                    let extension = if count > 0 {
                        Some(ExtensionPayload::parse(reader, count)?)
                    } else {
                        None
                    };

                    elements.push(SyntaxElement::Fill { extension });
                }
                7 => {
                    // ID_END
                    break;
                }
                id => todo!("Handle AAC packet with ID {}", id),
            }
        }

        Ok(elements)
    }
}

#[derive(Debug)]
pub enum ExtensionPayload {
    FillData { _data: Bytes },
}

impl ExtensionPayload {
    pub fn parse<T: ByteRead>(reader: &mut BitReader<T>, count: u64) -> Result<Self, Error> {
        let ext_type = reader.read_bits(4)?;
        match ext_type {
            0 => {
                // EXT_FILL
                let _nibble = reader.read_bits(4)?;
                let mut data = vec![0u8; (count - 1) as usize];
                for byte in &mut data {
                    *byte = reader.read_bits(8)? as u8;
                }
                Ok(ExtensionPayload::FillData { _data: data.into() })
            }
            _ => todo!("Unknown extension type: {}", ext_type),
        }
    }
}
