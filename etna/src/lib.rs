use std::sync::{Arc, Mutex};

use ash::khr;
pub use ash::{self, vk};
pub use gpu_allocator;
use gpu_allocator::vulkan::Allocator;

use crate::{builder::GPUContextBuilder, error::Error};

pub mod builder;
pub mod codec;
pub mod error;
pub mod image;
pub mod surface;
pub mod swapchain;

pub struct GPUContext {
    _entry: ash::Entry,
    instance: ash::Instance,
    physical_device: vk::PhysicalDevice,
    device: ash::Device,

    allocator: Arc<Mutex<Allocator>>,

    cmd_pool: vk::CommandPool,
    transfer_cmd_buffer: vk::CommandBuffer,

    descriptor_pool: vk::DescriptorPool,

    graphics_queue: (vk::Queue, u32),
    present_queue: (vk::Queue, u32),
    decode_queue: (vk::Queue, u32),
}

impl GPUContext {
    pub fn builder() -> GPUContextBuilder {
        GPUContextBuilder::default()
    }
}
