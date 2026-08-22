use std::sync::Arc;

use ash::vk;
use gpu_allocator::{
    MemoryLocation,
    vulkan::{Allocation, AllocationCreateDesc, AllocationScheme},
};

use crate::{Device, command_buffer::CommandBuffer, error::Error};

pub struct Buffer {
    handle: vk::Buffer,
    allocation: Option<Allocation>,
    size: u64,

    device: Arc<Device>,
}

impl Device {
    pub fn create_buffer(
        self: &Arc<Self>,
        size: u64,
        usage: vk::BufferUsageFlags,
        location: MemoryLocation,
    ) -> Result<Buffer, Error> {
        let buffer_info = vk::BufferCreateInfo::default().size(size).usage(usage);
        let buffer = unsafe { self.handle().create_buffer(&buffer_info, None)? };

        let alloc_info = AllocationCreateDesc {
            name: "Generic buffer",
            requirements: unsafe { self.handle().get_buffer_memory_requirements(buffer) },
            location,
            linear: true,
            allocation_scheme: AllocationScheme::DedicatedBuffer(buffer),
        };

        let mut allocator = self.allocator().lock().unwrap();
        let allocation = allocator.allocate(&alloc_info)?;

        let bind_info = vk::BindBufferMemoryInfo::default()
            .buffer(buffer)
            .memory(unsafe { allocation.memory() })
            .memory_offset(allocation.offset());

        unsafe { self.handle().bind_buffer_memory2(&[bind_info])? };

        Ok(Buffer {
            handle: buffer,
            allocation: Some(allocation),
            size,

            device: self.clone(),
        })
    }

    pub fn create_and_upload_buffer(
        self: &Arc<Self>,
        data: &[u8],
        usage: vk::BufferUsageFlags,
        location: MemoryLocation,
    ) -> Result<Buffer, Error> {
        let mut buffer = self.create_buffer(
            data.len() as u64,
            usage | vk::BufferUsageFlags::TRANSFER_DST,
            location,
        )?;
        buffer.upload(data)?;

        Ok(buffer)
    }
}

impl Buffer {
    pub fn upload(&mut self, data: &[u8]) -> Result<(), Error> {
        // if the buffer is `HOST_VISIBLE` just copy into it
        if let Some(slice) = self.as_mut_slice() {
            slice[..data.len()].copy_from_slice(data);
            return Ok(());
        }

        let mut staging = self.device.create_buffer(
            self.size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            MemoryLocation::CpuToGpu,
        )?;
        {
            let staging = staging.as_mut_slice().unwrap();
            staging[..data.len()].copy_from_slice(data);
        }

        let command_buffer = self.device.allocate_transfer_command_buffer()?;
        command_buffer.begin()?;
        command_buffer.buffer_pipeline_barrier(
            &staging,
            vk::AccessFlags2::NONE,
            vk::AccessFlags2::TRANSFER_READ,
            vk::PipelineStageFlags2::NONE,
            vk::PipelineStageFlags2::COPY,
        );
        command_buffer.buffer_pipeline_barrier(
            self,
            vk::AccessFlags2::NONE,
            vk::AccessFlags2::TRANSFER_WRITE,
            vk::PipelineStageFlags2::NONE,
            vk::PipelineStageFlags2::COPY,
        );
        command_buffer.copy_buffers(&staging, self);
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

    pub fn as_mut_slice(&mut self) -> Option<&mut [u8]> {
        self.allocation.as_mut()?.mapped_slice_mut()
    }

    pub fn write(&mut self, offset: usize, data: &[u8]) -> Result<(), Error> {
        let slice = self.as_mut_slice().ok_or(Error::WriteToUnmapped)?;
        slice[offset..offset + data.len()].copy_from_slice(data);
        Ok(())
    }

    pub fn handle(&self) -> vk::Buffer {
        self.handle
    }
}

impl CommandBuffer {
    pub fn buffer_pipeline_barrier(
        &self,
        buffer: &Buffer,
        src_access_mask: vk::AccessFlags2,
        dst_access_mask: vk::AccessFlags2,
        src_stage_mask: vk::PipelineStageFlags2,
        dst_stage_mask: vk::PipelineStageFlags2,
    ) {
        let buffer_memory_barrier = vk::BufferMemoryBarrier2::default()
            .buffer(buffer.handle())
            .dst_access_mask(dst_access_mask)
            .dst_stage_mask(dst_stage_mask)
            .size(u64::MAX)
            .src_access_mask(src_access_mask)
            .src_stage_mask(src_stage_mask);

        unsafe {
            self.device.handle().cmd_pipeline_barrier2(
                self.handle(),
                &vk::DependencyInfo::default()
                    .buffer_memory_barriers(std::slice::from_ref(&buffer_memory_barrier)),
            )
        }
    }

    pub fn copy_buffers(&self, from: &Buffer, to: &mut Buffer) {
        let region = vk::BufferCopy2::default().size(from.size);

        unsafe {
            self.device.handle().cmd_copy_buffer2(
                self.handle(),
                &vk::CopyBufferInfo2::default()
                    .dst_buffer(to.handle())
                    .src_buffer(from.handle())
                    .regions(std::slice::from_ref(&region)),
            )
        }
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        let mut allocator = self.device.allocator().lock().unwrap();
        allocator.free(self.allocation.take().unwrap()).unwrap();

        unsafe {
            self.device.handle().destroy_buffer(self.handle, None);
        }
    }
}
