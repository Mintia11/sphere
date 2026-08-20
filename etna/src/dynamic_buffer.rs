use std::sync::Arc;

use ash::vk;
use gpu_allocator::MemoryLocation;

use crate::{Device, buffer::Buffer, error::Error};

pub struct DynamicBuffer {
    buffer: Buffer,
    capacity: usize,
    usage: vk::BufferUsageFlags,
}

impl DynamicBuffer {
    pub fn new(
        device: &Arc<Device>,
        usage: vk::BufferUsageFlags,
        initial_capacity: usize,
    ) -> Result<Self, Error> {
        let buffer =
            device.create_buffer(initial_capacity as u64, usage, MemoryLocation::CpuToGpu)?;
        Ok(Self {
            buffer,
            capacity: initial_capacity,
            usage,
        })
    }

    pub fn ensure_capacity(&mut self, device: &Arc<Device>, needed: usize) -> Result<(), Error> {
        if needed <= self.capacity {
            return Ok(());
        }
        let new_capacity = (needed * 2).max(self.capacity * 2);
        self.buffer =
            device.create_buffer(new_capacity as u64, self.usage, MemoryLocation::CpuToGpu)?;
        self.capacity = new_capacity;
        Ok(())
    }

    pub fn write(&mut self, offset: usize, data: &[u8]) -> Result<(), Error> {
        self.buffer.write(offset, data)
    }

    pub fn handle(&self) -> vk::Buffer {
        self.buffer.handle()
    }
}
