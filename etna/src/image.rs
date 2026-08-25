use std::sync::{
    Arc,
    atomic::{AtomicI32, Ordering},
};

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

    current_layout: AtomicI32,

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
            .flags(vk::ImageCreateFlags::VIDEO_PROFILE_INDEPENDENT_KHR)
            .image_type(vk::ImageType::TYPE_2D)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .mip_levels(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(usage | vk::ImageUsageFlags::TRANSFER_DST);

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

        let bind_info = vk::BindImageMemoryInfo::default()
            .image(image)
            .memory(unsafe { allocation.memory() })
            .memory_offset(allocation.offset());
        unsafe { self.handle().bind_image_memory2(&[bind_info])? };

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
            current_layout: AtomicI32::new(vk::ImageLayout::UNDEFINED.as_raw()),
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

            current_layout: AtomicI32::new(vk::ImageLayout::UNDEFINED.as_raw()),

            allocation: None,
            device,
        }
    }

    pub fn upload(&mut self, data: &[u8]) -> Result<(), Error> {
        let buffer = self.device.create_and_upload_buffer(
            data,
            vk::BufferUsageFlags::TRANSFER_SRC,
            MemoryLocation::CpuToGpu,
        )?;

        let command_buffer = self.device.allocate_transfer_command_buffer()?;
        command_buffer.begin()?;
        command_buffer.buffer_pipeline_barrier(
            &buffer,
            vk::AccessFlags2::NONE,
            vk::AccessFlags2::TRANSFER_READ,
            vk::PipelineStageFlags2::NONE,
            vk::PipelineStageFlags2::COPY,
        );
        command_buffer.image_pipeline_barrier(
            self,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            vk::AccessFlags2::NONE,
            vk::AccessFlags2::TRANSFER_WRITE,
            vk::PipelineStageFlags2::ALL_TRANSFER,
            vk::PipelineStageFlags2::ALL_TRANSFER,
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

    pub fn upload_region(
        &mut self,
        data: &[u8],
        offset: [u32; 2],
        extent: [u32; 2],
    ) -> Result<(), Error> {
        let buffer = self.device.create_and_upload_buffer(
            data,
            vk::BufferUsageFlags::TRANSFER_SRC,
            MemoryLocation::CpuToGpu,
        )?;

        let command_buffer = self.device.allocate_transfer_command_buffer()?;
        command_buffer.begin()?;
        command_buffer.buffer_pipeline_barrier(
            &buffer,
            vk::AccessFlags2::NONE,
            vk::AccessFlags2::TRANSFER_READ,
            vk::PipelineStageFlags2::NONE,
            vk::PipelineStageFlags2::COPY,
        );

        let (src_access_mask, src_stage_mask) = match self.current_layout() {
            vk::ImageLayout::UNDEFINED => {
                (vk::AccessFlags2::NONE, vk::PipelineStageFlags2::TOP_OF_PIPE)
            }
            vk::ImageLayout::TRANSFER_DST_OPTIMAL => (
                vk::AccessFlags2::TRANSFER_WRITE,
                vk::PipelineStageFlags2::ALL_TRANSFER,
            ),
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL => (
                vk::AccessFlags2::SHADER_READ,
                vk::PipelineStageFlags2::FRAGMENT_SHADER,
            ),
            other => panic!("upload_region: unexpected image layout {other:?}"),
        };

        command_buffer.image_pipeline_barrier(
            self,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            src_access_mask,
            vk::AccessFlags2::TRANSFER_WRITE,
            src_stage_mask,
            vk::PipelineStageFlags2::ALL_TRANSFER,
        );
        command_buffer.copy_buffer_to_image_region(&buffer, self, offset, extent);
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

    pub fn view(&self) -> vk::ImageView {
        self.view
    }

    pub fn extent(&self) -> vk::Extent3D {
        self.extent
    }

    pub fn extent_2d(&self) -> vk::Extent2D {
        vk::Extent2D::default()
            .width(self.extent.width)
            .height(self.extent.height)
    }

    pub fn current_layout(&self) -> vk::ImageLayout {
        vk::ImageLayout::from_raw(self.current_layout.load(Ordering::SeqCst))
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
                    .layer_count(1)
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

    pub fn copy_buffer_to_image_region(
        &self,
        from: &Buffer,
        to: &mut Image,
        offset: [u32; 2],
        extent: [u32; 2],
    ) {
        let region = vk::BufferImageCopy2::default()
            .buffer_image_height(0)
            .buffer_offset(0)
            .buffer_row_length(0)
            .image_offset(vk::Offset3D {
                x: offset[0] as i32,
                y: offset[1] as i32,
                z: 0,
            })
            .image_extent(vk::Extent3D {
                width: extent[0],
                height: extent[1],
                depth: 1,
            })
            .image_subresource(
                vk::ImageSubresourceLayers::default()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .base_array_layer(0)
                    .layer_count(1)
                    .mip_level(0),
            );

        let copy_info = vk::CopyBufferToImageInfo2::default()
            .dst_image(to.handle())
            .dst_image_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .regions(std::slice::from_ref(&region))
            .src_buffer(from.handle());

        unsafe {
            self.device
                .handle()
                .cmd_copy_buffer_to_image2(self.handle(), &copy_info);
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn image_pipeline_barrier(
        &self,
        image: &Image,
        new_layout: vk::ImageLayout,
        src_access_mask: vk::AccessFlags2,
        dst_access_mask: vk::AccessFlags2,
        src_stage_mask: vk::PipelineStageFlags2,
        dst_stage_mask: vk::PipelineStageFlags2,
    ) {
        let image_memory_barrier = vk::ImageMemoryBarrier2::default()
            .dst_access_mask(dst_access_mask)
            .dst_stage_mask(dst_stage_mask)
            .image(image.handle())
            .new_layout(new_layout)
            .old_layout(image.current_layout())
            .src_access_mask(src_access_mask)
            .src_stage_mask(src_stage_mask)
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

        image
            .current_layout
            .store(new_layout.as_raw(), Ordering::SeqCst);
    }
}

impl Drop for Image {
    fn drop(&mut self) {
        if let Some(allocation) = self.allocation.take() {
            let mut allocator = self.device.allocator().lock().unwrap();
            let _ = allocator.free(allocation);

            // if the image hasn't got an allocation it's probably a swapchain image so don't destroy it
            unsafe {
                self.device.handle().destroy_image_view(self.view, None);
                self.device.handle().destroy_image(self.image, None);
            }
        }
    }
}
