use std::{ffi::CStr, sync::Arc};

use ash::vk;

use crate::{error::Error, instance::Instance};

pub struct Device {
    physical_device: vk::PhysicalDevice,
    device: ash::Device,

    graphics_queue: Queue,
}

pub struct Queue {
    handle: vk::Queue,
    family_idx: u32,
}

impl Device {
    pub fn new(instance: &Instance, device_exts: &[&'static CStr]) -> Result<Arc<Self>, Error> {
        let (physical_device, graphics_queue_family) = pick_physical_device(instance, device_exts)?;
        let device = create_device(
            instance,
            physical_device,
            device_exts,
            graphics_queue_family,
        )?;

        let graphics_queue = Queue {
            handle: unsafe { device.get_device_queue(graphics_queue_family, 0) },
            family_idx: graphics_queue_family,
        };

        Ok(Arc::new(Device {
            physical_device,
            device,
            graphics_queue,
        }))
    }

    pub fn handle(&self) -> &ash::Device {
        &self.device
    }
}

fn pick_physical_device(
    instance: &Instance,
    device_exts: &[&'static CStr],
) -> Result<(vk::PhysicalDevice, u32), Error> {
    for physical_device in unsafe { instance.handle().enumerate_physical_devices()? } {
        let props = unsafe {
            instance
                .handle()
                .get_physical_device_properties(physical_device)
        };

        println!(
            "physical device {:?} ({:?}):",
            props.device_name_as_c_str().unwrap(),
            props.device_type
        );

        let extensions = unsafe {
            instance
                .handle()
                .enumerate_device_extension_properties(physical_device)?
        };

        for ext in &extensions {
            println!(
                "extension {:?}: version {}",
                ext.extension_name_as_c_str().unwrap(),
                ext.spec_version
            );
        }

        let mut has_all_exts = true;
        for needed in device_exts {
            let mut has_this_ext = false;
            for has in &extensions {
                if has.extension_name_as_c_str().unwrap() == needed {
                    has_this_ext = true;
                }
            }

            has_all_exts = has_this_ext;
        }

        if !has_all_exts {
            continue;
        }

        let queue_families = unsafe {
            instance
                .handle()
                .get_physical_device_queue_family_properties(physical_device)
        };

        let mut graphics_queue: Option<u32> = None;

        for (i, queue_family) in queue_families.iter().enumerate() {
            println!(
                "queue family {i}: {:?} ({} queues)",
                queue_family.queue_flags, queue_family.queue_count
            );

            if queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                graphics_queue = Some(i as u32);
            }
        }

        if let Some(graphics_queue) = graphics_queue {
            return Ok((physical_device, graphics_queue));
        }
    }

    Err(Error::DeviceNotFound)
}

fn create_device(
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
    device_exts: &[&'static CStr],
    graphics_queue_family: u32,
) -> Result<ash::Device, Error> {
    let mut buffer_device_address =
        vk::PhysicalDeviceBufferDeviceAddressFeatures::default().buffer_device_address(true);

    let mut synchronization2 =
        vk::PhysicalDeviceSynchronization2Features::default().synchronization2(true);

    let mut dynamic_rendering =
        vk::PhysicalDeviceDynamicRenderingFeatures::default().dynamic_rendering(true);

    let queue_info = vk::DeviceQueueCreateInfo::default()
        .queue_family_index(graphics_queue_family)
        .queue_priorities(&[1.0]);

    let exts = device_exts.iter().map(|e| e.as_ptr()).collect::<Vec<_>>();
    let device_info = vk::DeviceCreateInfo::default()
        .queue_create_infos(std::slice::from_ref(&queue_info))
        .enabled_extension_names(&exts)
        .push_next(&mut dynamic_rendering)
        .push_next(&mut synchronization2)
        .push_next(&mut buffer_device_address);

    let device = unsafe {
        instance
            .handle()
            .create_device(physical_device, &device_info, None)?
    };

    Ok(device)
}
