use std::sync::Arc;

use etna::Image;

pub struct DpbTracker {
    slots: Vec<DpbSlot>,
    max_ref_frames: usize,
}

pub struct DpbSlot {
    pub slot_idx: DpbSlotIdx,
    pub image: Arc<Image>,
    pub is_used_for_ref: bool,
    pub frame_num: usize,
    pub pic_order_cnt: [i32; 2],
    pub was_ever_activated: bool,
}

type DpbSlotIdx = usize;

impl DpbTracker {
    pub fn new(
        max_ref_frames: usize,
        max_dpb_slots: usize,
        create_image: impl Fn() -> Arc<Image>,
    ) -> DpbTracker {
        let mut slots = Vec::with_capacity(max_dpb_slots);
        for i in 0..max_dpb_slots {
            slots.push(DpbSlot {
                slot_idx: i,
                image: create_image(),
                is_used_for_ref: false,
                frame_num: 0,
                pic_order_cnt: [0, 0],
                was_ever_activated: false,
            })
        }

        DpbTracker {
            slots,
            max_ref_frames,
        }
    }

    pub fn find_free_slot(&self) -> Option<DpbSlotIdx> {
        self.slots.iter().position(|slot| !slot.is_used_for_ref)
    }

    pub fn slots(&self) -> &[DpbSlot] {
        &self.slots
    }

    pub fn begin_slot_index(&self, slot: DpbSlotIdx) -> i32 {
        if self.slots[slot].was_ever_activated {
            slot as i32
        } else {
            -1
        }
    }

    pub fn apply_sliding_window(&mut self) {
        let ref_count = self.slots.iter().filter(|s| s.is_used_for_ref).count();
        if ref_count >= self.max_ref_frames
            && let Some(oldest) = self
                .slots
                .iter_mut()
                .filter(|s| s.is_used_for_ref)
                .min_by_key(|s| s.frame_num)
        {
            oldest.is_used_for_ref = false;
        }
    }

    pub fn register_frame(
        &mut self,
        slot_idx: DpbSlotIdx,
        frame_num: usize,
        is_reference: bool,
        pic_order_cnt: [i32; 2],
    ) {
        if let Some(slot) = self.slots.get_mut(slot_idx) {
            slot.frame_num = frame_num;
            slot.is_used_for_ref = is_reference;
            slot.was_ever_activated = true;
            slot.pic_order_cnt = pic_order_cnt;
        }
    }

    pub fn flush_idr(&mut self) {
        for slot in &mut self.slots {
            slot.is_used_for_ref = false;
        }
    }
}
