use winit::event_loop::{ControlFlow, EventLoop};

use crate::app::App;

mod app;
mod audio;
mod gui;
mod renderer;

fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::default();
    event_loop.run_app(&mut app).expect("Failed to run app");
    std::mem::forget(app); // leak the app's memory because the drop impl in something is broken
}
