use std::{
    collections::BTreeMap,
    ffi::{CStr, c_void},
    sync::Arc,
};

use ash::{
    Entry, ext, khr,
    vk::{self, QueueFlags, TaggedStructure},
};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use snafu::{ResultExt, Snafu};

use crate::{device::Device, surface::Surface};

pub struct Instance {
    pub entry: Entry,
    pub instance: ash::Instance,

    debug_utils_ext: Option<ext::debug_utils::Instance>,
    debug_messenger: Option<vk::DebugUtilsMessengerEXT>,
}

#[derive(Default, Debug)]
pub struct InstanceCreateInfo {
    pub debug: bool,
}

const INSTANCE_EXTENSIONS: &[&CStr] = &[
    khr::surface::NAME,
    ext::swapchain_colorspace::NAME,
    khr::get_surface_capabilities2::NAME,
    khr::surface_maintenance1::NAME,
    ext::surface_maintenance1::NAME,
    #[cfg(windows)]
    khr::win32_surface::NAME,
];

const DEVICE_EXTENSIONS: &[&CStr] = &[
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
    khr::dynamic_rendering::NAME,
    khr::video_maintenance1::NAME,
    khr::video_maintenance2::NAME,
];

unsafe extern "system" fn vulkan_debug_callback(
    sev: vk::DebugUtilsMessageSeverityFlagsEXT,
    typ: vk::DebugUtilsMessageTypeFlagsEXT,
    data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _: *mut c_void,
) -> vk::Bool32 {
    macro_rules! log_sev {
        ($sev:expr, $($t:tt)*) => {
            match $sev {
                vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => log::error!($($t)*),
                vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => log::warn!($($t)*),
                vk::DebugUtilsMessageSeverityFlagsEXT::INFO => log::debug!($($t)*),
                vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => log::trace!($($t)*),
                _ => unreachable!()
            }
        };
    }

    let data = unsafe { *data };
    if let Some(message) = unsafe { data.message_as_c_str() } {
        log_sev!(sev, "vulkan | {}", message.to_string_lossy());
    }

    let is_error = sev.contains(vk::DebugUtilsMessageSeverityFlagsEXT::ERROR)
        && typ.contains(vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION);
    is_error as vk::Bool32
}

