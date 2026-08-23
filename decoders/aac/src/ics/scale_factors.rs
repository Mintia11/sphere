use common::{bit_io::BitReader, byte_io::ByteRead, packet::Error};

use crate::{
    ics::{info::Info, section::SectionData},
    tables,
};

pub struct ScaleFactors {
    groups: Vec<Vec<f32>>,
}

impl ScaleFactors {
    pub fn parse<T: ByteRead>(
        reader: &mut BitReader<T>,
        global_gain: u8,
        info: &Info,
        section: &SectionData,
    ) -> Result<Self, Error> {
        let table = &tables::SCF_HUFFMAN_TABLE;
        let mut groups = Vec::with_capacity(info.max_sfb);
        let mut noise_pcm_flag = true;

        let mut normal = global_gain as i16;
        let mut noise = global_gain as i16 + 10;

        for g in 0..info.window_group_length.len() {
            let mut group_values = vec![];
            for sfb in 0..info.max_sfb {
                let section = section.find_section(g, sfb).unwrap();

                match section.cb {
                    0 => group_values.push(0.0),
                    13 => {
                        if noise_pcm_flag {
                            noise_pcm_flag = false;
                            noise += reader.read_bits(9)? as i16 - 256;
                        } else {
                            noise += table.decode(reader)? as i16 - 60;
                        }

                        assert!((0..256).contains(&noise), "noise out of range");

                        let value = tables::NORMAL_SCF_TABLE[noise as usize];
                        group_values.push(value);
                    }
                    14 | 15 => todo!("intensity"),
                    _ => {
                        normal += table.decode(reader)? as i16 - 60;
                        assert!((0..256).contains(&normal), "normal out of range");

                        let value = tables::NORMAL_SCF_TABLE[normal as usize];
                        group_values.push(value);
                    }
                }
            }

            groups.push(group_values);
        }

        Ok(ScaleFactors { groups })
    }
}
