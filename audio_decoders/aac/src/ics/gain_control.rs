use common::{bit_io::BitReader, byte_io::ByteRead, packet::Error};

#[derive(Clone)]
pub struct GainControl {}

impl GainControl {
    pub fn parse<T: ByteRead>(reader: &mut BitReader<T>) -> Result<Option<Self>, Error> {
        let gain_control_data_present = reader.read_bit()?;
        if !gain_control_data_present {
            return Ok(None);
        }

        todo!("parse gain control")
    }
}
