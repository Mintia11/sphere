use std::sync::Arc;

use ash::vk;
use gpu_allocator::vulkan::Allocation;

use crate::{command_buffer::CommandBuffer, device::Device};

pub struct Image {
    image: vk::Image,
    view: vk::ImageView,

    device: Arc<Device>,
    allocation: Option<Allocation>,
}

impl Image {
    pub(crate) fn from_parts_without_allocation(
        image: vk::Image,
        view: vk::ImageView,
        device: Arc<Device>,
    ) -> Image {
        Self {
            image,
            view,

            device,
            allocation: None,
        }
    }

    pub fn handle(&self) -> vk::Image {
        self.image
    }
}

impl CommandBuffer {
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
                self.handle(),
                &vk::DependencyInfo::default()
                    .image_memory_barriers(std::slice::from_ref(&image_memory_barrier)),
            )
        }
    }
}

impl Drop for Image {
    fn drop(&mut self) {
        unsafe {
            self.device.handle().destroy_image_view(self.view, None);
            self.device.handle().destroy_image(self.image, None);
        }
    }
}
