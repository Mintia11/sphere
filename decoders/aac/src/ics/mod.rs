use common::{bit_io::BitReader, byte_io::ByteRead, packet::Error};

use crate::config::Config;

#[derive(Debug)]
pub struct Ics {}

impl Ics {
    pub fn parse<T: ByteRead>(
        reader: &mut BitReader<T>,
        config: &Config,
        info: Option<()>,
    ) -> Result<Self, Error> {
        Ok(Self {})
    }
}
