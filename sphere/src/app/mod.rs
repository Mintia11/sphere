use egui::{Align, Layout, Widget};
use etna::vk;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::raw_window_handle::HasWindowHandle;
use winit::window::{Window, WindowId};

use crate::app::file_ops::open_demuxer;
use crate::app::menu_bar::MenuAction;
use crate::app::playback::Playback;
use crate::gui::EguiContext;
use crate::renderer::Renderer;

mod file_ops;
mod menu_bar;
mod playback;
mod track_info;

#[derive(Default)]
pub struct App {
    window: Option<Window>,
    renderer: Option<Renderer>,
    ctx: Option<EguiContext>,

    playback: Playback,

    track_info_open: bool,
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
            let mut demuxer = open_demuxer(file);
            if let Some(demuxer) = demuxer.take() {
                self.playback
                    .load(demuxer, renderer)
                    .expect("Failed to load playback");
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
                self.playback.advance();

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
                                let mut demuxer = open_demuxer(file);
                                if let Some(demuxer) = demuxer.take() {
                                    self.playback
                                        .load(demuxer, renderer)
                                        .expect("Failed to load playback");
                                }
                            }
                        }
                        MenuAction::Quit => event_loop.exit(),
                        MenuAction::ToggleTrackInfo => self.track_info_open = !self.track_info_open,
                    }

                    egui::CentralPanel::default().show(ui, |ui| {
                        if self.track_info_open {
                            track_info::window(ui, &self.playback);
                        }

                        ui.with_layout(Layout::left_to_right(Align::Max), |ui| {
                            if ui
                                .button(if self.playback.playing { "■" } else { ">" })
                                .clicked()
                            {
                                self.playback.toggle_play();
                            }
                            let pts = self.playback.current_pts.to_seconds();
                            ui.label(format!(
                                "{}:{}:{}",
                                (pts as u64) / 3600,
                                ((pts as u64) / 60) % 60,
                                (pts as u64) % 60
                            ));
                            egui::widgets::ProgressBar::new(self.playback.progress()).ui(ui);
                        })
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
