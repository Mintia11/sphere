use ash::vk::{self, TaggedStructure};

use crate::{Device, error::Error};

pub mod decode;
pub mod session;

impl Device {
    pub fn get_formats_for_profile(
        &self,
        profile: &vk::VideoProfileInfoKHR,
        usage: vk::ImageUsageFlags,
    ) -> Result<Vec<vk::VideoFormatPropertiesKHR<'_>>, Error> {
        let mut video_profile_list =
            vk::VideoProfileListInfoKHR::default().profiles(std::slice::from_ref(profile));
        let video_format_info = vk::PhysicalDeviceVideoFormatInfoKHR::default()
            .image_usage(usage)
            .push(&mut video_profile_list);
        let formats = unsafe {
            self.video_queue_instance_ext()
                .get_physical_device_video_format_properties_len(
                    self.physical_device(),
                    &video_format_info,
                )?
        };

        let mut formats = vec![vk::VideoFormatPropertiesKHR::default(); formats];

        unsafe {
            self.video_queue_instance_ext()
                .get_physical_device_video_format_properties(
                    self.physical_device(),
                    &video_format_info,
                    &mut formats,
                )?;
        }

        Ok(formats)
    }
}