impl Instance {
    #[profiling::function]
    pub fn new(info: InstanceCreateInfo) -> Result<Self, InstanceError> {
        let entry = unsafe { Entry::load().context(LoadingSnafu)? };

        log::debug!("Creating vulkan instance with: {info:?}");

        let api_version = unsafe { entry.try_enumerate_instance_version() }
            .context(VulkanSnafu)?
            .unwrap_or(vk::API_VERSION_1_0);
        log::debug!(
            "Running on instance version: v{}.{}.{}",
            vk::api_version_major(api_version),
            vk::api_version_minor(api_version),
            vk::api_version_patch(api_version)
        );

        let application_info = vk::ApplicationInfo::default().api_version(api_version);
        let mut instance_info =
            vk::InstanceCreateInfo::default().application_info(&application_info);

        let layers = unsafe { entry.enumerate_instance_layer_properties() }.context(VulkanSnafu)?;
        log::debug!("Available layers:");
        for layer in &layers {
            let major = vk::api_version_major(layer.spec_version);
            let minor = vk::api_version_minor(layer.spec_version);
            let patch = vk::api_version_patch(layer.spec_version);
            let name = layer
                .layer_name_as_c_str()
                .context(FromBytesUntilNullSnafu)?
                .to_string_lossy();

            log::debug!("    {name}: v{major}.{minor}.{patch}",);
        }

        let mut enabled_layers = Vec::new();

        if info.debug {
            let mut debug_layer_found = false;

            for layer in &layers {
                let major = vk::api_version_major(layer.spec_version);
                let minor = vk::api_version_minor(layer.spec_version);
                let patch = vk::api_version_patch(layer.spec_version);

                let name = layer
                    .layer_name_as_c_str()
                    .context(FromBytesUntilNullSnafu)?;

                if name == c"VK_LAYER_KHRONOS_validation" {
                    log::debug!(
                        "Enabling debug layer: {}: v{major}.{minor}.{patch}",
                        name.to_string_lossy()
                    );
                    debug_layer_found = true;

                    enabled_layers.push(name);
                }
            }

            if !debug_layer_found {
                log::warn!("Debug layer was requested but couldn't be found");
            }
        }

        let mut available_extensions: BTreeMap<String, Option<String>> = BTreeMap::new();

        let global_exts =
            unsafe { entry.enumerate_instance_extension_properties(None) }.context(VulkanSnafu)?;

        for ext in &global_exts {
            if let Ok(name) = ext.extension_name_as_c_str() {
                available_extensions.insert(name.to_string_lossy().to_string(), None);
            }
        }

        for layer in &layers {
            if let Ok(layer_name) = layer.layer_name_as_c_str() {
                if let Ok(layer_exts) =
                    unsafe { entry.enumerate_instance_extension_properties(Some(layer_name)) }
                {
                    for ext in &layer_exts {
                        if let Ok(ext_name) = ext.extension_name_as_c_str() {
                            available_extensions
                                .entry(ext_name.to_string_lossy().to_string())
                                .or_insert(Some(layer_name.to_string_lossy().to_string()));
                        }
                    }
                }
            }
        }

        log::debug!("Available instance extensions:");
        for (ext, layer_name) in &available_extensions {
            match layer_name {
                Some(layer_name) => log::debug!("    {ext} (via {layer_name})"),
                None => log::debug!("    {ext}"),
            }
        }

        let mut enabled_extensions = Vec::new();
        for &ext in INSTANCE_EXTENSIONS {
            let name: &str = &ext.to_string_lossy();
            if available_extensions.contains_key(name) {
                enabled_extensions.push(ext);
            }
        }

        let mut debug_info = vk::DebugUtilsMessengerCreateInfoEXT::default()
            .message_severity(
                vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                    | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                    | vk::DebugUtilsMessageSeverityFlagsEXT::INFO
                    | vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE,
            )
            .message_type(
                vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                    | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
                    | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
            )
            .pfn_user_callback(Some(vulkan_debug_callback));

        macro_rules! enable_layer_setting {
            ($name:literal) => {
                vk::LayerSettingEXT::default()
                    .layer_name(c"VK_LAYER_KHRONOS_validation")
                    .setting_name($name)
                    .ty(vk::LayerSettingTypeEXT::BOOL32)
                    .values({
                        static VALUE: &[u8] = &[1];
                        VALUE
                    })
            };
        }

        let debug_options = [
            enable_layer_setting!(c"validate_best_practices"),
            enable_layer_setting!(c"validate_sync"),
            enable_layer_setting!(c"syncval_shader_accesses_heuristic"),
            enable_layer_setting!(c"syncval_submit_time_validation"),
        ];
        let mut debug_layer_settings =
            vk::LayerSettingsCreateInfoEXT::default().settings(&debug_options);

        if info.debug {
            if available_extensions.contains_key("VK_EXT_debug_utils") {
                enabled_extensions.push(ext::debug_utils::NAME);
                instance_info = instance_info.push(&mut debug_info);
            } else {
                log::warn!("Validation layers enabled but no debug utils extension found");
            }

            if available_extensions.contains_key("VK_EXT_layer_settings") {
                enabled_extensions.push(ext::layer_settings::NAME);
                instance_info = instance_info.push(&mut debug_layer_settings);
            } else {
                log::warn!("Validation layers enabled but extra options won't be applied");
            }
        }

        log::debug!("Creating vulkan instance");
        log::debug!("    with extensions:");
        for ext in &enabled_extensions {
            log::debug!("        {}", ext.to_string_lossy());
        }
        log::debug!("    with layers:");
        for layer in &enabled_layers {
            log::debug!("        {}", layer.to_string_lossy());
        }

        let enabled_extensions: Vec<_> = enabled_extensions.iter().map(|e| e.as_ptr()).collect();
        let enabled_layers: Vec<_> = enabled_layers.iter().map(|e| e.as_ptr()).collect();

        instance_info = instance_info
            .enabled_extension_names(&enabled_extensions)
            .enabled_layer_names(&enabled_layers);

        let instance =
            unsafe { entry.create_instance(&instance_info, None) }.context(VulkanSnafu)?;

        let mut debug_utils_ext = None;
        let mut debug_messenger = None;

        if info.debug {
            let debug_utils = ext::debug_utils::Instance::load(&entry, &instance);

            debug_messenger = Some(
                unsafe { debug_utils.create_debug_utils_messenger(&debug_info, None) }
                    .context(VulkanSnafu)?,
            );
            debug_utils_ext = Some(debug_utils);
        }

        Ok(Instance {
            entry,
            instance,
            debug_utils_ext,
            debug_messenger,
        })
    }

