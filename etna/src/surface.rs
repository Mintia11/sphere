use ash::{khr, vk};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use snafu::{ResultExt, Snafu};

use crate::{destroy::DestroyWithInstance, instance::Instance};

pub struct Surface {
    pub handle: vk::SurfaceKHR,
    pub ext: khr::surface::Instance,
}

impl Surface {
    #[profiling::function]
    pub fn new(instance: &Instance, window: &impl HasWindowHandle) -> Result<Self, SurfaceError> {
        let handle = window
            .window_handle()
            .map_err(|_| SurfaceError::WindowHandle)?;
        let raw = handle.as_raw();

        let handle = match raw {
            RawWindowHandle::Win32(raw) => {
                let create_info = vk::Win32SurfaceCreateInfoKHR::default()
                    .hinstance(raw.hinstance.unwrap().get())
                    .hwnd(raw.hwnd.get());
                let ext = khr::win32_surface::Instance::load(&instance.entry, &instance.instance);

                unsafe { ext.create_win32_surface(&create_info, None) }.context(VulkanSnafu)?
            }
            x => todo!("unimplemented platform handle: {x:?}"),
        };
        let ext = khr::surface::Instance::load(&instance.entry, &instance.instance);

        Ok(Surface { handle, ext })
    }
}

impl DestroyWithInstance for Surface {
    fn destroy(&mut self, _instance: &ash::Instance) {
        unsafe {
            self.ext.destroy_surface(self.handle, None);
        }
    }
}

#[derive(Debug, Snafu)]
pub enum SurfaceError {
    #[snafu(display("Vulkan error"))]
    Vulkan { source: vk::Result },

    #[snafu(display("Failed to get window handle"))]
    WindowHandle,
}
