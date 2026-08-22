use std::collections::HashMap;

use common::demuxer::Demuxer;
use common::packet::PacketDecoder;
use common::track::{CodecId, TrackId};
use h264::H264Decoder;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::raw_window_handle::HasWindowHandle;
use winit::window::{Window, WindowId};

use crate::app::file_ops::open_demuxer;
use crate::app::menu_bar::MenuAction;
use crate::gui::EguiContext;
use crate::renderer::Renderer;

mod file_ops;
mod menu_bar;
mod track_info;

#[derive(Default)]
pub struct App {
    window: Option<Window>,
    renderer: Option<Renderer>,
    ctx: Option<EguiContext>,

    demuxer: Option<Box<dyn Demuxer>>,
    decoders: HashMap<TrackId, Box<dyn PacketDecoder>>,

    track_info_open: bool,
}

impl App {
    pub fn init_decoders(
        demuxer: &dyn Demuxer,
        decoders: &mut HashMap<TrackId, Box<dyn PacketDecoder>>,
        renderer: &Renderer,
    ) {
        decoders.clear();

        for track in demuxer.tracks() {
            let decoder: Option<Box<dyn PacketDecoder>> = match track.codec {
                CodecId::H264 => Some(Box::new(H264Decoder::new(&renderer.device))),
                _ => None,
            };

            if let Some(mut decoder) = decoder {
                decoder
                    .track(track)
                    .expect("Failed to give track to decoder");

                decoders.insert(track.id, decoder);
            }
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.window = Some(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("Sphere Video Player")
                        .with_inner_size(LogicalSize::new(1366.0, 768.0)),
                )
                .unwrap(),
        );

        let window = self.window.as_ref().unwrap();
        let handle = window.window_handle().expect("Failed to get window handle");
        self.renderer =
            Some(Renderer::new(handle.as_raw()).expect("Failed to initialize the vulkan renderer"));

        let renderer = self.renderer.as_ref().unwrap();
        self.ctx = Some(EguiContext::new(&renderer.device, &renderer.swapchain));

        let file = std::env::args().nth(1);
        if let Some(file) = file {
            self.demuxer = open_demuxer(file);
            if let Some(demuxer) = self.demuxer.as_deref() {
                Self::init_decoders(demuxer, &mut self.decoders, renderer);
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        self.ctx.as_mut().unwrap().on_window_event(&event);

        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::Resized(_) => {
                let renderer = self.renderer.as_mut().unwrap();
                renderer
                    .swapchain
                    .recreate()
                    .expect("Failed to recreate swapchain");
            }
            WindowEvent::RedrawRequested => {
                let window = self.window.as_ref().unwrap();
                let ctx = self.ctx.as_mut().unwrap();
                let renderer = self.renderer.as_mut().unwrap();

                let data = ctx.run_ui(self.window.as_ref().unwrap(), |ui| {
                    match menu_bar::menu_bar(ui) {
                        MenuAction::None => {}
                        MenuAction::OpenFile => {
                            let file = rfd::FileDialog::new()
                                .add_filter("Matroska files", &["mkv"])
                                .set_parent(window)
                                .pick_file();

                            if let Some(file) = file {
                                self.demuxer = open_demuxer(file);
                            }

                            if let Some(demuxer) = self.demuxer.as_deref() {
                                Self::init_decoders(demuxer, &mut self.decoders, renderer);
                            }
                        }
                        MenuAction::Quit => event_loop.exit(),
                        MenuAction::ToggleTrackInfo => self.track_info_open = !self.track_info_open,
                    }

                    egui::CentralPanel::default().show(ui, |ui| {
                        ui.label("Hello world!");

                        if self.track_info_open {
                            track_info::window(ui, self.demuxer.as_deref());
                        }
                    });
                });

                let (command_buffer, image) = renderer.begin().expect("Failed to begin rendering");
                ctx.renderer
                    .draw(command_buffer, &image, data)
                    .expect("Failed to render ui");
                renderer.end().expect("Failed to end rendering");

                window.request_redraw();
            }
            _ => (),
        }
    }
}