    #[profiling::function]
    pub fn create_surface(
        self: &Arc<Self>,
        window: &impl HasWindowHandle,
    ) -> Result<Surface, InstanceError> {
        let handle = window
            .window_handle()
            .map_err(|_| InstanceError::WindowHandle)?;
        let raw = handle.as_raw();

        let handle = match raw {
            RawWindowHandle::Win32(raw) => {
                let create_info = vk::Win32SurfaceCreateInfoKHR::default()
                    .hinstance(raw.hinstance.unwrap().get())
                    .hwnd(raw.hwnd.get());
                let ext = khr::win32_surface::Instance::load(&self.entry, &self.instance);

                unsafe { ext.create_win32_surface(&create_info, None) }.context(VulkanSnafu)?
            }
            x => todo!("unimplemented platform handle: {x:?}"),
        };
        let ext = khr::surface::Instance::load(&self.entry, &self.instance);

        Ok(Surface {
            handle,
            ext,
            _instance: self.clone(),
        })
    }

    #[profiling::function]
    pub fn pick_physical_device(
        &self,
        surface: Option<&Surface>,
    ) -> Result<vk::PhysicalDevice, InstanceError> {
        let devices = unsafe { self.instance.enumerate_physical_devices() }.context(VulkanSnafu)?;

        log::info!("Picking vulkan device:");
        for (i, &physical_device) in devices.iter().enumerate() {
            let mut prop = vk::PhysicalDeviceProperties2::default();
            unsafe {
                self.instance
                    .get_physical_device_properties2(physical_device, &mut prop);
            }

            let name = prop
                .properties
                .device_name_as_c_str()
                .context(FromBytesUntilNullSnafu)?;

            log::info!(
                "    GPU {i}: {} v{}.{}.{} ({:?})",
                name.to_string_lossy(),
                vk::api_version_major(prop.properties.api_version),
                vk::api_version_minor(prop.properties.api_version),
                vk::api_version_patch(prop.properties.api_version),
                prop.properties.device_type,
            );

            let queue_family_count = unsafe {
                self.instance
                    .get_physical_device_queue_family_properties2_len(physical_device)
            };

            if let Some(surface) = surface {
                let mut supports_present = false;
                for idx in 0..(queue_family_count as u32) {
                    if unsafe {
                        surface.ext.get_physical_device_surface_support(
                            physical_device,
                            idx,
                            surface.handle,
                        )
                    }
                    .context(VulkanSnafu)?
                    {
                        supports_present = true;
                    }
                }

                if !supports_present {
                    continue;
                }
            }

            let mut internally_synchronized_queues =
                vk::PhysicalDeviceInternallySynchronizedQueuesFeaturesKHR::default();
            let mut swapchain_maintenance1 =
                vk::PhysicalDeviceSwapchainMaintenance1FeaturesKHR::default();
            let mut video_maintenance1 = vk::PhysicalDeviceVideoMaintenance1FeaturesKHR::default();
            let mut video_maintenance2 = vk::PhysicalDeviceVideoMaintenance2FeaturesKHR::default();
            let mut vulkan_11 = vk::PhysicalDeviceVulkan11Features::default();
            let mut vulkan_12 = vk::PhysicalDeviceVulkan12Features::default();
            let mut vulkan_13 = vk::PhysicalDeviceVulkan13Features::default();
            let mut shader_object = vk::PhysicalDeviceShaderObjectFeaturesEXT::default();
            let mut descriptor_heap = vk::PhysicalDeviceDescriptorHeapFeaturesEXT::default();
            let mut features = vk::PhysicalDeviceFeatures2::default()
                .push(&mut descriptor_heap)
                .push(&mut shader_object)
                .push(&mut vulkan_13)
                .push(&mut vulkan_12)
                .push(&mut vulkan_11)
                .push(&mut video_maintenance1)
                .push(&mut video_maintenance2)
                .push(&mut swapchain_maintenance1)
                .push(&mut internally_synchronized_queues);

            unsafe {
                self.instance
                    .get_physical_device_features2(physical_device, &mut features);
            }

            if descriptor_heap.descriptor_heap == 0
                || shader_object.shader_object == 0
                || vulkan_13.dynamic_rendering == 0
                || vulkan_13.synchronization2 == 0
                || vulkan_12.timeline_semaphore == 0
                || vulkan_11.sampler_ycbcr_conversion == 0
                || video_maintenance2.video_maintenance2 == 0
                || video_maintenance1.video_maintenance1 == 0
                || swapchain_maintenance1.swapchain_maintenance1 == 0
                || internally_synchronized_queues.internally_synchronized_queues == 0
            {
                log::debug!("    Skipping GPU {i}: Required feature(s) missing");
                continue;
            }

            let supported_exts = unsafe {
                self.instance
                    .enumerate_device_extension_properties(physical_device)
            }
            .context(VulkanSnafu)?;

            let has_ext = |ext_name: &std::ffi::CStr| {
                supported_exts
                    .iter()
                    .any(|e| e.extension_name_as_c_str().ok() == Some(ext_name))
            };

            if DEVICE_EXTENSIONS.iter().any(|ext| !has_ext(ext)) {
                log::debug!("    Skipping GPU {i}: Required extension(s) missing");
                continue;
            }

            return Ok(physical_device);
        }

        log::error!("Failed to find a suitable device!");
        snafu::whatever!("Failed to find a suitable device")
    }

