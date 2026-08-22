use std::sync::Arc;

use common::{
    packet::{Error, PacketDecoder},
    track::TrackInfo,
};
use etna::Device;

use crate::avcc::Avcc;

mod avcc;
mod nal;

pub struct H264Decoder {
    device: Arc<Device>,
    config: Option<Avcc>,
}

impl H264Decoder {
    pub fn new(device: &Arc<Device>) -> Self {
        Self {
            device: device.clone(),
            config: None,
        }
    }
}

impl PacketDecoder for H264Decoder {
    fn track(&mut self, track: &TrackInfo) -> Result<(), Error> {
        let private_data = track.extra_data.as_ref().ok_or(Error::InvalidData(
            "Track has no codec private data".to_string(),
        ))?;

        let avcc = Avcc::parse(private_data)?;
        self.config = Some(avcc);

        Ok(())
    }

    fn can_decode_track(&self) -> Result<bool, Error> {
        Ok(false)
    }
}
