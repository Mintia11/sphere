use std::{marker::PhantomData, sync::Arc};

use common::{
    packet::{Error, Frame, Packet, PacketDecoder},
    track::TrackInfo,
};
use etna::{
    Device,
    dynamic_buffer::DynamicBuffer,
    video::session::VideoSession,
    vk::{self, TaggedStructure},
};

use crate::{
    avcc::Avcc,
    nal::{LenghtPrefixedNal, NalType},
    poc::PocDecoderState,
    pps::Pps,
    slice_header::SliceHeader,
    sps::Sps,
};

mod avcc;
mod nal;
mod poc;
mod pps;
mod slice_header;
mod sps;

pub struct H264Decoder {
    device: Arc<Device>,
    sps: Option<Sps>,
    pps: Option<Pps>,

    session: Option<VideoSession>,
    input_buffer: DynamicBuffer,

    poc_state: PocDecoderState,
}

impl H264Decoder {
    pub fn new(device: &Arc<Device>) -> Self {
        Self {
            device: device.clone(),
            sps: None,
            pps: None,

            session: None,
            input_buffer: DynamicBuffer::new(
                device,
                vk::BufferUsageFlags::VIDEO_DECODE_SRC_KHR,
                32 * 1024,
            )
            .expect("Failed to create input buffer"),

            poc_state: Default::default(),
        }
    }

    fn h264_profile<'a>(
        sps: &Sps,
        profile_h264: &'a mut vk::VideoDecodeH264ProfileInfoKHR<'a>,
    ) -> vk::VideoProfileInfoKHR<'a> {
        *profile_h264 = vk::VideoDecodeH264ProfileInfoKHR::default()
            .std_profile_idc(sps.profile_idc.into())
            .picture_layout(vk::VideoDecodeH264PictureLayoutFlagsKHR::PROGRESSIVE);

        vk::VideoProfileInfoKHR::default()
            .chroma_bit_depth(sps.bit_depth_chroma())
            .chroma_subsampling(sps.chroma_format.into())
            .luma_bit_depth(sps.bit_depth_luma())
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
        self.sps = Some(Sps::parse(&avcc.sps[0])?);
        self.pps = Some(Pps::parse(&avcc.pps[0])?);

        Ok(())
    }

    fn info_strings(&self) -> Vec<String> {
        let mut info_strings = Vec::new();
        if let Some(sps) = &self.sps {
            info_strings.push(format!(
                "Profile: {:?} (Level: {:?})",
                sps.profile_idc, sps.level_idc
            ));
            info_strings.push(format!(
                "Chroma Format: {:?} (Luma bits: {}, Chroma bits: {})",
                sps.chroma_format,
                sps.bit_depth_luma_minus_8 + 8,
                sps.bit_depth_chroma_minus_8 + 8
            ));
            info_strings.push(format!("Resolution: {}x{}", sps.width(), sps.height()));
        }

        info_strings
    }

    fn can_decode_track(&self) -> Result<bool, Error> {
        let Some(sps) = &self.sps else {
            return Ok(false);
        };

        let mut profile_h264 = vk::VideoDecodeH264ProfileInfoKHR::default();
        let profile = Self::h264_profile(sps, &mut profile_h264);

        match self.get_capabilities(&profile) {
            Ok(_) => Ok(true),
            Err(e) => {
                eprintln!("Video capability query failed: {e}");
                Ok(false)
            }
        }
    }

    fn start_decode_session(&mut self) -> Result<(), Error> {
        let Some(sps) = &self.sps else {
            return Ok(());
        };

        let mut profile_h264 = vk::VideoDecodeH264ProfileInfoKHR::default();
        let profile = Self::h264_profile(sps, &mut profile_h264);
        let (caps, _, _) = self.get_capabilities(&profile)?;

        let formats = self
            .device
            .get_formats_for_profile(&profile, vk::ImageUsageFlags::VIDEO_DECODE_DST_KHR)?;

        println!("Formats: {formats:#x?}");

        let pps = self.pps.unwrap().into();
        let sps = self.sps.clone().unwrap().into();

        let add_info = vk::VideoDecodeH264SessionParametersAddInfoKHR::default()
            .std_pp_ss(std::slice::from_ref(&pps))
            .std_sp_ss(std::slice::from_ref(&sps));
        let mut params_info = vk::VideoDecodeH264SessionParametersCreateInfoKHR::default()
            .max_std_pps_count(1)
            .max_std_sps_count(1)
            .parameters_add_info(&add_info);

        let session = self.device.create_video_session(
            &caps,
            &profile,
            formats[0].format,
            formats[0].format,
            &mut params_info,
        )?;
        self.session = Some(session);

        Ok(())
    }

    fn send_packet(&mut self, packet: Packet) -> Result<(), Error> {
        let Some(sps) = &self.sps else {
            return Ok(());
        };
        let Some(pps) = &self.pps else {
            return Ok(());
        };

        let nals = LenghtPrefixedNal::parse(packet.data, 4)?;
        let (picture_info, nal) = nals
            .iter()
            .find(|nal| matches!(nal.typ(), NalType::Idr | NalType::NonIdr))
            .map(|nal| {
                (
                    SliceHeader::parse(nal.data(), sps, pps, nal.typ() == NalType::Idr)
                        .expect("failed to parse slice header"),
                    nal,
                )
            })
            .expect("picture nal not found");

        let poc = self.poc_state.compute_and_update(
            sps,
            &picture_info,
            nal.typ() == NalType::Idr,
            nal.ref_idc(),
        );
        let picture_info =
            picture_info.into_picture_info(sps, nal.typ() == NalType::Idr, nal.ref_idc() != 0, poc);

        let mut annex_b_data = Vec::new();
        for nal in nals {
            match nal.typ() {
                NalType::Sps | NalType::Pps | NalType::Sei => {}
                _ => annex_b_data.extend(nal.into_annex_b()),
            }
        }

        self.input_buffer
            .ensure_capacity(&self.device, annex_b_data.len().next_multiple_of(256))?;
        self.input_buffer.write(0, &annex_b_data)?;

        Ok(())
    }

    fn grab_frame(&self) -> Result<Option<Frame>, Error> {
        Ok(None)
    }
}
