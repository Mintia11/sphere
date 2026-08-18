use ash::khr;
pub use ash::{self, vk};
pub use gpu_allocator;

use crate::builder::GPUContextBuilder;

pub mod builder;
pub mod codec;
mod error;

pub struct GPUContext {
    _entry: ash::Entry,
    instance: ash::Instance,
    physical_device: vk::PhysicalDevice,
    device: ash::Device,

    surface: vk::SurfaceKHR,
    surface_ext: khr::surface::Instance,

    swapchain: vk::SwapchainKHR,
    swapchain_ext: khr::swapchain::Device,

    graphics_queue: (vk::Queue, u32),
    present_queue: (vk::Queue, u32),
    decode_queue: (vk::Queue, u32),
}

impl GPUContext {
    pub fn builder() -> GPUContextBuilder {
        GPUContextBuilder::default()
    }
}
