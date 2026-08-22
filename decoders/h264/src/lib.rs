use std::sync::Arc;

use common::{
    bit_io::BitReader,
    byte_io::ByteReader,
    packet::{Error, PacketDecoder},
    track::TrackInfo,
};
use etna::Device;

use crate::avcc::Avcc;

mod avcc;

pub struct H264Decoder {
    device: Arc<Device>,
}

impl H264Decoder {
    pub fn new(device: &Arc<Device>) -> Self {
        Self {
            device: device.clone(),
        }
    }
}

impl PacketDecoder for H264Decoder {
    fn track(&mut self, track: &TrackInfo) -> Result<(), Error> {
        let private_data = track.extra_data.as_ref().ok_or(Error::InvalidData(
            "Track has no codec private data".to_string(),
        ))?;

        let reader = ByteReader::new(private_data);
        let mut reader = BitReader::new(reader);

        let avcc = Avcc::parse(&mut reader)?;

        Ok(())
    }

    fn can_decode_track(&self) -> Result<bool, Error> {
        Ok(false)
    }
}
