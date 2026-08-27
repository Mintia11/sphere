use std::sync::Arc;

use etna::{
    device::Device,
    instance::{Instance, InstanceCreateInfo},
    surface::Surface,
};
use log::LevelFilter;
use sdl3::{event::Event, keyboard::Keycode};

fn main() {
    pretty_env_logger::formatted_builder()
        .filter_level(LevelFilter::Trace)
        .init();

    let instance = Instance::new(InstanceCreateInfo { debug: true })
        .expect("Failed to intiailize vulkan instance");
    let instance = Arc::new(instance);

    let sdl_context = sdl3::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Sphere Video Player", 1366, 768)
        .position_centered()
        .vulkan()
        .build()
        .unwrap();

    let surface = Surface::new(&instance, &window).expect("Failed to create a surface");
    let physical_device = instance
        .pick_physical_device(Some(&surface))
        .expect("Failed to pick physical device");

    let device = Device::new(&instance, physical_device, Some(&surface))
        .expect("Failed to create logical device");
    let device = Arc::new(device);

    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }
    }
}
