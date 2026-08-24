use common::{bit_io::BitReader, byte_io::ByteReader, packet::Error};
use etna::vk;

use crate::nal::RawNal;

pub struct Pps {
    pub pic_parameter_set_id: u64,
    pub seq_parameter_set_id: u64,
    pub entropy_coding_mode: bool,
    pub bottom_field_pic_order_in_frame_present: bool,
    pub num_slice_groups_minus1: u64,
    pub num_ref_idx_l0_default_active_minus1: u64,
    pub num_ref_idx_l1_default_active_minus1: u64,
    pub weighted_pred: bool,
    pub weighted_bipred_idc: u64,
    pub pic_init_qp_minus26: i64,
    pub pic_init_qs_minus26: i64,
    pub chroma_qp_index_offset: i64,
    pub deblocking_filter_control_present: bool,
    pub constrained_intra_pred: bool,
    pub redundant_pic_cnt_present: bool,
}

impl Pps {
    pub fn parse(nal: &RawNal) -> Result<Self, Error> {
        let reader = ByteReader::new(nal.strip_emulation_prevention());
        let mut reader = BitReader::new(reader);

        let pic_parameter_set_id = reader.read_exp()?;
        let seq_parameter_set_id = reader.read_exp()?;
        let entropy_coding_mode = reader.read_bit()?;
        let bottom_field_pic_order_in_frame_present = reader.read_bit()?;
        let num_slice_groups_minus1 = reader.read_exp()?;
        if num_slice_groups_minus1 > 0 {
            todo!("slice groups");
        }

        let num_ref_idx_l0_default_active_minus1 = reader.read_exp()?;
        let num_ref_idx_l1_default_active_minus1 = reader.read_exp()?;
        let weighted_pred = reader.read_bit()?;
        let weighted_bipred_idc = reader.read_bits(2)?;
        let pic_init_qp_minus26 = reader.read_exp_signed()?;
        let pic_init_qs_minus26 = reader.read_exp_signed()?;
        let chroma_qp_index_offset = reader.read_exp_signed()?;
        let deblocking_filter_control_present = reader.read_bit()?;
        let constrained_intra_pred = reader.read_bit()?;
        let redundant_pic_cnt_present = reader.read_bit()?;

        if !reader.eof() {
            todo!("parse the rest of the pps");
        }

        Ok(Self {
            pic_parameter_set_id,
            seq_parameter_set_id,
            entropy_coding_mode,
            bottom_field_pic_order_in_frame_present,
            num_slice_groups_minus1,
            num_ref_idx_l0_default_active_minus1,
            num_ref_idx_l1_default_active_minus1,
            weighted_pred,
            weighted_bipred_idc,
            pic_init_qp_minus26,
            pic_init_qs_minus26,
            chroma_qp_index_offset,
            deblocking_filter_control_present,
            constrained_intra_pred,
            redundant_pic_cnt_present,
        })
    }
}

impl From<Pps> for vk::native::StdVideoH264PictureParameterSet {
    fn from(value: Pps) -> Self {
        let mut flags: vk::native::StdVideoH264PpsFlags = unsafe { std::mem::zeroed() };
        flags.set_bottom_field_pic_order_in_frame_present_flag(
            value.bottom_field_pic_order_in_frame_present as u32,
        );
        flags.set_constrained_intra_pred_flag(value.constrained_intra_pred as u32);
        flags.set_deblocking_filter_control_present_flag(
            value.deblocking_filter_control_present as u32,
        );
        flags.set_entropy_coding_mode_flag(value.entropy_coding_mode as u32);
        flags.set_pic_scaling_matrix_present_flag(false as u32); // rest of the pps
        flags.set_redundant_pic_cnt_present_flag(value.redundant_pic_cnt_present as u32);
        flags.set_transform_8x8_mode_flag(false as u32); // rest of the pps
        flags.set_transform_8x8_mode_flag(false as u32); // |
        flags.set_weighted_pred_flag(value.weighted_pred as u32);

        vk::native::StdVideoH264PictureParameterSet {
            flags,
            seq_parameter_set_id: value.seq_parameter_set_id as u8,
            pic_parameter_set_id: value.pic_parameter_set_id as u8,
            num_ref_idx_l0_default_active_minus1: value.num_ref_idx_l0_default_active_minus1 as u8,
            num_ref_idx_l1_default_active_minus1: value.num_ref_idx_l1_default_active_minus1 as u8,
            weighted_bipred_idc: value.weighted_bipred_idc as u32,
            pic_init_qp_minus26: value.pic_init_qp_minus26 as i8,
            pic_init_qs_minus26: value.pic_init_qs_minus26 as i8,
            chroma_qp_index_offset: value.chroma_qp_index_offset as i8,
            second_chroma_qp_index_offset: 0, // rest of the pps
            pScalingLists: core::ptr::null(), // |
        }
    }
}
