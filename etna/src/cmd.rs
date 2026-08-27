use std::sync::Arc;

use ash::vk;

use crate::{device::Device, sync::TimelineSemaphore};

pub struct CommandPool {
    pub queue_family_idx: u32,
    pub queues: Vec<vk::Queue>,
    pub semaphores: Vec<TimelineSemaphore>,

    pub(crate) device: Arc<Device>,
}
