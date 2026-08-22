use std::sync::Arc;

use egui::{ClippedPrimitive, Context, TexturesDelta, Ui};
use etna::{Device, Swapchain};
use winit::window::Window;

use crate::gui::{input::EguiInputState, render::EguiRenderer};

mod input;
mod render;

pub struct EguiContext {
    input: EguiInputState,
    pub renderer: EguiRenderer,
    ctx: Context,
}

pub struct RenderData {
    clipped_primitives: Vec<ClippedPrimitive>,
    textures_delta: TexturesDelta,
    screen_size: [f32; 2],
    pixels_per_point: f32,
}

impl EguiContext {
    pub fn new(device: &Arc<Device>, swapchain: &Swapchain) -> Self {
        EguiContext {
            input: EguiInputState::default(),
            renderer: EguiRenderer::new(device, swapchain)
                .expect("Failed to initialize egui renderer"),
            ctx: Context::default(),
        }
    }

    pub fn run_ui(&mut self, window: &Window, run_ui: impl FnMut(&mut Ui)) -> RenderData {
        let raw_input = self.input.take_raw_input(window);

        let full_output = self.ctx.run_ui(raw_input, run_ui);
        let clipped_primitives = self
            .ctx
            .tessellate(full_output.shapes, full_output.pixels_per_point);

        let size = window.inner_size();
        RenderData {
            clipped_primitives,
            textures_delta: full_output.textures_delta,
            screen_size: [size.width as f32, size.height as f32],
            pixels_per_point: full_output.pixels_per_point,
        }
    }

    pub fn on_window_event(&mut self, event: &winit::event::WindowEvent) {
        self.input.on_window_event(event);
    }
}
