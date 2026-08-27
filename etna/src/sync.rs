use std::sync::Arc;

use ash::vk::{self, TaggedStructure};
use snafu::{ResultExt, Snafu};

use crate::device::Device;

pub struct TimelineSemaphore {
    pub handle: vk::Semaphore,
    pub value: u64,

    pub(crate) device: Arc<Device>,
}

impl TimelineSemaphore {
    pub fn new(device: &Arc<Device>, init: u64) -> Result<Self, SyncError> {
        let mut info = vk::SemaphoreTypeCreateInfo::default()
            .initial_value(init)
            .semaphore_type(vk::SemaphoreType::TIMELINE);

        let create_info = vk::SemaphoreCreateInfo::default().push(&mut info);

        let handle =
            unsafe { device.device.create_semaphore(&create_info, None) }.context(VulkanSnafu)?;

        Ok(TimelineSemaphore {
            handle,
            value: init,

            device: device.clone(),
        })
    }
}

impl Drop for TimelineSemaphore {
    fn drop(&mut self) {
        unsafe { self.device.device.destroy_semaphore(self.handle, None) };
    }
}

#[derive(Debug, Snafu)]
pub enum SyncError {
    #[snafu(display("Vulkan error"))]
    Vulkan { source: vk::Result },
}
