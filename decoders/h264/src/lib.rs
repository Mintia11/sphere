use std::sync::Arc;

use common::{
    packet::{Error, PacketDecoder},
    track::TrackInfo,
};
use etna::{
    Device,
    vk::{self, TaggedStructure},
};

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

    fn info_strings(&self) -> Vec<String> {
        let mut info_strings = Vec::new();
        if let Some(config) = &self.config {
            info_strings.push(format!(
                "Profile: {:?} (Level: {:?})",
                config.profile, config.level
            ));
            info_strings.push(format!(
                "Chroma Format: {:?} (Luma bits: {}, Chroma bits: {})",
                config.chroma_format, config.bit_depth_luma, config.bit_depth_chroma
            ));
        }

        info_strings
    }

    fn can_decode_track(&self) -> Result<bool, Error> {
        if let Some(config) = &self.config {
            let mut profile_h264 = vk::VideoDecodeH264ProfileInfoKHR::default()
                .std_profile_idc(config.profile.into())
                .picture_layout(vk::VideoDecodeH264PictureLayoutFlagsKHR::PROGRESSIVE);

            let profile = vk::VideoProfileInfoKHR::default()
                .chroma_bit_depth(config.bit_depth_chroma())
                .chroma_subsampling(config.chroma_format.into())
                .luma_bit_depth(config.bit_depth_luma())
                .video_codec_operation(vk::VideoCodecOperationFlagsKHR::DECODE_H264)
                .push(&mut profile_h264);

            let mut h264_capabilities = vk::VideoDecodeH264CapabilitiesKHR::default();
            let mut decode_capabilities = vk::VideoDecodeCapabilitiesKHR::default();
            let mut capabilities = vk::VideoCapabilitiesKHR::default()
                .push(&mut decode_capabilities)
                .push(&mut h264_capabilities);

            unsafe {
                self.device
                    .video_queue_instance_ext()
                    .get_physical_device_video_capabilities(
                        self.device.physical_device(),
                        &profile,
                        &mut capabilities,
                    )?;
            }

            println!("{capabilities:#?}");
            println!("{decode_capabilities:#?}");
            println!("{h264_capabilities:#?}");

            Ok(true)
        } else {
            Ok(false)
        }
    }
}
