use std::sync::Arc;

use ash::vk;
use gpu_allocator::{
    MemoryLocation,
    vulkan::{Allocation, AllocationCreateDesc, AllocationScheme},
};

use crate::{buffer::Buffer, command_buffer::CommandBuffer, device::Device, error::Error};

pub struct Image {
    image: vk::Image,
    view: vk::ImageView,

    extent: vk::Extent3D,

    allocation: Option<Allocation>,

    device: Arc<Device>,
}

impl Device {
    pub fn create_image(
        self: &Arc<Self>,
        width: u32,
        height: u32,
        format: vk::Format,
        usage: vk::ImageUsageFlags,
        location: MemoryLocation,
    ) -> Result<Image, Error> {
        let extent = vk::Extent3D::default().width(width).height(height).depth(1);

        let image_info = vk::ImageCreateInfo::default()
            .array_layers(1)
            .extent(extent)
            .format(format)
            .image_type(vk::ImageType::TYPE_2D)
            .initial_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .mip_levels(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(usage);

        let image = unsafe { self.handle().create_image(&image_info, None)? };

        let alloc_info = AllocationCreateDesc {
            name: "Generic image",
            requirements: unsafe { self.handle().get_image_memory_requirements(image) },
            location,
            linear: true,
            allocation_scheme: AllocationScheme::DedicatedImage(image),
        };

        let mut allocator = self.allocator().lock().unwrap();
        let allocation = allocator.allocate(&alloc_info)?;

        let image_view_info = vk::ImageViewCreateInfo::default()
            .components(
                vk::ComponentMapping::default()
                    .r(vk::ComponentSwizzle::IDENTITY)
                    .g(vk::ComponentSwizzle::IDENTITY)
                    .b(vk::ComponentSwizzle::IDENTITY),
            )
            .format(format)
            .image(image)
            .subresource_range(
                vk::ImageSubresourceRange::default()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .base_array_layer(0)
                    .base_mip_level(0)
                    .layer_count(1)
                    .level_count(1),
            )
            .view_type(vk::ImageViewType::TYPE_2D);

        let view = unsafe { self.handle().create_image_view(&image_view_info, None)? };

        Ok(Image {
            image,
            view,
            extent,
            allocation: Some(allocation),
            device: self.clone(),
        })
    }

    pub fn create_image_and_upload(
        self: &Arc<Self>,
        width: u32,
        height: u32,
        data: &[u8],
        format: vk::Format,
        usage: vk::ImageUsageFlags,
        location: MemoryLocation,
    ) -> Result<Image, Error> {
        let mut image = self.create_image(width, height, format, usage, location)?;
        image.upload(data)?;

        Ok(image)
    }
}

impl Image {
    pub(crate) fn from_parts_without_allocation(
        image: vk::Image,
        view: vk::ImageView,
        extent: vk::Extent3D,
        device: Arc<Device>,
    ) -> Image {
        Self {
            image,
            view,
            extent,

            device,
            allocation: None,
        }
    }

    pub fn upload(&mut self, data: &[u8]) -> Result<(), Error> {
        let buffer = self
            .device
            .create_and_upload_buffer(data, MemoryLocation::CpuToGpu)?;

        let command_buffer = self.device.allocate_transfer_command_buffer()?;
        command_buffer.begin()?;
        command_buffer.buffer_pipeline_barrier(
            &buffer,
            vk::AccessFlags2::NONE,
            vk::AccessFlags2::TRANSFER_READ,
        );
        command_buffer.image_pipeline_barrier(
            self,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        );
        command_buffer.copy_buffer_to_image(&buffer, self);
        command_buffer.end()?;

        let submit_fence = self.device.create_fence(false)?;
        self.device.submit_queue(
            self.device.transfer_queue(),
            &command_buffer,
            None,
            None,
            Some(&submit_fence),
        )?;
        submit_fence.wait(u64::MAX)?;

        Ok(())
    }

    pub fn handle(&self) -> vk::Image {
        self.image
    }
}

impl CommandBuffer {
    pub fn copy_buffer_to_image(&self, from: &Buffer, to: &mut Image) {
        let region = vk::BufferImageCopy2::default()
            .buffer_image_height(0)
            .buffer_offset(0)
            .buffer_row_length(0)
            .image_extent(to.extent)
            .image_subresource(
                vk::ImageSubresourceLayers::default()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .base_array_layer(0)
                    .layer_count(0)
                    .mip_level(0),
            );

        let copy_buffer_to_image_info = vk::CopyBufferToImageInfo2::default()
            .dst_image(to.handle())
            .dst_image_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .regions(std::slice::from_ref(&region))
            .src_buffer(from.handle());

        unsafe {
            self.device
                .handle()
                .cmd_copy_buffer_to_image2(self.handle(), &copy_buffer_to_image_info);
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
