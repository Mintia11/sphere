use std::{ops::Deref, sync::Arc};

use ash::vk::{self, Extends, TaggedStructure};

use crate::{
    Device, Image, buffer::Buffer, command_buffer::CommandBuffer, error::Error,
    video::session::VideoSession,
};

pub struct DecodeCommandBuffer {
    pub(crate) inner: CommandBuffer,
}

impl Deref for DecodeCommandBuffer {
    type Target = CommandBuffer;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DecodeCommandBuffer {
    pub fn begin_videocoding(
        &self,
        video_session: &VideoSession,
        reference_slots: &[vk::VideoReferenceSlotInfoKHR],
    ) {
        let begin_info = vk::VideoBeginCodingInfoKHR::default()
            .reference_slots(reference_slots)
            .video_session(video_session.handle())
            .video_session_parameters(video_session.params());

        unsafe {
            self.device
                .video_queue_ext()
                .cmd_begin_video_coding(self.handle, &begin_info);
        }
    }

    pub fn reset_session(&self) {
        let coding_control_info =
            vk::VideoCodingControlInfoKHR::default().flags(vk::VideoCodingControlFlagsKHR::RESET);

        unsafe {
            self.device
                .video_queue_ext()
                .cmd_control_video_coding(self.handle, &coding_control_info);
        }
    }

    pub fn decode<'a, T>(
        &self,
        src: &Buffer,
        dst: &Image,
        reference_slots: &'a [vk::VideoReferenceSlotInfoKHR],
        setup_reference_slot: &'a vk::VideoReferenceSlotInfoKHR,
        next: &'a mut T,
    ) where
        T: Extends<vk::VideoDecodeInfoKHR<'a>> + TaggedStructure<'a>,
    {
        let decode_info = vk::VideoDecodeInfoKHR::default()
            .dst_picture_resource(
                vk::VideoPictureResourceInfoKHR::default()
                    .base_array_layer(0)
                    .coded_extent(dst.extent_2d())
                    .image_view_binding(dst.view()),
            )
            .reference_slots(reference_slots)
            .setup_reference_slot(setup_reference_slot)
            .src_buffer(src.handle())
            .src_buffer_offset(0)
            .src_buffer_range(src.size())
            .push(next);

        unsafe {
            self.device
                .video_decode_queue_ext()
                .cmd_decode_video(self.handle, &decode_info);
        }
    }

    pub fn end_videocoding(&self) {
        let end_info = vk::VideoEndCodingInfoKHR::default();

        unsafe {
            self.device
                .video_queue_ext()
                .cmd_end_video_coding(self.handle, &end_info);
        }
    }
}

impl Device {
    pub fn allocate_decode_command_buffer(self: &Arc<Self>) -> Result<DecodeCommandBuffer, Error> {
        let handles = unsafe {
            self.handle().allocate_command_buffers(
                &vk::CommandBufferAllocateInfo::default()
                    .command_buffer_count(1)
                    .command_pool(self.decode_queue().command_pool())
                    .level(vk::CommandBufferLevel::PRIMARY),
            )?
        };

        Ok(DecodeCommandBuffer {
            inner: CommandBuffer {
                handle: handles[0],
                device: self.clone(),
            },
        })
    }
}
