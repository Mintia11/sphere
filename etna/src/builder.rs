use std::ffi::CStr;
use std::sync::{Arc, Mutex};

use ash::khr;
use ash::{Entry, ext, vk};
use gpu_allocator::vulkan::{Allocator, AllocatorCreateDesc};
use raw_window_handle::RawWindowHandle;

use crate::{GPUContext, codec::VideoCodecExtension, error::Error};

pub struct GPUContextBuilder {
    codecs: Vec<Box<dyn VideoCodecExtension>>,
    window_handle: Option<RawWindowHandle>,
}

impl Default for GPUContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl GPUContextBuilder {
    pub fn new() -> Self {
        GPUContextBuilder {
            codecs: Vec::new(),
            window_handle: None,
        }
    }

    pub fn with_codec(mut self, codec: impl VideoCodecExtension + 'static) -> Self {
        self.codecs.push(Box::new(codec));
        self
    }

    pub fn with_window_handle(mut self, window_handle: RawWindowHandle) -> Self {
        self.window_handle = Some(window_handle);
        self
    }

    pub fn build(self) -> Result<GPUContext, Error> {
        let entry = unsafe { Entry::load()? };
        let instance = create_instance(&entry)?;
        let (surface, surface_ext) = create_surface(
            &entry,
            &instance,
            &self
                .window_handle
                .expect("Cannot create surface without window handle"),
        )?;

        let mut device_exts = vec![
            khr::swapchain::NAME,
            khr::synchronization2::NAME,
            // khr::video_queue::NAME,
            // khr::video_decode_queue::NAME,
        ];
        for codec in &self.codecs {
            device_exts.extend(codec.device_extensions());
        }

        let ops: Vec<_> = self.codecs.iter().map(|c| c.codec_operation()).collect();

        let (physical_device, queue_families) =
            pick_physical_device(&instance, surface, surface_ext.clone(), &device_exts, &ops)?;
        let (device, queues) =
            create_device(&instance, physical_device, &device_exts, queue_families)?;

        let (swapchain, swapchain_ext, swapchain_format, swapchain_images, swapchain_image_views) =
            create_swapchain(
                &instance,
                physical_device,
                &device,
                surface,
                surface_ext.clone(),
                SwapchainInfo {
                    old_swapchain: None,
                    preferred_format: vk::Format::R8G8B8A8_UNORM,
                    preferred_colorspace: vk::ColorSpaceKHR::SRGB_NONLINEAR,
                    preferred_present_mode: vk::PresentModeKHR::MAILBOX,
                },
            )?;

        todo!()
    }
}

fn create_instance(entry: &Entry) -> Result<ash::Instance, Error> {
    let instance_extensions = unsafe { entry.enumerate_instance_extension_properties(None)? };

    let mut used_extensions = vec![];
    for extension in &instance_extensions {
        let name = extension.extension_name_as_c_str().unwrap();

        println!("extension {:?}: version {}", name, extension.spec_version);

        if name == ext::swapchain_colorspace::NAME {
            used_extensions.push(ext::swapchain_colorspace::NAME.as_ptr());
        }

        #[cfg(debug_assertions)]
        if name == ext::debug_utils::NAME {
            used_extensions.push(ext::debug_utils::NAME.as_ptr());
        }

        #[cfg(debug_assertions)]
        if name == ext::debug_report::NAME {
            used_extensions.push(ext::debug_report::NAME.as_ptr());
        }
    }

    used_extensions.push(khr::surface::NAME.as_ptr());

    #[cfg(windows)]
    used_extensions.push(khr::win32_surface::NAME.as_ptr());

    let instance_layers = unsafe { entry.enumerate_instance_layer_properties()? };

    let mut used_layers = vec![];
    for layer in &instance_layers {
        let name = layer.layer_name_as_c_str().unwrap();
        let description = layer.description_as_c_str().unwrap();

        println!(
            "layer {:?}: {:?} - version {}",
            name, description, layer.spec_version
        );

        #[cfg(debug_assertions)]
        if name == c"VK_LAYER_KHRONOS_validation" {
            used_layers.push(c"VK_LAYER_KHRONOS_validation".as_ptr());
        }
    }

    let application_info = vk::ApplicationInfo::default().api_version(vk::API_VERSION_1_3);

    let instance_info = vk::InstanceCreateInfo::default()
        .application_info(&application_info)
        .enabled_extension_names(&used_extensions)
        .enabled_layer_names(&used_layers);

    Ok(unsafe { entry.create_instance(&instance_info, None)? })
}

