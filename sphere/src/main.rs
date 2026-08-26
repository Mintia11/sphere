use etna::instance::InstanceCreateInfo;
use log::LevelFilter;
use winit::event_loop::{ControlFlow, EventLoop};

fn main() {
    pretty_env_logger::formatted_builder()
        .filter_level(LevelFilter::Trace)
        .init();

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let instance = etna::instance::Instance::new(InstanceCreateInfo { debug: true });
}
