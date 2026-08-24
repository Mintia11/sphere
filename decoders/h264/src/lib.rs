use std::{marker::PhantomData, sync::Arc};

use common::{
    packet::{Error, Frame, Packet, PacketDecoder},
    track::TrackInfo,
};
use etna::{
    Device,
    video::session::VideoSession,
    vk::{self, TaggedStructure},
};

use crate::{avcc::Avcc, sps::Sps};

mod avcc;
mod nal;
mod sps;

pub struct H264Decoder {
    device: Arc<Device>,
    config: Option<Avcc>,

    session: Option<VideoSession>,
}

impl H264Decoder {
    pub fn new(device: &Arc<Device>) -> Self {
        Self {
            device: device.clone(),
            config: None,

            session: None,
        }
    }

    fn h264_profile<'a>(
        config: &Avcc,
        profile_h264: &'a mut vk::VideoDecodeH264ProfileInfoKHR<'a>,
    ) -> vk::VideoProfileInfoKHR<'a> {
        *profile_h264 = vk::VideoDecodeH264ProfileInfoKHR::default()
            .std_profile_idc(config.profile.into())
            .picture_layout(vk::VideoDecodeH264PictureLayoutFlagsKHR::PROGRESSIVE);

        vk::VideoProfileInfoKHR::default()
            .chroma_bit_depth(config.bit_depth_chroma())
            .chroma_subsampling(config.chroma_format.into())
            .luma_bit_depth(config.bit_depth_luma())
            .video_codec_operation(vk::VideoCodecOperationFlagsKHR::DECODE_H264)
            .push(profile_h264)
    }

    fn get_capabilities(
        &self,
        profile: &vk::VideoProfileInfoKHR,
    ) -> Result<
        (
            vk::VideoCapabilitiesKHR<'_>,
            vk::VideoDecodeCapabilitiesKHR<'_>,
            vk::VideoDecodeH264CapabilitiesKHR<'_>,
        ),
        Error,
    > {
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
                    profile,
                    &mut capabilities,
                )?;
        }

        Ok((
            vk::VideoCapabilitiesKHR {
                _marker: PhantomData,
                ..capabilities
            },
            vk::VideoDecodeCapabilitiesKHR {
                _marker: PhantomData,
                ..decode_capabilities
            },
            vk::VideoDecodeH264CapabilitiesKHR {
                _marker: PhantomData,
                ..h264_capabilities
            },
        ))
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
            if let Ok(sps) = Sps::parse(&config.sps[0]) {
                info_strings.push(format!(
                    "Chroma Format: {:?} (Luma bits: {}, Chroma bits: {})",
                    sps.chroma_format,
                    sps.bit_depth_luma_minus_8 + 8,
                    sps.bit_depth_chroma_minus_8 + 8
                ));
                info_strings.push(format!("Resolution: {}x{}", sps.width(), sps.height()));
            }
        }

        info_strings
    }

    fn can_decode_track(&self) -> Result<bool, Error> {
        let Some(config) = &self.config else {
            return Ok(false);
        };

        let mut profile_h264 = vk::VideoDecodeH264ProfileInfoKHR::default();
        let profile = Self::h264_profile(config, &mut profile_h264);

        match self.get_capabilities(&profile) {
            Ok(_) => Ok(true),
            Err(e) => {
                eprintln!("Video capability query failed: {e}");
                Ok(false)
            }
        }
    }

    fn start_decode_session(&mut self) -> Result<(), Error> {
        let Some(config) = &self.config else {
            return Ok(());
        };

        let mut profile_h264 = vk::VideoDecodeH264ProfileInfoKHR::default();
        let profile = Self::h264_profile(config, &mut profile_h264);
        let (caps, _, _) = self.get_capabilities(&profile)?;

        let formats = self
            .device
            .get_formats_for_profile(&profile, vk::ImageUsageFlags::VIDEO_DECODE_DST_KHR)?;

        println!("Formats: {formats:#x?}");

        let session = self.device.create_video_session(
            &caps,
            &profile,
            formats[0].format,
            formats[0].format,
        )?;
        self.session = Some(session);

        Ok(())
    }

    fn send_packet(&mut self, packet: Packet) -> Result<(), Error> {
        Ok(())
    }

    fn grab_frame(&self) -> Result<Option<Frame>, Error> {
        Ok(None)
    }
}
