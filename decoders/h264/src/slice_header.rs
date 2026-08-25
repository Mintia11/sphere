use common::{bit_io::BitReader, byte_io::ByteReader, packet::Error};
use etna::vk;

use crate::{pps::Pps, sps::Sps};

#[derive(Clone, Copy)]
pub struct SliceHeader {
    pub first_mb_in_slice: u64,
    pub slice_type: u64,
    pub pic_parameter_set_id: u64,
    pub color_plane_id: u64,
    pub frame_num: u64,
    pub field_pic: bool,
    pub bottom_field: bool,
    pub idr_pic_id: u64,
    pub pic_order_cnt_lsb: u64,
    pub delta_pic_order_cnt_bottom: i64,
    pub delta_pic_order_cnt: [i64; 2],
}

impl SliceHeader {
    pub fn parse(data: &[u8], sps: &Sps, pps: &Pps, is_idr: bool) -> Result<Self, Error> {
        let reader = ByteReader::new(data);
        let mut reader = BitReader::new(reader);

        let first_mb_in_slice = reader.read_exp()?;
        let slice_type = reader.read_exp()?;
        let pic_parameter_set_id = reader.read_exp()?;

        let mut color_plane_id = 0;
        if sps.separate_colour_plane {
            color_plane_id = reader.read_bits(2)?;
        }

        let frame_num = reader.read_bits(sps.log2_max_frame_num_minus4 as u32 + 4)?;

        let mut field_pic = false;
        let mut bottom_field = false;
        if !sps.frame_mbs_only {
            field_pic = reader.read_bit()?;
            if field_pic {
                bottom_field = reader.read_bit()?;
            }
        }

        let mut idr_pic_id = 0;
        if is_idr {
            idr_pic_id = reader.read_exp()?;
        }

        let mut pic_order_cnt_lsb = 0;
        let mut delta_pic_order_cnt_bottom = 0;
        if sps.pic_order_cnt_type == 0 {
            pic_order_cnt_lsb =
                reader.read_bits(sps.log2_max_pic_order_cnt_lsb_minus4 as u32 + 4)?;
            if pps.bottom_field_pic_order_in_frame_present && !field_pic {
                delta_pic_order_cnt_bottom = reader.read_exp_signed()?;
            }
        }

        let mut delta_pic_order_cnt: [i64; 2] = [0; 2];
        if sps.pic_order_cnt_type == 1 && !sps.delta_pic_order_always_zero {
            delta_pic_order_cnt[0] = reader.read_exp_signed()?;
            if pps.bottom_field_pic_order_in_frame_present && !field_pic {
                delta_pic_order_cnt[1] = reader.read_exp_signed()?;
            }
        }

        Ok(Self {
            first_mb_in_slice,
            slice_type,
            pic_parameter_set_id,
            color_plane_id,
            frame_num,
            field_pic,
            bottom_field,
            idr_pic_id,
            pic_order_cnt_lsb,
            delta_pic_order_cnt_bottom,
            delta_pic_order_cnt,
        })
    }

    pub fn into_picture_info(
        self,
        sps: &Sps,
        is_idr: bool,
        is_reference: bool,
        poc: i32,
    ) -> vk::native::StdVideoDecodeH264PictureInfo {
        let mut flags: vk::native::StdVideoDecodeH264PictureInfoFlags =
            unsafe { std::mem::zeroed() };
        flags.set_IdrPicFlag(is_idr as u32);
        flags.set_bottom_field_flag(self.bottom_field as u32);
        flags.set_complementary_field_pair(false as u32); // todo: what is this?
        flags.set_field_pic_flag(self.field_pic as u32);
        let is_intra = is_idr || (self.slice_type % 5 == 2);
        flags.set_is_intra(is_intra as u32);
        flags.set_is_reference(is_reference as u32);

        vk::native::StdVideoDecodeH264PictureInfo {
            flags,
            seq_parameter_set_id: sps.seq_parameter_set_id as u8,
            pic_parameter_set_id: self.pic_parameter_set_id as u8,
            reserved1: 0,
            reserved2: 0,
            frame_num: self.frame_num as u16,
            idr_pic_id: self.idr_pic_id as u16,
            PicOrderCnt: [poc; 2],
        }
    }
}
