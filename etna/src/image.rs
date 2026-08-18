use std::sync::Arc;

use ash::vk;
use gpu_allocator::vulkan::Allocation;

pub struct Image {
    image: vk::Image,
    view: vk::ImageView,

    device: Arc<ash::Device>,
    allocation: Option<Allocation>,
}

impl Image {
    pub(crate) fn from_parts_without_allocation(
        image: vk::Image,
        view: vk::ImageView,
        device: Arc<ash::Device>,
    ) -> Image {
        Self {
            image,
            view,

            device,
            allocation: None,
        }
    }
}

impl Drop for Image {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_image_view(self.view, None);
            self.device.destroy_image(self.image, None);
        }
    }
}
