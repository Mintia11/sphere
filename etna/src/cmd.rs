use ash::vk;
use snafu::{ResultExt, Snafu};

use crate::{
    destroy::DestroyWithDevice,
    sync::{self, TimelineSemaphore},
};

pub struct CommandPool {
    pub queue_family_idx: u32,
    pub queues: Vec<vk::Queue>,
    pub semaphores: Vec<TimelineSemaphore>,
    pub pool: vk::CommandPool,
}

impl CommandPool {
    pub fn new(
        device: &ash::Device,
        queue_family_idx: u32,
        queue_count: usize,
    ) -> Result<Self, CommandError> {
        let mut queues = Vec::with_capacity(queue_count);
        let mut semaphores = Vec::with_capacity(queue_count);

        for i in 0..queue_count {
            let queue_info = vk::DeviceQueueInfo2::default()
                .flags(vk::DeviceQueueCreateFlags::INTERNALLY_SYNCHRONIZED_KHR)
                .queue_family_index(queue_family_idx)
                .queue_index(i as u32);

            let queue = unsafe { device.get_device_queue2(&queue_info) };
            let semaphore = TimelineSemaphore::new(device, 0).context(SyncSnafu)?;

            queues.push(queue);
            semaphores.push(semaphore);
        }

        let pool_info = vk::CommandPoolCreateInfo::default()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(queue_family_idx);
        let pool = unsafe { device.create_command_pool(&pool_info, None) }.context(VulkanSnafu)?;

        Ok(Self {
            queue_family_idx,
            queues,
            semaphores,
            pool,
        })
    }
}

impl DestroyWithDevice for CommandPool {
    fn destroy(&mut self, device: &ash::Device) {
        for semaphore in &mut self.semaphores {
            semaphore.destroy(device);
        }

        unsafe {
            device.destroy_command_pool(self.pool, None);
        }
    }
}

#[derive(Debug, Snafu)]
pub enum CommandError {
    #[snafu(display("Vulkan error"))]
    Vulkan { source: vk::Result },

    #[snafu(display("Sync error"))]
    Sync { source: sync::SyncError },
}
