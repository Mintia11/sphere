use std::sync::Arc;

use ash::vk;
use gpu_allocator::vulkan::{Allocation, AllocationCreateDesc, AllocationScheme};

use crate::{Device, error::Error};

pub struct VideoSession {
    handle: vk::VideoSessionKHR,
    memory: Vec<Option<Allocation>>,

    device: Arc<Device>,
}

impl VideoSession {
    pub fn handle(&self) -> vk::VideoSessionKHR {
        self.handle
    }
}

impl Device {
    pub fn create_video_session(
        self: &Arc<Self>,
        caps: &vk::VideoCapabilitiesKHR,
        profile: &vk::VideoProfileInfoKHR,
        picture_format: vk::Format,
        reference_picture_format: vk::Format,
    ) -> Result<VideoSession, Error> {
        let session_info = vk::VideoSessionCreateInfoKHR::default()
            .max_active_reference_pictures(caps.max_active_reference_pictures)
            .max_coded_extent(caps.max_coded_extent)
            .max_dpb_slots(caps.max_dpb_slots)
            .picture_format(picture_format)
            .queue_family_index(self.decode_queue().family_idx)
            .reference_picture_format(reference_picture_format)
            .std_header_version(&caps.std_header_version)
            .video_profile(profile);

        let handle = unsafe {
            self.video_queue_ext()
                .create_video_session(&session_info, None)?
        };

        let len = unsafe {
            self.video_queue_ext()
                .get_video_session_memory_requirements_len(handle)?
        };

        let mut requirements = vec![vk::VideoSessionMemoryRequirementsKHR::default(); len];

        unsafe {
            self.video_queue_ext()
                .get_video_session_memory_requirements(handle, &mut requirements)?;
        }

        let mut allocator = self.allocator().lock().unwrap();
        let mut memory = Vec::new();
        for req in requirements {
            let alloc_info = AllocationCreateDesc {
                name: "VideoSessionKHR memory requirement",
                requirements: req.memory_requirements,
                location: self.location_for(req.memory_requirements.memory_type_bits),
                linear: true,
                allocation_scheme: AllocationScheme::GpuAllocatorManaged,
            };

            let allocation = allocator.allocate(&alloc_info)?;

            let bind_info = vk::BindVideoSessionMemoryInfoKHR::default()
                .memory_bind_index(req.memory_bind_index)
                .memory(unsafe { allocation.memory() })
                .memory_offset(allocation.offset())
                .memory_size(req.memory_requirements.size);

            memory.push(Some(allocation));

            unsafe {
                self.video_queue_ext()
                    .bind_video_session_memory(handle, &[bind_info])?;
            }
        }

        Ok(VideoSession {
            handle,
            memory,
            device: self.clone(),
        })
    }
}

impl Drop for VideoSession {
    fn drop(&mut self) {
        unsafe {
            self.device
                .video_queue_ext()
                .destroy_video_session(self.handle, None);
        }

        let mut allocator = self.device.allocator().lock().unwrap();
        for allocation in &mut self.memory {
            allocator.free(allocation.take().unwrap()).unwrap();
        }
    }
}
