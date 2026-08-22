use std::sync::Arc;

use ash::vk;

use crate::{Device, error::Error};

pub struct VideoSession {
    handle: vk::VideoSessionKHR,

    device: Arc<Device>,
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

        Ok(VideoSession {
            handle,
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
    }
}
