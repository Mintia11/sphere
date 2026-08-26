use common::{bit_io::BitReader, byte_io::ByteRead, packet::Error};

use crate::ics::info::{Info, WindowSequence};

#[derive(Clone)]
pub struct SectionData {
    pub groups: Vec<Vec<Section>>,
}

#[derive(Clone, Copy)]
pub struct Section {
    pub cb: u8,
    pub start: usize,
    pub end: usize,
}

impl SectionData {
    pub fn parse<T: ByteRead>(reader: &mut BitReader<T>, info: &Info) -> Result<Self, Error> {
        let is_short = info.window_sequence == WindowSequence::EightShort;
        let escape_value = if is_short { (1 << 3) - 1 } else { (1 << 5) - 1 };
        let lenght_len = if is_short { 3 } else { 5 };

        let mut groups: Vec<Vec<Section>> = Vec::with_capacity(info.window_group_length.len());

        for _ in 0..info.window_group_length.len() {
            let mut sections = Vec::new();
            let mut sfb = 0;
            while sfb < info.max_sfb {
                let cb = reader.read_bits(4)? as u8;

                let mut len = 0u16;
                let mut read_len = reader.read_bits(lenght_len)? as u16;
                while read_len == escape_value {
                    len += escape_value;
                    read_len = reader.read_bits(lenght_len)? as u16;
                }
                len += read_len;

                let start = sfb;
                let end = sfb + len as usize;
                if end > info.max_sfb {
                    panic!("Section end {} exceeds max_sfb {}", end, info.max_sfb);
                }

                sections.push(Section { cb, start, end });
                sfb = end;
            }
            groups.push(sections);
        }

        Ok(SectionData { groups })
    }

    pub fn find_section(&self, group: usize, sfb: usize) -> Option<&Section> {
        self.groups.get(group).and_then(|group| {
            group
                .iter()
                .find(|&section| sfb >= section.start && sfb < section.end)
        })
    }
}
