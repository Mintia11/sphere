use common::{bit_io::BitReader, byte_io::ByteRead, packet::Error};

use crate::{
    config::Config,
    ics::{info::Info, pulse::PulseData, scale_factors::ScaleFactors, section::SectionData},
};

mod info;
mod pulse;
mod scale_factors;
mod section;

#[derive(Debug)]
pub struct Ics {}

impl Ics {
    pub fn parse<T: ByteRead>(
        reader: &mut BitReader<T>,
        config: &Config,
        info: Option<Info>,
    ) -> Result<Self, Error> {
        let global_gain = reader.read_bits(8)? as u8;
        let info = match info {
            Some(info) => info,
            None => Info::parse(reader, config)?,
        };
        let section = SectionData::parse(reader, &info)?;
        let scale_factors = ScaleFactors::parse(reader, global_gain, &info, &section)?;
        let pulse = PulseData::parse(reader, &info)?;
        assert!(pulse.is_none(), "todo: use pulse data");

        Ok(Self {})
    }
}
