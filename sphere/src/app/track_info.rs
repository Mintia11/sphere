use egui::Ui;

use crate::app::playback::Playback;

pub fn window(ui: &mut Ui, playback: &Playback) {
    egui::Window::new("Track Info").show(ui.ctx(), |ui| match playback.demuxer() {
        Some(demuxer) => {
            for track in demuxer.tracks() {
                ui.label(format!("Track {}", track.id));
                ui.label(format!("\tKind: {:?}", track.kind));
                ui.label(format!("\tCodec: {:?}", track.codec));

                if let Some(decoder) = playback.decoders().get(&track.id) {
                    for info_string in decoder.info_strings() {
                        ui.label(format!("\t{}", info_string));
                    }
                }
            }
        }
        None => {
            ui.label("No file loaded");
        }
    });
}
