use common::{bit_io::BitReader, byte_io::ByteRead, packet::Error};

use crate::{config::Config, ics::info::Info};

mod info;

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

        Ok(Self {})
    }
}
