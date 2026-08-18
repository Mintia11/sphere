use ash::{khr, vk};
use raw_window_handle::RawWindowHandle;

use crate::error::Error;

pub struct Surface {
    surface: vk::SurfaceKHR,
    ext: khr::surface::Instance,
}

impl Surface {
    pub fn new(
        entry: &ash::Entry,
        instance: &ash::Instance,
        raw_window_handle: RawWindowHandle,
    ) -> Result<Self, Error> {
        let surface = match raw_window_handle {
            RawWindowHandle::Win32(win32) => {
                let os_extension = khr::win32_surface::Instance::new(entry, instance);

                let create_info = vk::Win32SurfaceCreateInfoKHR::default()
                    .hwnd(win32.hwnd.get())
                    .hinstance(win32.hinstance.unwrap().get());

                unsafe { os_extension.create_win32_surface(&create_info, None)? }
            }
            _ => todo!("Unimplemented surface creation for: {raw_window_handle:?}"),
        };

        Ok(Surface {
            surface,
            ext: khr::surface::Instance::new(entry, instance),
        })
    }

    pub fn handle(&self) -> vk::SurfaceKHR {
        self.surface
    }

    pub fn available_formats(
        &self,
        physical_device: vk::PhysicalDevice,
    ) -> Result<Vec<vk::SurfaceFormatKHR>, vk::Result> {
        unsafe {
            self.ext
                .get_physical_device_surface_formats(physical_device, self.surface)
        }
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

    pub fn capabilities(
        &self,
        physical_device: vk::PhysicalDevice,
    ) -> Result<vk::SurfaceCapabilitiesKHR, vk::Result> {
        unsafe {
            self.ext
                .get_physical_device_surface_capabilities(physical_device, self.surface)
        }
    }
}
