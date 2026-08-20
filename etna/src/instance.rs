use ash::{ext, khr, vk};

use crate::error::Error;

pub struct Instance {
    entry: ash::Entry,
    instance: ash::Instance,
}

impl Instance {
    pub fn new() -> Result<Self, Error> {
        let entry = unsafe { ash::Entry::load()? };
        let instance_extensions = unsafe { entry.enumerate_instance_extension_properties(None)? };

        let mut used_extensions = vec![];
        for extension in &instance_extensions {
            let name = extension.extension_name_as_c_str().unwrap();

            if name == ext::swapchain_colorspace::NAME {
                used_extensions.push(ext::swapchain_colorspace::NAME.as_ptr());
            }

            #[cfg(debug_assertions)]
            if name == ext::debug_utils::NAME {
                used_extensions.push(ext::debug_utils::NAME.as_ptr());
            }
        }

        used_extensions.push(khr::surface::NAME.as_ptr());
        used_extensions.push(khr::get_surface_capabilities2::NAME.as_ptr());

        #[cfg(windows)]
        used_extensions.push(khr::win32_surface::NAME.as_ptr());

        let application_info = vk::ApplicationInfo::default().api_version(vk::API_VERSION_1_3);

        let instance_info = vk::InstanceCreateInfo::default()
            .application_info(&application_info)
            .enabled_extension_names(&used_extensions);

        let instance = unsafe { entry.create_instance(&instance_info, None)? };

        Ok(Instance { entry, instance })
    }

    pub fn entry(&self) -> &ash::Entry {
        &self.entry
    }

    pub fn handle(&self) -> &ash::Instance {
        &self.instance
    }
}
