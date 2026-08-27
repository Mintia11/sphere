use std::sync::Arc;

use ash::{khr, vk};

use crate::instance::Instance;

pub struct Surface {
    pub handle: vk::SurfaceKHR,
    pub ext: khr::surface::Instance,

    pub(crate) _instance: Arc<Instance>,
}

impl Drop for Surface {
    fn drop(&mut self) {
        unsafe {
            self.ext.destroy_surface(self.handle, None);
        }
    }
}
