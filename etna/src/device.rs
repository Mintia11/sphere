use std::{
    ffi::CStr,
    sync::{Arc, Mutex},
};

use ash::vk;
use gpu_allocator::{
    AllocationSizes, AllocatorDebugSettings,
    vulkan::{Allocator, AllocatorCreateDesc},
};

use crate::{
    command_buffer::CommandBuffer,
    error::Error,
    instance::Instance,
    sync::{Fence, Semaphore},
};

pub struct Device {
    physical_device: vk::PhysicalDevice,
    device: ash::Device,
    allocator: Mutex<Allocator>,

    graphics_queue: Queue,
}

pub struct Queue {
    handle: vk::Queue,
    family_idx: u32,
    command_pool: vk::CommandPool,
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
            command_pool: unsafe {
                device.create_command_pool(
                    &vk::CommandPoolCreateInfo::default()
                        .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
                        .queue_family_index(graphics_queue_family),
                    None,
                )?
            },
        };

        let allocator_info = AllocatorCreateDesc {
            instance: instance.handle().clone(),
            device: device.clone(),
            physical_device,
            debug_settings: AllocatorDebugSettings::default(),
            buffer_device_address: false, // TODO: Use this it's so good
            allocation_sizes: AllocationSizes::default(),
        };
        let allocator = Allocator::new(&allocator_info)?;

        Ok(Arc::new(Device {
            physical_device,
            device,
            graphics_queue,
            allocator: Mutex::new(allocator),
        }))
    }

    pub fn handle(&self) -> &ash::Device {
        &self.device
    }

    pub fn physical_device(&self) -> vk::PhysicalDevice {
        self.physical_device
    }

    pub fn allocator(&self) -> &Mutex<Allocator> {
        &self.allocator
    }

    pub fn graphics_queue(&self) -> &Queue {
        &self.graphics_queue
    }

    pub fn transfer_queue(&self) -> &Queue {
        &self.graphics_queue // todo: maybe create a separate transfer queue to not clog up the graphics one
    }

    pub fn submit_queue(
        &self,
        queue: &Queue,
        command_buffer: &CommandBuffer,
        wait: Option<&Semaphore>,
        signal: Option<&Semaphore>,
        fence: Option<&Fence>,
    ) -> Result<(), Error> {
        let command_buffer_info = vk::CommandBufferSubmitInfo::default()
            .command_buffer(command_buffer.handle())
            .device_mask(1);

        let wait = wait.map(|wait| {
            vk::SemaphoreSubmitInfo::default()
                .semaphore(wait.handle())
                .stage_mask(vk::PipelineStageFlags2::ALL_COMMANDS)
        });
        let signal = signal.map(|signal| {
            vk::SemaphoreSubmitInfo::default()
                .semaphore(signal.handle())
                .stage_mask(vk::PipelineStageFlags2::ALL_COMMANDS)
        });

        let submit_info = vk::SubmitInfo2::default()
            .command_buffer_infos(std::slice::from_ref(&command_buffer_info));

        let submit_info = if let Some(signal) = &signal {
            submit_info.signal_semaphore_infos(std::slice::from_ref(signal))
        } else {
            submit_info
        };

        let submit_info = if let Some(wait) = &wait {
            submit_info.wait_semaphore_infos(std::slice::from_ref(wait))
        } else {
            submit_info
        };

        unsafe {
            self.handle()
                .queue_submit2(
                    queue.handle(),
                    &[submit_info],
                    fence.map(Fence::handle).unwrap_or(vk::Fence::null()),
                )
                .map_err(Into::into)
        }
    }
}

impl Queue {
    pub fn handle(&self) -> vk::Queue {
        self.handle
    }

    pub fn command_pool(&self) -> vk::CommandPool {
        self.command_pool
    }
}

fn pick_physical_device(
    instance: &Instance,
    device_exts: &[&'static CStr],
) -> Result<(vk::PhysicalDevice, u32), Error> {
    for physical_device in unsafe { instance.handle().enumerate_physical_devices()? } {
        let mut props = vk::PhysicalDeviceProperties2::default();

        unsafe {
            instance
                .handle()
                .get_physical_device_properties2(physical_device, &mut props)
        }

        let props = props.properties;
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

        // for ext in &extensions {
        //     println!(
        //         "extension {:?}: version {}",
        //         ext.extension_name_as_c_str().unwrap(),
        //         ext.spec_version
        //     );
        // }

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

        let queue_family_count = unsafe {
            instance
                .handle()
                .get_physical_device_queue_family_properties2_len(physical_device)
        };

        let mut queue_families = vec![vk::QueueFamilyProperties2::default(); queue_family_count];
        unsafe {
            instance
                .handle()
                .get_physical_device_queue_family_properties2(physical_device, &mut queue_families);
        }

        let mut graphics_queue: Option<u32> = None;

        for (i, queue_family) in queue_families.iter().enumerate() {
            let queue_family = queue_family.queue_family_properties;

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
