use std::{ops::Deref, sync::Arc};

use ash::vk;

use crate::{
    Device, buffer::Buffer, command_buffer::CommandBuffer, error::Error,
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
    pub fn begin_videocoding(&self, video_session: &VideoSession) {
        let begin_info = vk::VideoBeginCodingInfoKHR::default()
            .video_session(video_session.handle())
            .video_session_parameters(video_session.params());

        unsafe {
            self.device
                .video_queue_ext()
                .cmd_begin_video_coding(self.handle, &begin_info);
        }
    }

    pub fn decode(&self) {
        let decode_info = vk::VideoDecodeInfoKHR::default();

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