    #[profiling::function]
    pub fn create_device(
        self: &Arc<Self>,
        physical_device: vk::PhysicalDevice,
        surface: Option<&Surface>,
    ) -> Result<Device, InstanceError> {
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
            self.instance
                .get_physical_device_queue_family_properties2_len(physical_device)
        };
        let mut video_infos =
            vec![vk::QueueFamilyVideoPropertiesKHR::default(); queue_family_count];
        let mut queue_families: Vec<_> = video_infos
            .iter_mut()
            .map(|video_info| vk::QueueFamilyProperties2::default().push(video_info))
            .collect();

        unsafe {
            self.instance
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

            if queue_family.queue_flags.contains(QueueFlags::GRAPHICS) && graphics_queue.is_none() {
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
            self.instance
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
        let graphics_queue = graphics_queue.ok_or_else(|| InstanceError::Whatever {
            message: "did not find a graphics capable queue".to_string(),
        })?;
        let decode_queue = decode_queue.ok_or_else(|| InstanceError::Whatever {
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
            self.instance
                .create_device(physical_device, &create_info, None)
        }
        .context(VulkanSnafu)?;

        Ok(Device {
            device,
            physical_device,

            _instance: self.clone(),
        })
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        unsafe {
            if let Some(debug_utils_ext) = &self.debug_utils_ext {
                if let Some(debug_messenger) = self.debug_messenger {
                    debug_utils_ext.destroy_debug_utils_messenger(debug_messenger, None);
                }
            }

            self.instance.destroy_instance(None);
        }
    }
}

#[derive(Debug, Snafu)]
pub enum InstanceError {
    #[snafu(display("Error while loading vulkan library"))]
    Loading { source: ash::LoadingError },

    #[snafu(display("Vulkan error"))]
    Vulkan { source: vk::Result },

    #[snafu(display("Error while reading UTF-8 string"))]
    FromBytesUntilNull {
        source: std::ffi::FromBytesUntilNulError,
    },

    #[snafu(display("Failed to get window handle"))]
    WindowHandle,

    #[snafu(whatever)]
    Whatever { message: String },
}
