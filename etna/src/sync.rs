use std::sync::Arc;

use ash::vk;

use crate::device::Device;

pub struct TimelineSemaphore {
    pub handle: vk::Semaphore,
    pub value: u64,

    pub(crate) device: Arc<Device>,
}

impl Drop for TimelineSemaphore {
    fn drop(&mut self) {
        unsafe { self.device.device.destroy_semaphore(self.handle, None) };
    }
}
