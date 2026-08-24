use bytes::Bytes;
use common::{bit_io::BitReader, byte_io::ByteRead, packet::Error};

use crate::{
    config::Config,
    ics::{Ics, WindowSequence, info::Info},
};

pub enum SyntaxElement {
    SingleChannel { ics: Ics },
    ChannelPairElement { ics: [Ics; 2] },
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
                1 => {
                    // ID_CPE
                    let _tag = reader.read_bits(4)?;
                    let common_window = reader.read_bit()?;
                    let mut ms_used = Vec::new();
                    let info = if common_window {
                        let info = Info::parse(reader, config)?;
                        let ms_mask_present = reader.read_bits(2)?;
                        match ms_mask_present {
                            0 | 2 => {
                                let is_used = ms_mask_present == 2;
                                for _ in 0..info.window_group_length.len() {
                                    let mut group = Vec::new();
                                    for _ in 0..info.max_sfb {
                                        group.push(is_used);
                                    }
                                    ms_used.push(group);
                                }
                            }
                            1 => {
                                for _ in 0..info.window_group_length.len() {
                                    let mut group = Vec::new();
                                    for _ in 0..info.max_sfb {
                                        group.push(reader.read_bit()?);
                                    }
                                    ms_used.push(group);
                                }
                            }
                            _ => unreachable!(),
                        }

                        Some(info)
                    } else {
                        None
                    };

                    let mut ics_1 = Ics::parse(reader, config, info.clone())?;
                    let mut ics_2 = Ics::parse(reader, config, info)?;

                    if common_window {
                        let offsets = &ics_1.info.sfb_offsets;
                        let mut w = 0;

                        for (g, &group_len) in ics_1.info.window_group_length.iter().enumerate() {
                            for _ in 0..group_len {
                                for sfb in 0..ics_1.info.max_sfb {
                                    let start = w * 128 + offsets[g][sfb];
                                    let end = w * 128 + offsets[g][sfb + 1];

                                    let Some(section) = ics_2.section.find_section(g, sfb) else {
                                        continue;
                                    };

                                    let ms_used = ms_used[g][sfb];
                                    match section.cb {
                                        13 => {}
                                        cb @ (14 | 15) => {
                                            let dir = if cb == 15 { 1.0 } else { -1.0 };
                                            let scale = dir * ics_2.scale_factors.groups[g][sfb];

                                            let left = &ics_2.spectral.coefficents[start..end];
                                            let right = &mut ics_1.spectral.coefficents[start..end];

                                            for (l, r) in left.iter().zip(right) {
                                                *r = scale * l;
                                            }
                                        }
                                        _ if ms_used => {
                                            let mid = &mut ics_1.spectral.coefficents[start..end];
                                            let side = &mut ics_2.spectral.coefficents[start..end];

                                            for (m, s) in mid.iter_mut().zip(side) {
                                                let tmp = *m - *s;
                                                *m += *s;
                                                *s = tmp;
                                            }
                                        }
                                        _ => {}
                                    }
                                }

                                w += 1;
                            }
                        }
                    }
                    elements.push(SyntaxElement::ChannelPairElement {
                        ics: [ics_1, ics_2],
                    });
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
