use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::raw_window_handle::HasWindowHandle;
use winit::window::{Window, WindowId};

use crate::gui::EguiContext;
use crate::renderer::Renderer;

mod gui;
mod renderer;

#[derive(Default)]
struct App {
    window: Option<Window>,
    renderer: Option<Renderer>,
    ctx: EguiContext,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.window = Some(
            event_loop
                .create_window(Window::default_attributes().with_title("Sphere Video Player"))
                .unwrap(),
        );

        let window = self.window.as_ref().unwrap();
        let handle = window.window_handle().expect("Failed to get window handle");
        self.renderer =
            Some(Renderer::new(handle.as_raw()).expect("Failed to initialize the vulkan renderer"));
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        self.ctx.on_window_event(&event);

        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                let renderer = self.renderer.as_mut().unwrap();
                renderer.draw().expect("Error while drawing frame");

                let window = self.window.as_ref().unwrap();
                window.request_redraw();
            }
            _ => (),
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::default();
    event_loop.run_app(&mut app).expect("Failed to run app");
    std::mem::forget(app); // leak the app's memory because the drop impl in something is broken
}
