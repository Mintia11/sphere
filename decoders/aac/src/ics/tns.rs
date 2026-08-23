use common::{bit_io::BitReader, byte_io::ByteRead, packet::Error};

#[derive(Clone)]
pub struct TNSData {}

impl TNSData {
    pub fn parse<T: ByteRead>(reader: &mut BitReader<T>) -> Result<Option<Self>, Error> {
        let tns_data_present = reader.read_bit()?;
        if !tns_data_present {
            return Ok(None);
        }

        todo!("parse tns data")
    }
}
