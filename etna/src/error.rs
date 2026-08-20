use ash::vk;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Vulkan error: {0}")]
    Vulkan(#[from] vk::Result),

    #[error("Error while loading the vulkan driver: {0}")]
    LoadingError(#[from] ash::LoadingError),

    #[error("Could not find suitable device")]
    DeviceNotFound,

    #[error("Tried to present without a valid swapchain")]
    PresentWithoutSwapchain,

    #[error("Gpu allocator error: {0}")]
    Allocation(#[from] gpu_allocator::AllocationError),

    #[error("Tried to write to an unmapped buffer")]
    WriteToUnmapped,
}
