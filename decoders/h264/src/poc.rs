use crate::{slice_header::SliceHeader, sps::Sps};

#[derive(Default)]
pub struct PocDecoderState {
    pub prev_poc_msb: i64,
    pub prev_poc_lsb: i64,
    pub prev_frame_num: u64,
    pub prev_frame_num_offset: i64,
}

impl PocDecoderState {
    pub fn compute_and_update(
        &mut self,
        sps: &Sps,
        slice: &SliceHeader,
        is_idr: bool,
        nal_ref_idc: u64,
    ) -> i32 {
        assert_eq!(
            sps.pic_order_cnt_type, 0,
            "only pic_order_cnt_type 0 is currently implemented"
        );

        let is_reference = nal_ref_idc != 0;
        let max_poc_lsb: i64 = 1 << (sps.log2_max_pic_order_cnt_lsb_minus4 as i64 + 4);

        let current_poc_msb = if is_idr {
            0
        } else {
            let cur_lsb = slice.pic_order_cnt_lsb as i64;
            let prev_lsb = self.prev_poc_lsb;
            let prev_msb = self.prev_poc_msb;

            if cur_lsb < prev_lsb && (prev_lsb - cur_lsb) >= (max_poc_lsb / 2) {
                prev_msb + max_poc_lsb
            } else if cur_lsb > prev_lsb && (cur_lsb - prev_lsb) > (max_poc_lsb / 2) {
                prev_msb - max_poc_lsb
            } else {
                prev_msb
            }
        };

        let poc = current_poc_msb + slice.pic_order_cnt_lsb as i64;

        let max_frame_num: i64 = 1 << (sps.log2_max_frame_num_minus4 as i64 + 4);
        let current_frame_num_offset = if is_idr {
            0
        } else if self.prev_frame_num > slice.frame_num {
            self.prev_frame_num_offset + max_frame_num
        } else {
            self.prev_frame_num_offset
        };

        if is_reference {
            self.prev_poc_msb = current_poc_msb;
            self.prev_poc_lsb = slice.pic_order_cnt_lsb as i64;
        }

        if is_idr {
            self.prev_frame_num = 0;
            self.prev_frame_num_offset = 0;
        } else {
            self.prev_frame_num_offset = current_frame_num_offset;
            self.prev_frame_num = slice.frame_num;
        }

        poc as i32
    }
}
