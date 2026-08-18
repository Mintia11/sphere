use std::sync::Arc;

use ash::{khr, vk};

use crate::{error::Error, image::Image, instance::Instance, surface::Surface};

pub struct Swapchain {
    ext: khr::swapchain::Device,
    device: Arc<ash::Device>,
    surface: Arc<Surface>,
    physical_device: vk::PhysicalDevice,

    swapchain: Option<vk::SwapchainKHR>,
    format: Option<vk::Format>,

    images: Vec<Arc<Image>>,
    current_idx: u32,

    preferred_format: vk::Format,
    preferred_colorspace: vk::ColorSpaceKHR,
    preferred_present_mode: vk::PresentModeKHR,
}

pub struct SwapchainCreateInfo<'a> {
    pub instance: &'a Instance,
    pub device: Arc<ash::Device>,
    pub surface: Arc<Surface>,
    pub physical_device: vk::PhysicalDevice,
    pub preferred_format: vk::Format,
    pub preferred_colorspace: vk::ColorSpaceKHR,
    pub preferred_present_mode: vk::PresentModeKHR,
}

impl Swapchain {
    pub fn new(info: SwapchainCreateInfo<'_>) -> Result<Self, Error> {
        let mut this = Self {
            ext: khr::swapchain::Device::new(info.instance.handle(), &info.device),
            device: info.device,
            surface: info.surface,
            physical_device: info.physical_device,
            swapchain: None,
            format: None,
            images: Vec::with_capacity(2),
            current_idx: 0,

            preferred_format: info.preferred_format,
            preferred_colorspace: info.preferred_colorspace,
            preferred_present_mode: info.preferred_present_mode,
        };

        this.recreate()?;

        Ok(this)
    }

    pub fn acquire_image(
        &mut self,
        timeout: u64,
        semaphore: Option<vk::Semaphore>,
        fence: Option<vk::Fence>,
    ) -> Result<Arc<Image>, Error> {
        if self.swapchain.is_none() {
            self.recreate()?;
        }

        let acquire_info = vk::AcquireNextImageInfoKHR::default()
            .fence(fence.unwrap_or_else(vk::Fence::null))
            .semaphore(semaphore.unwrap_or_else(vk::Semaphore::null))
            .swapchain(self.swapchain.unwrap())
            .timeout(timeout);

        let (idx, is_suboptimal) = unsafe { self.ext.acquire_next_image2(&acquire_info)? };
        if is_suboptimal {
            self.recreate()?;
            return self.acquire_image(timeout, semaphore, fence);
        }

        self.current_idx = idx;
        Ok(self.images[idx as usize].clone())
    }

    pub fn present(&mut self, queue: vk::Queue, semaphore: vk::Semaphore) -> Result<(), Error> {
        let swapchain = self
            .swapchain
            .as_ref()
            .ok_or(Error::PresentWithoutSwapchain)?;

        let mut results = [vk::Result::default()];
        let present_info = vk::PresentInfoKHR::default()
            .image_indices(std::slice::from_ref(&self.current_idx))
            .results(&mut results)
            .swapchains(std::slice::from_ref(swapchain))
            .wait_semaphores(std::slice::from_ref(&semaphore));

        let is_suboptimal = unsafe { self.ext.queue_present(queue, &present_info)? };

        if results[0] != vk::Result::SUCCESS {
            return Err(results[0].into());
        }

        if is_suboptimal {
            self.recreate()?;
        }

        Ok(())
    }

    pub fn change_preferred_format(
        &mut self,
        format: vk::Format,
        colorspace: vk::ColorSpaceKHR,
    ) -> Result<(), Error> {
        self.preferred_format = format;
        self.preferred_colorspace = colorspace;

        self.recreate()
    }

    fn recreate(&mut self) -> Result<(), Error> {
        let formats = self.surface.available_formats(self.physical_device)?;
        let chosen_format = formats
            .iter()
            .cloned()
            .find(|f| {
                f.format == self.preferred_format && f.color_space == self.preferred_colorspace
            })
            .unwrap_or(vk::SurfaceFormatKHR {
                format: vk::Format::B8G8R8A8_UNORM,
                color_space: vk::ColorSpaceKHR::SRGB_NONLINEAR,
            });

        let present_modes = self.surface.present_modes(self.physical_device)?;
        let chosen_present_mode = present_modes
            .iter()
            .cloned()
            .find(|&m| m == self.preferred_present_mode)
            .unwrap_or(vk::PresentModeKHR::FIFO);

        let surface_capabilities = self.surface.capabilities(self.physical_device)?;

        let swapchain_info = vk::SwapchainCreateInfoKHR::default()
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .image_array_layers(1)
            .image_color_space(chosen_format.color_space)
            .image_extent(surface_capabilities.current_extent)
            .image_format(chosen_format.format)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .pre_transform(surface_capabilities.current_transform)
            .present_mode(chosen_present_mode)
            .min_image_count(surface_capabilities.min_image_count)
            .surface(self.surface.handle());

        let swapchain_info = if let Some(old) = self.swapchain {
            swapchain_info.old_swapchain(old)
        } else {
            swapchain_info
        };

        let swapchain = unsafe { self.ext.create_swapchain(&swapchain_info, None)? };
        self.swapchain = Some(swapchain);
        self.format = Some(chosen_format.format);

        let raw_images = unsafe { self.ext.get_swapchain_images(swapchain)? };
        let images = raw_images
            .into_iter()
            .map(|image| {
                let image_view_info = vk::ImageViewCreateInfo::default()
                    .components(
                        vk::ComponentMapping::default()
                            .r(vk::ComponentSwizzle::IDENTITY)
                            .g(vk::ComponentSwizzle::IDENTITY)
                            .b(vk::ComponentSwizzle::IDENTITY),
                    )
                    .format(chosen_format.format)
                    .image(image)
                    .subresource_range(
                        vk::ImageSubresourceRange::default()
                            .aspect_mask(vk::ImageAspectFlags::COLOR)
                            .base_array_layer(0)
                            .base_mip_level(0)
                            .layer_count(1)
                            .level_count(1),
                    )
                    .view_type(vk::ImageViewType::TYPE_2D);

                let view = unsafe { self.device.create_image_view(&image_view_info, None)? };

                Ok(Arc::new(Image::from_parts_without_allocation(
                    image,
                    view,
                    self.device.clone(),
                )))
            })
            .collect::<Result<Vec<_>, Error>>()?;

        self.images = images;

        Ok(())
    }
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        unsafe {
            if let Some(swapchain) = self.swapchain {
                self.ext.destroy_swapchain(swapchain, None);
            }
        }
    }
}
