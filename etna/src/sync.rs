use ash::vk::{self, TaggedStructure};
use snafu::{ResultExt, Snafu};

use crate::{destroy::DestroyWithDevice, device::Device};

pub struct TimelineSemaphore {
    pub handle: vk::Semaphore,
    pub value: u64,
}

impl TimelineSemaphore {
    pub fn new(device: &ash::Device, init: u64) -> Result<Self, SyncError> {
        let mut info = vk::SemaphoreTypeCreateInfo::default()
            .initial_value(init)
            .semaphore_type(vk::SemaphoreType::TIMELINE);

        let create_info = vk::SemaphoreCreateInfo::default().push(&mut info);

        let handle = unsafe { device.create_semaphore(&create_info, None) }.context(VulkanSnafu)?;

        Ok(TimelineSemaphore {
            handle,
            value: init,
        })
    }
}

impl DestroyWithDevice for TimelineSemaphore {
    fn destroy(&mut self, device: &Device) {
        unsafe {
            device.device.destroy_semaphore(self.handle, None);
        }
    }
}

pub struct BinarySemaphore {
    pub handle: vk::Semaphore,
}

impl BinarySemaphore {
    pub fn new(device: &ash::Device) -> Result<Self, SyncError> {
        let mut info =
            vk::SemaphoreTypeCreateInfo::default().semaphore_type(vk::SemaphoreType::BINARY);

        let create_info = vk::SemaphoreCreateInfo::default().push(&mut info);

        let handle = unsafe { device.create_semaphore(&create_info, None) }.context(VulkanSnafu)?;

        Ok(BinarySemaphore { handle })
    }
}

impl DestroyWithDevice for BinarySemaphore {
    fn destroy(&mut self, device: &Device) {
        unsafe {
            device.device.destroy_semaphore(self.handle, None);
        }
    }
}

#[derive(Debug, Snafu)]
pub enum SyncError {
    #[snafu(display("Vulkan error"))]
    Vulkan { source: vk::Result },
}
