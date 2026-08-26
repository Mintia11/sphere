use common::{bit_io::BitReader, byte_io::ByteRead, packet::Error};
use derive_try_from_primitive::TryFromPrimitive;

use crate::{config::Config, tables};

#[derive(Clone)]
pub struct Info {
    pub window_sequence: WindowSequence,
    pub window_shape: bool,
    pub max_sfb: usize,
    pub scale_factor_grouping: Option<u8>,
    pub window_group_length: Vec<u8>,
    pub sfb_offsets: Vec<Vec<usize>>,
}

impl Info {
    pub fn parse<T: ByteRead>(reader: &mut BitReader<T>, config: &Config) -> Result<Self, Error> {
        let reserved_bit = reader.read_bit()?;
        if reserved_bit {
            return Err(Error::InvalidData(
                "ics_info reserved_bit shouln't be set".to_string(),
            ));
        }

        let window_sequence: WindowSequence = (reader.read_bits(2)? as u8)
            .try_into()
            .map_err(|_| Error::InvalidData("invalid window sequence".to_string()))?;
        let window_shape = reader.read_bit()?;

        match window_sequence {
            WindowSequence::EightShort => {
                let max_sfb = reader.read_bits(4)? as u8;
                let grouping_bits = reader.read_bits(7)? as u8;

                let mut window_group_length = vec![1u8];
                for i in 0..7 {
                    let bit_set = (grouping_bits >> (6 - i)) & 1 == 1;
                    if bit_set {
                        *window_group_length.last_mut().unwrap() += 1;
                    } else {
                        window_group_length.push(1);
                    }
                }

                let band_info = tables::find_band_info(config.sampling_frequency).unwrap();
                let num_sfb = band_info.short.len();
                let mut sfb_offsets = Vec::new();
                for _ in &window_group_length {
                    let mut offsets = Vec::new();
                    for sfb in 0..num_sfb {
                        let offset = band_info.short[sfb];
                        offsets.push(offset);
                    }
                    sfb_offsets.push(offsets);
                }

                Ok(Info {
                    window_sequence,
                    window_shape,
                    max_sfb: max_sfb as usize,
                    scale_factor_grouping: Some(grouping_bits),
                    window_group_length,
                    sfb_offsets,
                })
            }
            _ => {
                let max_sfb = reader.read_bits(6)? as usize;
                let predictor_data_present = reader.read_bit()?;
                if predictor_data_present {
                    todo!("handle predictor data");
                }

                let band_info = tables::find_band_info(config.sampling_frequency).unwrap();

                let mut sfb_offsets = Vec::new();
                let mut group = Vec::new();
                for sfb in 0..=max_sfb {
                    let offset = band_info.long[sfb];
                    group.push(offset);
                }
                sfb_offsets.push(group);

                Ok(Info {
                    window_sequence,
                    window_shape,
                    max_sfb,
                    scale_factor_grouping: None,
                    window_group_length: vec![1],
                    sfb_offsets,
                })
            }
        }
    }
}

#[derive(Debug, TryFromPrimitive, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
#[repr(u8)]
pub enum WindowSequence {
    OnlyLong = 0,
    LongStart = 1,
    EightShort = 2,
    LongStop = 3,
}
