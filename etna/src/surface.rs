use ash::{khr, vk};

pub struct Surface {
    pub handle: vk::SurfaceKHR,
    pub ext: khr::surface::Instance,
}

impl Drop for Surface {
    fn drop(&mut self) {
        unsafe {
            self.ext.destroy_surface(self.handle, None);
        }
    }
}
