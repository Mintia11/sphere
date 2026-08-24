use common::{bit_io::BitReader, byte_io::ByteReader, packet::Error};
use etna::vk;

use crate::{
    avcc::{ChromaFormat, Level, Profile},
    nal::RawNal,
};

pub struct Sps {
    profile_idc: Profile,
    constrain_set_flags: [bool; 6],
    level_idc: Level,
    seq_parameter_set_id: u64,
    chroma_format: ChromaFormat,
    separate_colour_plane: bool,
    bit_depth_luma_minus_8: u64,
    bit_depth_chroma_minus_8: u64,
    qpprime_y_zero_transform_bypass: bool,
    seq_scaling_matrix_present: bool,
    log2_max_frame_num_minus4: u64,
    pic_order_cnt_type: u64,
    log2_max_pic_order_cnt_lsb_minus4: u64,
    delta_pic_order_always_zero: bool,
    offset_for_non_ref_pic: i64,
    offset_for_top_to_bottom_field: i64,
    num_ref_frames_in_pic_order_cnt_cycle: u64,
    offset_for_ref_frame: Vec<i64>,
    max_num_ref_frames: u64,
    gaps_in_frame_num_value_allowed: bool,
    pic_width_in_mbs_minus1: u64,
    pic_height_in_map_units_minus1: u64,
    frame_mbs_only: bool,
    mb_adaptive_frame_field: bool,
    direct_8x8_inference: bool,
    frame_cropping: bool,
    vui_parameters_present: bool,
}

impl Sps {
    pub fn parse(nal: &RawNal) -> Result<Self, Error> {
        let reader = ByteReader::new(nal.data());
        let mut reader = BitReader::new(reader);

        let profile_idc = reader.read_bits(8)?;
        let constraint_set0_flag = reader.read_bit()?;
        let constraint_set1_flag = reader.read_bit()?;
        let constraint_set2_flag = reader.read_bit()?;
        let constraint_set3_flag = reader.read_bit()?;
        let constraint_set4_flag = reader.read_bit()?;
        let constraint_set5_flag = reader.read_bit()?;
        let _ = reader.read_bits(2)?;
        let level_idc = reader.read_bits(8)?;
        let seq_parameter_set_id = reader.read_exp()?;

        let profile_idc = Profile::parse(profile_idc as u8, 0);

        let mut chroma_format = ChromaFormat::Yuv420;
        let mut separate_colour_plane = false;
        let mut bit_depth_luma_minus_8 = 0;
        let mut bit_depth_chroma_minus_8 = 0;
        let mut qpprime_y_zero_transform_bypass = false;
        let mut seq_scaling_matrix_present = false;

        if profile_idc.is_high() {
            chroma_format = reader
                .read_exp()?
                .try_into()
                .map_err(|_| Error::InvalidData("invalid chroma_format".to_string()))?;
            if chroma_format == ChromaFormat::Yuv444 {
                separate_colour_plane = reader.read_bit()?;
            }

            bit_depth_luma_minus_8 = reader.read_exp()?;
            bit_depth_chroma_minus_8 = reader.read_exp()?;
            qpprime_y_zero_transform_bypass = reader.read_bit()?;
            seq_scaling_matrix_present = reader.read_bit()?;
            if seq_scaling_matrix_present {
                todo!("scaling matrix");
            }
        }

        let log2_max_frame_num_minus4 = reader.read_exp()?;
        let pic_order_cnt_type = reader.read_exp()?;

        let mut log2_max_pic_order_cnt_lsb_minus4 = 0;
        let mut delta_pic_order_always_zero = false;
        let mut offset_for_non_ref_pic = 0;
        let mut offset_for_top_to_bottom_field = 0;
        let mut num_ref_frames_in_pic_order_cnt_cycle = 0;
        let mut offset_for_ref_frame = Vec::new();

        if pic_order_cnt_type == 0 {
            log2_max_pic_order_cnt_lsb_minus4 = reader.read_exp()?;
        } else if pic_order_cnt_type == 1 {
            delta_pic_order_always_zero = reader.read_bit()?;
            offset_for_non_ref_pic = reader.read_exp_signed()?;
            offset_for_top_to_bottom_field = reader.read_exp_signed()?;
            num_ref_frames_in_pic_order_cnt_cycle = reader.read_exp()?;
            for _ in 0..num_ref_frames_in_pic_order_cnt_cycle {
                offset_for_ref_frame.push(reader.read_exp_signed()?);
            }
        }

        let max_num_ref_frames = reader.read_exp()?;
        let gaps_in_frame_num_value_allowed = reader.read_bit()?;
        let pic_width_in_mbs_minus1 = reader.read_exp()?;
        let pic_height_in_map_units_minus1 = reader.read_exp()?;
        let frame_mbs_only = reader.read_bit()?;

        let mut mb_adaptive_frame_field = false;
        if !frame_mbs_only {
            mb_adaptive_frame_field = reader.read_bit()?;
        }

        let direct_8x8_inference = reader.read_bit()?;
        let frame_cropping = reader.read_bit()?;
        if frame_cropping {
            todo!("frame cropping");
        }

        let vui_parameters_present = reader.read_bit()?;
        if vui_parameters_present {
            todo!("vui parameters");
        }

        Ok(Self {
            profile_idc,
            constrain_set_flags: [
                constraint_set0_flag,
                constraint_set1_flag,
                constraint_set2_flag,
                constraint_set3_flag,
                constraint_set4_flag,
                constraint_set5_flag,
            ],
            level_idc: Level::parse(level_idc as u8),
            seq_parameter_set_id,
            chroma_format,
            separate_colour_plane,
            bit_depth_luma_minus_8,
            bit_depth_chroma_minus_8,
            qpprime_y_zero_transform_bypass,
            seq_scaling_matrix_present,
            log2_max_frame_num_minus4,
            pic_order_cnt_type,
            log2_max_pic_order_cnt_lsb_minus4,
            delta_pic_order_always_zero,
            offset_for_non_ref_pic,
            offset_for_top_to_bottom_field,
            num_ref_frames_in_pic_order_cnt_cycle,
            offset_for_ref_frame,
            max_num_ref_frames,
            gaps_in_frame_num_value_allowed,
            pic_width_in_mbs_minus1,
            pic_height_in_map_units_minus1,
            frame_mbs_only,
            mb_adaptive_frame_field,
            direct_8x8_inference,
            frame_cropping,
            vui_parameters_present,
        })
    }
}

