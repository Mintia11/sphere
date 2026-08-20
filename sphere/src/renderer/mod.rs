use std::sync::Arc;

use etna::{
    Device, Instance, Surface, Swapchain,
    ash::{ext, khr},
    command_buffer::CommandBuffer,
    error::Error,
    swapchain::SwapchainCreateInfo,
    sync::{Fence, Semaphore},
    vk,
};
use winit::raw_window_handle::RawWindowHandle;

pub struct Renderer {
    _instance: Instance,
    pub device: Arc<Device>,
    surface: Arc<Surface>,
    pub swapchain: Swapchain,

    frames: Vec<FrameData>,
    frame_idx: usize,
}

struct FrameData {
    image_available: Semaphore,
    render_finished: Semaphore,
    inflight: Fence,
    command_buffer: CommandBuffer,
}

impl FrameData {
    pub fn new(device: &Arc<Device>) -> Result<Self, Error> {
        let image_available = device.create_semaphore()?;
        let render_finished = device.create_semaphore()?;
        let inflight = device.create_fence(true)?;
        let command_buffer = device.allocate_graphics_command_buffer()?;

        Ok(FrameData {
            image_available,
            render_finished,
            inflight,
            command_buffer,
        })
    }
}

impl Renderer {
    pub fn new(window_handle: RawWindowHandle) -> Result<Self, Error> {
        let instance = Instance::new()?;
        let surface = Surface::new(&instance, window_handle)?;
        let device = Device::new(
            &instance,
            &[
                khr::swapchain::NAME,
                ext::descriptor_heap::NAME,
                khr::maintenance5::NAME,
                ext::pageable_device_local_memory::NAME,
                ext::memory_priority::NAME,
            ],
        )?;
        let swapchain = Swapchain::new(SwapchainCreateInfo {
            instance: &instance,
            device: device.clone(),
            surface: surface.clone(),
            preferred_format: vk::Format::R8G8B8A8_SNORM,
            preferred_colorspace: vk::ColorSpaceKHR::SRGB_NONLINEAR,
            preferred_present_mode: vk::PresentModeKHR::MAILBOX,
        })?;

        let mut frames = Vec::with_capacity(swapchain.image_count());
        for _ in 0..swapchain.image_count() {
            frames.push(FrameData::new(&device)?);
        }

        Ok(Self {
            _instance: instance,
            device,
            surface,
            swapchain,
            frames,
            frame_idx: 0,
        })
    }

    pub fn draw(&mut self) -> Result<(), Error> {
        self.frames[self.frame_idx].inflight.wait(u64::MAX)?;
        self.frames[self.frame_idx].inflight.reset()?;

        let image = self.swapchain.acquire_image(
            u64::MAX,
            Some(&self.frames[self.frame_idx].image_available),
            None,
        )?;

        let command_buffer = &self.frames[self.frame_idx].command_buffer;
        command_buffer.reset()?;

        command_buffer.begin()?;
        command_buffer.image_pipeline_barrier(
            &image,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            vk::AccessFlags2::NONE,
            vk::AccessFlags2::TRANSFER_WRITE,
            vk::PipelineStageFlags2::ALL_COMMANDS,
            vk::PipelineStageFlags2::ALL_TRANSFER,
        );
        unsafe {
            self.device.handle().cmd_clear_color_image(
                command_buffer.handle(),
                image.handle(),
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &vk::ClearColorValue {
                    float32: [1.0, 0.0, 0.0, 0.0],
                },
                &[vk::ImageSubresourceRange::default()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .base_array_layer(0)
                    .base_mip_level(0)
                    .layer_count(1)
                    .level_count(1)],
            )
        };
        command_buffer.image_pipeline_barrier(
            &image,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            vk::ImageLayout::PRESENT_SRC_KHR,
            vk::AccessFlags2::TRANSFER_WRITE,
            vk::AccessFlags2::NONE,
            vk::PipelineStageFlags2::ALL_TRANSFER,
            vk::PipelineStageFlags2::TOP_OF_PIPE,
        );
        command_buffer.end()?;

        self.device.submit_queue(
            self.device.graphics_queue(),
            command_buffer,
            Some(&self.frames[self.frame_idx].image_available),
            Some(&self.frames[self.frame_idx].render_finished),
            Some(&self.frames[self.frame_idx].inflight),
        )?;

        self.swapchain.present(
            self.device.graphics_queue(),
            &self.frames[self.frame_idx].render_finished,
        )?;

        self.frame_idx = (self.frame_idx + 1) % self.swapchain.image_count();

        Ok(())
    }
}
