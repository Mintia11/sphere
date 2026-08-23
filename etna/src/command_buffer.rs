use std::sync::Arc;

use ash::vk;

use crate::{Device, error::Error};

pub struct CommandBuffer {
    pub(crate) handle: vk::CommandBuffer,

    pub(crate) device: Arc<Device>,
}

impl Device {
    pub fn allocate_graphics_command_buffer(self: &Arc<Self>) -> Result<CommandBuffer, Error> {
        let handles = unsafe {
            self.handle().allocate_command_buffers(
                &vk::CommandBufferAllocateInfo::default()
                    .command_buffer_count(1)
                    .command_pool(self.graphics_queue().command_pool())
                    .level(vk::CommandBufferLevel::PRIMARY),
            )?
        };

        Ok(CommandBuffer {
            handle: handles[0],
            device: self.clone(),
        })
    }

    pub fn allocate_transfer_command_buffer(self: &Arc<Self>) -> Result<CommandBuffer, Error> {
        let handles = unsafe {
            self.handle().allocate_command_buffers(
                &vk::CommandBufferAllocateInfo::default()
                    .command_buffer_count(1)
                    .command_pool(self.transfer_queue().command_pool())
                    .level(vk::CommandBufferLevel::PRIMARY),
            )?
        };

        Ok(CommandBuffer {
            handle: handles[0],
            device: self.clone(),
        })
    }
}

impl CommandBuffer {
    pub fn handle(&self) -> vk::CommandBuffer {
        self.handle
    }

    pub fn reset(&self) -> Result<(), Error> {
        unsafe {
            self.device
                .handle()
                .reset_command_buffer(self.handle, vk::CommandBufferResetFlags::empty())
                .map_err(Into::into)
        }
    }

    pub fn begin(&self) -> Result<(), Error> {
        unsafe {
            self.device
                .handle()
                .begin_command_buffer(self.handle, &vk::CommandBufferBeginInfo::default())
                .map_err(Into::into)
        }
    }

    pub fn end(&self) -> Result<(), Error> {
        unsafe {
            self.device
                .handle()
                .end_command_buffer(self.handle)
                .map_err(Into::into)
        }
    }
}
