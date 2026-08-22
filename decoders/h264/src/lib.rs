use std::sync::Arc;

use common::{
    packet::{Error, PacketDecoder},
    track::TrackInfo,
};
use etna::Device;

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
    fn can_decode_track(&self, track: &TrackInfo) -> Result<bool, Error> {
        let private_data = track.extra_data.as_ref().ok_or(Error::CannotDecodeTrack(
            "Track has no codec private data".to_string(),
        ))?;

        Ok(false)
    }
}
