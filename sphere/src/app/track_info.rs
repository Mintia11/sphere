use common::demuxer::Demuxer;
use egui::Ui;

pub fn window(ui: &mut Ui, demuxer: Option<&dyn Demuxer>) {
    egui::Window::new("Track Info").show(ui.ctx(), |ui| match demuxer {
        Some(demuxer) => {
            for track in demuxer.tracks() {
                ui.label(format!("Track {}", track.id));
                ui.label(format!("\tKind: {:?}", track.kind));
                ui.label(format!("\tCodec: {:?}", track.codec));
            }
        }
        None => {
            ui.label("No file loaded");
        }
    });
}
