use std::ffi::CStr;

use ash::{
    ext, khr,
    vk::{self, TaggedStructure},
};
use snafu::{ResultExt, Snafu};

use crate::{
    cmd::{self, CommandPool},
    destroy::{DestroyWithDevice, DestroyWithInstance},
    instance::Instance,
    surface::Surface,
};

pub struct Device {
    pub device: ash::Device,
    pub physical_device: vk::PhysicalDevice,

    pub command_pools: Vec<CommandPool>,
}

pub(crate) const DEVICE_EXTENSIONS: &[&CStr] = &[
    khr::swapchain::NAME,
    khr::swapchain_maintenance1::NAME,
    khr::swapchain_mutable_format::NAME,
    khr::internally_synchronized_queues::NAME,
    ext::swapchain_maintenance1::NAME,
    ext::shader_object::NAME,
    ext::descriptor_heap::NAME,
    khr::video_queue::NAME,
    khr::video_decode_queue::NAME,
    khr::video_decode_av1::NAME,
    khr::video_decode_h264::NAME,
    khr::video_decode_h265::NAME,
    khr::video_maintenance1::NAME,
    khr::video_maintenance2::NAME,
];

impl Device {
    pub fn new(
        instance: &Instance,
        physical_device: vk::PhysicalDevice,
        surface: Option<&Surface>,
    ) -> Result<Self, DeviceError> {
        let mut internally_synchronized_queues =
            vk::PhysicalDeviceInternallySynchronizedQueuesFeaturesKHR::default()
                .internally_synchronized_queues(true);
        let mut swapchain_maintenance1 =
            vk::PhysicalDeviceSwapchainMaintenance1FeaturesKHR::default()
                .swapchain_maintenance1(true);
        let mut video_maintenance1 =
            vk::PhysicalDeviceVideoMaintenance1FeaturesKHR::default().video_maintenance1(true);
        let mut video_maintenance2 =
            vk::PhysicalDeviceVideoMaintenance2FeaturesKHR::default().video_maintenance2(true);
        let mut vulkan_11 =
            vk::PhysicalDeviceVulkan11Features::default().sampler_ycbcr_conversion(true);
        let mut vulkan_12 = vk::PhysicalDeviceVulkan12Features::default().timeline_semaphore(true);
        let mut vulkan_13 = vk::PhysicalDeviceVulkan13Features::default()
            .dynamic_rendering(true)
            .synchronization2(true);
        let mut shader_object =
            vk::PhysicalDeviceShaderObjectFeaturesEXT::default().shader_object(true);
        let mut descriptor_heap =
            vk::PhysicalDeviceDescriptorHeapFeaturesEXT::default().descriptor_heap(true);
        let mut features = vk::PhysicalDeviceFeatures2::default();

        let queue_family_count = unsafe {
            instance
                .instance
                .get_physical_device_queue_family_properties2_len(physical_device)
        };
        let mut video_infos =
            vec![vk::QueueFamilyVideoPropertiesKHR::default(); queue_family_count];
        let mut queue_families: Vec<_> = video_infos
            .iter_mut()
            .map(|video_info| vk::QueueFamilyProperties2::default().push(video_info))
            .collect();

        unsafe {
            instance
                .instance
                .get_physical_device_queue_family_properties2(physical_device, &mut queue_families);
        }

        let queue_families: Vec<_> = queue_families
            .iter()
            .map(|f| f.queue_family_properties)
            .collect();

        let mut graphics_queue = None;
        let mut decode_queue = None;

        log::debug!("Queue families supported by device:");
        for (i, (queue_family, video_info)) in
            queue_families.iter().zip(video_infos.iter()).enumerate()
        {
            log::debug!(
                "    {}: {:?} num {}",
                i,
                queue_family.queue_flags,
                queue_family.queue_count
            );
            if queue_family
                .queue_flags
                .contains(vk::QueueFlags::VIDEO_DECODE_KHR)
            {
                log::debug!(
                    "        supported ops: {:?}",
                    video_info.video_codec_operations
                );
            }

            if queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                && graphics_queue.is_none()
            {
                if let Some(surface) = surface {
                    if !unsafe {
                        surface.ext.get_physical_device_surface_support(
                            physical_device,
                            i as u32,
                            surface.handle,
                        )
                    }
                    .context(VulkanSnafu)?
                    {
                        log::error!(
                            "Graphics queue doesn't support presenting to the specified surface!"
                        );
                        snafu::whatever!(
                            "Graphics queue doesn't support presenting to the specified surface!"
                        )
                    }
                }

                graphics_queue = Some(i as u32);
            }

            if video_info.video_codec_operations.contains(
                vk::VideoCodecOperationFlagsKHR::DECODE_H264
                    | vk::VideoCodecOperationFlagsKHR::DECODE_H265
                    | vk::VideoCodecOperationFlagsKHR::DECODE_AV1,
            ) && decode_queue.is_none()
            {
                decode_queue = Some(i as u32);
            }
        }

        let supported_exts = unsafe {
            instance
                .instance
                .enumerate_device_extension_properties(physical_device)
        }
        .context(VulkanSnafu)?;

        log::debug!("Available device extensions:");
        for ext in supported_exts {
            let name = ext
                .extension_name_as_c_str()
                .context(FromBytesUntilNullSnafu)?
                .to_string_lossy();
            log::debug!("    {name}");
        }

        let mut queue_infos = Vec::new();
        let graphics_queue = graphics_queue.ok_or_else(|| DeviceError::Whatever {
            message: "did not find a graphics capable queue".to_string(),
        })?;
        let decode_queue = decode_queue.ok_or_else(|| DeviceError::Whatever {
            message: "did not find a decode capable queue".to_string(),
        })?;

        let graphics_queue_priorities = [1.0, 1.0]; // one graphics one transfer
        queue_infos.push(
            vk::DeviceQueueCreateInfo::default()
                .flags(vk::DeviceQueueCreateFlags::INTERNALLY_SYNCHRONIZED_KHR)
                .queue_family_index(graphics_queue)
                .queue_priorities(&graphics_queue_priorities),
        );
        let decode_queue_priorities = [1.0];
        queue_infos.push(
            vk::DeviceQueueCreateInfo::default()
                .flags(vk::DeviceQueueCreateFlags::INTERNALLY_SYNCHRONIZED_KHR)
                .queue_family_index(decode_queue)
                .queue_priorities(&decode_queue_priorities),
        );

        let enabled_extensions = DEVICE_EXTENSIONS.to_vec();
        log::debug!("Creating vulkan device");
        log::debug!("    with extensions:");
        for ext in &enabled_extensions {
            log::debug!("        {}", ext.to_string_lossy());
        }

        let enabled_extensions: Vec<_> = enabled_extensions.iter().map(|e| e.as_ptr()).collect();

        let create_info = vk::DeviceCreateInfo::default()
            .push(&mut features)
            .push(&mut descriptor_heap)
            .push(&mut shader_object)
            .push(&mut vulkan_13)
            .push(&mut vulkan_12)
            .push(&mut vulkan_11)
            .push(&mut video_maintenance1)
            .push(&mut video_maintenance2)
            .push(&mut swapchain_maintenance1)
            .push(&mut internally_synchronized_queues)
            .enabled_extension_names(&enabled_extensions)
            .queue_create_infos(&queue_infos);

        let device = unsafe {
            instance
                .instance
                .create_device(physical_device, &create_info, None)
        }
        .context(VulkanSnafu)?;

        let graphics_command_pool =
            CommandPool::new(&device, graphics_queue, 2).context(CommandSnafu)?;
        let decode_command_pool =
            CommandPool::new(&device, decode_queue, 1).context(CommandSnafu)?;

        Ok(Device {
            device,
            physical_device,
            command_pools: vec![graphics_command_pool, decode_command_pool],
        })
    }
}

impl DestroyWithInstance for Device {
    fn destroy(&mut self, _instance: &ash::Instance) {
        for command_pool in &mut self.command_pools {
            command_pool.destroy(&self.device);
        }

        unsafe { self.device.destroy_device(None) };
    }
}

#[derive(Debug, Snafu)]
pub enum DeviceError {
    #[snafu(display("Vulkan error"))]
    Vulkan { source: vk::Result },

    #[snafu(display("Error while reading UTF-8 string"))]
    FromBytesUntilNull {
        source: std::ffi::FromBytesUntilNulError,
    },

    #[snafu(display("Command error"))]
    Command { source: cmd::CommandError },

    #[snafu(whatever)]
    Whatever { message: String },
}
