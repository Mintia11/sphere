use std::sync::Arc;

use ash::{khr, vk};
use raw_window_handle::RawWindowHandle;

use crate::{error::Error, instance::Instance};

pub struct Surface {
    surface: vk::SurfaceKHR,
    ext: khr::surface::Instance,
    caps_ext: khr::get_surface_capabilities2::Instance,
}

impl Surface {
    pub fn new(
        instance: &Instance,
        raw_window_handle: RawWindowHandle,
    ) -> Result<Arc<Self>, Error> {
        let surface = match raw_window_handle {
            RawWindowHandle::Win32(win32) => {
                let os_extension =
                    khr::win32_surface::Instance::load(instance.entry(), instance.handle());

                let create_info = vk::Win32SurfaceCreateInfoKHR::default()
                    .hwnd(win32.hwnd.get())
                    .hinstance(win32.hinstance.unwrap().get());

                unsafe { os_extension.create_win32_surface(&create_info, None)? }
            }
            _ => todo!("Unimplemented surface creation for: {raw_window_handle:?}"),
        };

        Ok(Arc::new(Surface {
            surface,
            ext: khr::surface::Instance::load(instance.entry(), instance.handle()),
            caps_ext: khr::get_surface_capabilities2::Instance::load(
                instance.entry(),
                instance.handle(),
            ),
        }))
    }

    pub fn handle(&self) -> vk::SurfaceKHR {
        self.surface
    }

    pub fn available_formats(
        &self,
        physical_device: vk::PhysicalDevice,
    ) -> Result<Vec<vk::SurfaceFormatKHR>, vk::Result> {
        let surface_info = vk::PhysicalDeviceSurfaceInfo2KHR::default().surface(self.surface);
        let format_count = unsafe {
            self.caps_ext
                .get_physical_device_surface_formats2_len(physical_device, &surface_info)?
        };

        let mut formats = vec![vk::SurfaceFormat2KHR::default(); format_count];

        unsafe {
            self.caps_ext.get_physical_device_surface_formats2(
                physical_device,
                &surface_info,
                &mut formats,
            )?;
        }

        Ok(formats.iter().map(|f| f.surface_format).collect())
    }

    pub fn capabilities(
        &self,
        physical_device: vk::PhysicalDevice,
    ) -> Result<vk::SurfaceCapabilitiesKHR, vk::Result> {
        let surface_info = vk::PhysicalDeviceSurfaceInfo2KHR::default().surface(self.surface);

        let mut surface_capabilities = vk::SurfaceCapabilities2KHR::default();

        unsafe {
            self.caps_ext.get_physical_device_surface_capabilities2(
                physical_device,
                &surface_info,
                &mut surface_capabilities,
            )?;
        }

        Ok(surface_capabilities.surface_capabilities)
    }

    pub fn present_modes(
        &self,
        physical_device: vk::PhysicalDevice,
    ) -> Result<Vec<vk::PresentModeKHR>, vk::Result> {
        unsafe {
            self.ext
                .get_physical_device_surface_present_modes(physical_device, self.surface)
        }
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        unsafe { self.ext.destroy_surface(self.surface, None) };
    }
}
