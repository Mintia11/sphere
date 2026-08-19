use std::sync::Arc;

use ash::vk;

use crate::{Device, Image, error::Error};

pub struct CommandBuffer {
    handle: vk::CommandBuffer,

    device: Arc<Device>,
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

    pub fn image_pipeline_barrier(
        &self,
        image: &Image,
        old_layout: vk::ImageLayout,
        new_layout: vk::ImageLayout,
    ) {
        let image_memory_barrier = vk::ImageMemoryBarrier2::default()
            .image(image.handle())
            .new_layout(new_layout)
            .old_layout(old_layout)
            .subresource_range(
                vk::ImageSubresourceRange::default()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .base_array_layer(0)
                    .base_mip_level(0)
                    .layer_count(1)
                    .level_count(1),
            );

        unsafe {
            self.device.handle().cmd_pipeline_barrier2(
                self.handle,
                &vk::DependencyInfo::default()
                    .image_memory_barriers(std::slice::from_ref(&image_memory_barrier)),
            )
        }
    }
}
