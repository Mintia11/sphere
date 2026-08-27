use std::sync::Arc;

use ash::vk::{self, TaggedStructure};
use snafu::{ResultExt, Snafu};

use crate::{instance::Instance, sync::TimelineSemaphore};

pub struct Device {
    pub device: ash::Device,
    pub physical_device: vk::PhysicalDevice,

    pub(crate) _instance: Arc<Instance>,
}

impl Device {
    #[profiling::function]
    pub fn create_timeline_semaphore(
        self: &Arc<Self>,
        init: u64,
    ) -> Result<TimelineSemaphore, InstanceError> {
        let mut info = vk::SemaphoreTypeCreateInfo::default()
            .initial_value(init)
            .semaphore_type(vk::SemaphoreType::TIMELINE);

        let create_info = vk::SemaphoreCreateInfo::default().push(&mut info);

        let handle =
            unsafe { self.device.create_semaphore(&create_info, None) }.context(VulkanSnafu)?;

        Ok(TimelineSemaphore {
            handle,
            value: init,

            device: self.clone(),
        })
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_device(None);
        }
    }
}

#[derive(Debug, Snafu)]
pub enum InstanceError {
    #[snafu(display("Vulkan error"))]
    Vulkan { source: vk::Result },
}