impl From<Sps> for vk::native::StdVideoH264SequenceParameterSet {
    fn from(value: Sps) -> Self {
        let mut flags: vk::native::StdVideoH264SpsFlags = unsafe { std::mem::zeroed() };
        flags.set_constraint_set0_flag(value.constrain_set_flags[0] as u32);
        flags.set_constraint_set1_flag(value.constrain_set_flags[1] as u32);
        flags.set_constraint_set2_flag(value.constrain_set_flags[2] as u32);
        flags.set_constraint_set3_flag(value.constrain_set_flags[3] as u32);
        flags.set_constraint_set4_flag(value.constrain_set_flags[4] as u32);
        flags.set_constraint_set5_flag(value.constrain_set_flags[5] as u32);
        flags.set_delta_pic_order_always_zero_flag(value.delta_pic_order_always_zero as u32);
        flags.set_direct_8x8_inference_flag(value.direct_8x8_inference as u32);
        flags.set_frame_cropping_flag(value.frame_cropping as u32);
        flags.set_frame_mbs_only_flag(value.frame_mbs_only as u32);
        flags
            .set_gaps_in_frame_num_value_allowed_flag(value.gaps_in_frame_num_value_allowed as u32);
        flags.set_mb_adaptive_frame_field_flag(value.mb_adaptive_frame_field as u32);
        flags
            .set_qpprime_y_zero_transform_bypass_flag(value.qpprime_y_zero_transform_bypass as u32);
        flags.set_separate_colour_plane_flag(value.separate_colour_plane as u32);
        flags.set_seq_scaling_matrix_present_flag(value.seq_scaling_matrix_present as u32);
        flags.set_vui_parameters_present_flag(value.vui_parameters_present as u32);

        vk::native::StdVideoH264SequenceParameterSet {
            flags,
            profile_idc: value.profile_idc.into(),
            level_idc: value.level_idc.into(),
            chroma_format_idc: value.chroma_format as u32,
            seq_parameter_set_id: value.seq_parameter_set_id as u8,
            bit_depth_luma_minus8: value.bit_depth_luma_minus_8 as u8,
            bit_depth_chroma_minus8: value.bit_depth_chroma_minus_8 as u8,
            log2_max_frame_num_minus4: value.log2_max_frame_num_minus4 as u8,
            pic_order_cnt_type: value.pic_order_cnt_type as u32,
            offset_for_non_ref_pic: value.offset_for_non_ref_pic as i32,
            offset_for_top_to_bottom_field: value.offset_for_top_to_bottom_field as i32,
            log2_max_pic_order_cnt_lsb_minus4: value.log2_max_pic_order_cnt_lsb_minus4 as u8,
            num_ref_frames_in_pic_order_cnt_cycle: value.num_ref_frames_in_pic_order_cnt_cycle
                as u8,
            max_num_ref_frames: value.max_num_ref_frames as u8,
            reserved1: 0,
            pic_width_in_mbs_minus1: value.pic_width_in_mbs_minus1 as u32,
            pic_height_in_map_units_minus1: value.pic_height_in_map_units_minus1 as u32,
            // TODO: Frame cropping
            frame_crop_left_offset: 0,
            frame_crop_right_offset: 0,
            frame_crop_top_offset: 0,
            frame_crop_bottom_offset: 0,
            reserved2: 0,
            pOffsetForRefFrame: value
                .offset_for_ref_frame
                .iter()
                .map(|&off| off as i32)
                .collect::<Vec<_>>()
                .leak()
                .as_ptr(),
            pScalingLists: core::ptr::null(),
            pSequenceParameterSetVui: core::ptr::null(),
        }
    }
}
