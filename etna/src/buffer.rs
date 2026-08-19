use std::sync::Arc;

use ash::vk;
use gpu_allocator::{
    MemoryLocation,
    vulkan::{Allocation, AllocationCreateDesc, AllocationScheme},
};

use crate::{Device, error::Error};

pub struct Buffer {
    handle: vk::Buffer,
    allocation: Allocation,

    device: Arc<Device>,
}

impl Device {
    pub fn create_buffer(
        self: &Arc<Self>,
        size: u64,
        location: MemoryLocation,
    ) -> Result<Buffer, Error> {
        let buffer_info = vk::BufferCreateInfo::default().size(size);
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

        Ok(Buffer {
            handle: buffer,
            allocation,

            device: self.clone(),
        })
    }
}
