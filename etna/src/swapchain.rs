use ash::vk;
use snafu::Snafu;

use crate::{device::Device, surface::Surface};

pub struct Swapchain {}

impl Swapchain {
    pub fn new(device: &Device, surface: &Surface) -> Result<Swapchain, SwapchainError> {
        todo!()
    }
}

#[derive(Debug, Snafu)]
pub enum SwapchainError {
    #[snafu(display("Vulkan error"))]
    Vulkan { source: vk::Result },
}
