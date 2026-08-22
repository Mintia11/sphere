use std::{
    ffi::CStr,
    sync::{Arc, Mutex},
};

use ash::{
    ext, khr,
    vk::{self, TaggedStructure},
};
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

    video_queue_ext: khr::video_queue::Device,
    video_queue_instance_ext: khr::video_queue::Instance,

    video_decode_queue_ext: khr::video_decode_queue::Device,

    graphics_queue: Queue,
    decode_queue: Queue,
}

pub struct Queue {
    handle: vk::Queue,
    family_idx: u32,
    command_pool: vk::CommandPool,
}

impl Device {
    pub fn new(instance: &Instance, device_exts: &[&'static CStr]) -> Result<Arc<Self>, Error> {
        let mut device_exts = device_exts.to_vec();
        device_exts.extend([
            khr::swapchain::NAME,
            ext::pageable_device_local_memory::NAME,
            ext::memory_priority::NAME,
            khr::video_queue::NAME,
            khr::video_decode_queue::NAME,
            khr::video_decode_h264::NAME,
        ]);

        let (physical_device, queue_families) = pick_physical_device(instance, &device_exts)?;
        let device = create_device(instance, physical_device, &device_exts, queue_families)?;

        let graphics_queue = Queue {
            handle: unsafe { device.get_device_queue(queue_families.0, 0) },
            family_idx: queue_families.0,
            command_pool: unsafe {
                device.create_command_pool(
                    &vk::CommandPoolCreateInfo::default()
                        .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
                        .queue_family_index(queue_families.0),
                    None,
                )?
            },
        };
        let decode_queue = Queue {
            handle: unsafe { device.get_device_queue(queue_families.1, 0) },
            family_idx: queue_families.1,
            command_pool: unsafe {
                device.create_command_pool(
                    &vk::CommandPoolCreateInfo::default()
                        .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
                        .queue_family_index(queue_families.1),
                    None,
                )?
            },
        };

        let allocator_info = AllocatorCreateDesc {
            instance: instance.handle().clone(),
            device: device.clone(),
            physical_device,
            debug_settings: AllocatorDebugSettings::default(),
            buffer_device_address: true,
            allocation_sizes: AllocationSizes::default(),
        };
        let allocator = Allocator::new(&allocator_info)?;

        Ok(Arc::new(Device {
            physical_device,
            device: device.clone(),
            allocator: Mutex::new(allocator),

            video_queue_ext: khr::video_queue::Device::load(instance.handle(), &device),
            video_queue_instance_ext: khr::video_queue::Instance::load(
                instance.entry(),
                instance.handle(),
            ),
            video_decode_queue_ext: khr::video_decode_queue::Device::load(
                instance.handle(),
                &device,
            ),

            graphics_queue,
            decode_queue,
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

    pub fn decode_queue(&self) -> &Queue {
        &self.decode_queue
    }

    pub fn video_queue_ext(&self) -> &khr::video_queue::Device {
        &self.video_queue_ext
    }

    pub fn video_queue_instance_ext(&self) -> &khr::video_queue::Instance {
        &self.video_queue_instance_ext
    }

    pub fn video_decode_queue_ext(&self) -> &khr::video_decode_queue::Device {
        &self.video_decode_queue_ext
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
) -> Result<(vk::PhysicalDevice, (u32, u32)), Error> {
    for physical_device in unsafe { instance.handle().enumerate_physical_devices()? } {
        let extensions = unsafe {
            instance
                .handle()
                .enumerate_device_extension_properties(physical_device)?
        };

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
        let mut decode_queue: Option<u32> = None;

        for (i, queue_family) in queue_families.iter().enumerate() {
            let queue_family = queue_family.queue_family_properties;

            if queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                graphics_queue = Some(i as u32);
            }

            if queue_family
                .queue_flags
                .contains(vk::QueueFlags::VIDEO_DECODE_KHR)
            {
                decode_queue = Some(i as u32);
            }
        }

        if let Some(graphics_queue) = graphics_queue
            && let Some(decode_queue) = decode_queue
        {
            return Ok((physical_device, (graphics_queue, decode_queue)));
        }
    }

    Err(Error::DeviceNotFound)
}

fn create_device(
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
    device_exts: &[&'static CStr],
    queue_families: (u32, u32),
) -> Result<ash::Device, Error> {
    let mut buffer_device_address =
        vk::PhysicalDeviceBufferDeviceAddressFeatures::default().buffer_device_address(true);

    let mut synchronization2 =
        vk::PhysicalDeviceSynchronization2Features::default().synchronization2(true);

    let mut dynamic_rendering =
        vk::PhysicalDeviceDynamicRenderingFeatures::default().dynamic_rendering(true);

    let graphics_queue_info = vk::DeviceQueueCreateInfo::default()
        .queue_family_index(queue_families.0)
        .queue_priorities(&[1.0]);
    let decode_queue_info = vk::DeviceQueueCreateInfo::default()
        .queue_family_index(queue_families.1)
        .queue_priorities(&[1.0]);

    let queue_infos = [graphics_queue_info, decode_queue_info];
    let exts = device_exts.iter().map(|e| e.as_ptr()).collect::<Vec<_>>();
    let device_info = vk::DeviceCreateInfo::default()
        .queue_create_infos(&queue_infos)
        .enabled_extension_names(&exts)
        .push(&mut dynamic_rendering)
        .push(&mut synchronization2)
        .push(&mut buffer_device_address);

    let device = unsafe {
        instance
            .handle()
            .create_device(physical_device, &device_info, None)?
    };

    Ok(device)
}