#[cfg(windows)]
fn create_surface(
    entry: &Entry,
    instance: &ash::Instance,
    handle: &RawWindowHandle,
) -> Result<(vk::SurfaceKHR, khr::surface::Instance), Error> {
    let surface_extension = khr::win32_surface::Instance::new(entry, instance);

    let (hwnd, hinstance) = match handle {
        RawWindowHandle::Win32(win32_handle) => (win32_handle.hwnd, win32_handle.hinstance),
        _ => unreachable!(),
    };

    let surface_info = vk::Win32SurfaceCreateInfoKHR::default()
        .hwnd(hwnd.get())
        .hinstance(hinstance.unwrap().get());
    let surface = unsafe { surface_extension.create_win32_surface(&surface_info, None)? };

    Ok((surface, khr::surface::Instance::new(entry, instance)))
}

fn pick_physical_device(
    instance: &ash::Instance,
    surface: vk::SurfaceKHR,
    surface_ext: khr::surface::Instance,
    device_exts: &[&'static CStr],
    ops: &[vk::VideoCodecOperationFlagsKHR],
) -> Result<(vk::PhysicalDevice, (u32, u32, u32)), Error> {
    for physical_device in unsafe { instance.enumerate_physical_devices()? } {
        let mut graphics_queue: Option<u32> = None;
        let mut present_queue: Option<u32> = None;
        let mut decode_queue: Option<u32> = None;

        let props = unsafe { instance.get_physical_device_properties(physical_device) };

        println!(
            "physical device {:?} ({:?}):",
            props.device_name_as_c_str().unwrap(),
            props.device_type
        );

        let extensions =
            unsafe { instance.enumerate_device_extension_properties(physical_device)? };

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

        let num_queue_families =
            unsafe { instance.get_physical_device_queue_family_properties2_len(physical_device) };

        let mut video_props =
            vec![vk::QueueFamilyVideoPropertiesKHR::default(); num_queue_families];
        let mut queue_families = vec![vk::QueueFamilyProperties2::default(); num_queue_families];

        for (queue_family, video_prop) in queue_families.iter_mut().zip(video_props.iter_mut()) {
            let taken = std::mem::take(queue_family);
            *queue_family = taken.push_next(video_prop);
        }

        unsafe {
            instance
                .get_physical_device_queue_family_properties2(physical_device, &mut queue_families);
        }

        let queue_families = queue_families
            .iter()
            .map(|p| p.queue_family_properties)
            .collect::<Vec<_>>();
        let video_props = video_props.to_vec();
        for (i, (queue_family, video_props)) in
            queue_families.iter().zip(video_props.iter()).enumerate()
        {
            println!(
                "queue family {i}: {:?} ({} queues)",
                queue_family.queue_flags, queue_family.queue_count
            );

            if queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                graphics_queue = Some(i as u32);
            }

            if unsafe {
                surface_ext.get_physical_device_surface_support(
                    physical_device,
                    i as u32,
                    surface,
                )?
            } {
                present_queue = Some(i as u32);
            }

            if !video_props.video_codec_operations.is_empty()
                && ops
                    .iter()
                    .all(|&op| video_props.video_codec_operations.contains(op))
            {
                decode_queue = Some(i as u32);
            }
        }

        if let Some(graphics_queue) = graphics_queue
            && let Some(present_queue) = present_queue
            && let Some(decode_queue) = decode_queue
        {
            return Ok((
                physical_device,
                (graphics_queue, present_queue, decode_queue),
            ));
        }
    }

    Err(Error::DeviceNotFound)
}

