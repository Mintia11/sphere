use std::sync::Arc;

use ash::vk;

use crate::{Device, error::Error};

pub struct Semaphore {
    handle: vk::Semaphore,
    device: Arc<Device>,
}

impl Device {
    pub fn create_semaphore(self: &Arc<Self>) -> Result<Semaphore, Error> {
        let handle = unsafe {
            self.handle()
                .create_semaphore(&vk::SemaphoreCreateInfo::default(), None)?
        };

        Ok(Semaphore {
            handle,
            device: self.clone(),
        })
    }
}

impl Semaphore {
    pub fn handle(&self) -> vk::Semaphore {
        self.handle
    }
}

impl Drop for Semaphore {
    fn drop(&mut self) {
        unsafe { self.device.handle().destroy_semaphore(self.handle, None) };
    }
}

pub struct Fence {
    handle: vk::Fence,
    device: Arc<Device>,
}

impl Device {
    pub fn create_fence(self: &Arc<Self>, signaled: bool) -> Result<Fence, Error> {
        let handle = unsafe {
            self.handle().create_fence(
                &vk::FenceCreateInfo::default().flags(if signaled {
                    vk::FenceCreateFlags::SIGNALED
                } else {
                    vk::FenceCreateFlags::empty()
                }),
                None,
            )?
        };

        Ok(Fence {
            handle,
            device: self.clone(),
        })
    }
}

impl Fence {
    pub fn handle(&self) -> vk::Fence {
        self.handle
    }

    pub fn wait(&self, timeout: u64) -> Result<(), Error> {
        unsafe {
            self.device
                .handle()
                .wait_for_fences(&[self.handle], true, timeout)
                .map_err(Into::into)
        }
    }
}