fn create_device(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    device_exts: &[&'static CStr],
    queue_families: (u32, u32, u32),
) -> Result<(ash::Device, (vk::Queue, vk::Queue, vk::Queue)), Error> {
    let (graphics, present, decode) = queue_families;

    let mut queue_infos = vec![];
    queue_infos.push(
        vk::DeviceQueueCreateInfo::default()
            .queue_family_index(graphics)
            .queue_priorities(if graphics == present {
                &[1.0, 1.0]
            } else {
                &[1.0]
            }),
    );
    if present != graphics {
        queue_infos.push(
            vk::DeviceQueueCreateInfo::default()
                .queue_family_index(present)
                .queue_priorities(&[1.0]),
        );
    }
    queue_infos.push(
        vk::DeviceQueueCreateInfo::default()
            .queue_family_index(decode)
            .queue_priorities(&[1.0]),
    );

    let mut buffer_device_address =
        vk::PhysicalDeviceBufferDeviceAddressFeatures::default().buffer_device_address(true);

    let mut synchronization2 =
        vk::PhysicalDeviceSynchronization2Features::default().synchronization2(true);

    let mut dynamic_rendering =
        vk::PhysicalDeviceDynamicRenderingFeatures::default().dynamic_rendering(true);

    let exts = device_exts.iter().map(|e| e.as_ptr()).collect::<Vec<_>>();
    let device_info = vk::DeviceCreateInfo::default()
        .queue_create_infos(&queue_infos)
        .enabled_extension_names(&exts)
        .push_next(&mut dynamic_rendering)
        .push_next(&mut synchronization2)
        .push_next(&mut buffer_device_address);

    let device = unsafe { instance.create_device(physical_device, &device_info, None)? };
    let graphics_queue = unsafe { device.get_device_queue(graphics, 0) };
    let present_queue =
        unsafe { device.get_device_queue(present, if present == graphics { 1 } else { 0 }) };
    let decode_queue = unsafe { device.get_device_queue(decode, 0) };

    Ok((device, (graphics_queue, present_queue, decode_queue)))
}

pub struct SwapchainInfo {
    old_swapchain: Option<vk::SwapchainKHR>,
    preferred_format: vk::Format,
    preferred_colorspace: vk::ColorSpaceKHR,
    preferred_present_mode: vk::PresentModeKHR,
}

fn create_swapchain(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    device: &ash::Device,
    surface: vk::SurfaceKHR,
    surface_ext: khr::surface::Instance,
    info: SwapchainInfo,
) -> Result<
    (
        vk::SwapchainKHR,
        khr::swapchain::Device,
        vk::Format,
        Vec<vk::Image>,
        Vec<vk::ImageView>,
    ),
    Error,
> {
    let formats =
        unsafe { surface_ext.get_physical_device_surface_formats(physical_device, surface)? };
    for format in &formats {
        println!("format: {:?} - {:?}", format.format, format.color_space);
    }

    let chosen_format = formats
        .iter()
        .cloned()
        .find(|f| f.format == info.preferred_format && f.color_space == info.preferred_colorspace)
        .unwrap_or(vk::SurfaceFormatKHR {
            format: vk::Format::B8G8R8A8_UNORM,
            color_space: vk::ColorSpaceKHR::SRGB_NONLINEAR,
        });

    let surface_capabilities =
        unsafe { surface_ext.get_physical_device_surface_capabilities(physical_device, surface)? };

    let present_modes =
        unsafe { surface_ext.get_physical_device_surface_present_modes(physical_device, surface)? };

    for mode in &present_modes {
        println!("present mode: {mode:?}");
    }

    let chosen_present_mode = present_modes
        .iter()
        .cloned()
        .find(|&m| m == info.preferred_present_mode)
        .unwrap_or(vk::PresentModeKHR::FIFO);

    let swapchain_info = vk::SwapchainCreateInfoKHR::default()
        .surface(surface)
        .image_format(chosen_format.format)
        .image_color_space(chosen_format.color_space)
        .pre_transform(surface_capabilities.current_transform)
        .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
        .image_extent(surface_capabilities.current_extent)
        .min_image_count(surface_capabilities.min_image_count)
        .image_array_layers(1)
        .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
        .present_mode(chosen_present_mode)
        .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT);

    let swapchain_info = if let Some(old) = info.old_swapchain {
        swapchain_info.old_swapchain(old)
    } else {
        swapchain_info
    };

    let swapchain_ext = khr::swapchain::Device::new(instance, device);
    let swapchain = unsafe { swapchain_ext.create_swapchain(&swapchain_info, None)? };

    let images = unsafe { swapchain_ext.get_swapchain_images(swapchain)? };
    let image_views = images
        .iter()
        .cloned()
        .map(|image| {
            let image_view_info = vk::ImageViewCreateInfo::default()
                .image(image)
                .format(chosen_format.format)
                .components(
                    vk::ComponentMapping::default()
                        .r(vk::ComponentSwizzle::IDENTITY)
                        .g(vk::ComponentSwizzle::IDENTITY)
                        .b(vk::ComponentSwizzle::IDENTITY),
                )
                .view_type(vk::ImageViewType::TYPE_2D)
                .subresource_range(
                    vk::ImageSubresourceRange::default()
                        .aspect_mask(vk::ImageAspectFlags::COLOR)
                        .base_array_layer(0)
                        .base_mip_level(0)
                        .layer_count(1)
                        .level_count(1),
                );

            let view = unsafe { device.create_image_view(&image_view_info, None)? };

            Ok(view)
        })
        .collect::<Result<Vec<_>, Error>>()?;

    Ok((
        swapchain,
        swapchain_ext,
        chosen_format.format,
        images,
        image_views,
    ))
}
