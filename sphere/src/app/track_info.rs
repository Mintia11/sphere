use egui::Ui;

use crate::app::playback::Playback;

pub fn window(ui: &mut Ui, playback: &Playback) {
    egui::Window::new("Track Info").show(ui.ctx(), |ui| {
        for track in &playback.tracks {
            ui.label(format!("Track {}", track.id));
            ui.label(format!("\tKind: {:?}", track.kind));
            ui.label(format!("\tCodec: {:?}", track.codec));

            if let Some(info_strings) = playback.track_info(track.id) {
                for info_string in info_strings {
                    ui.label(format!("\t{}", info_string));
                }
            }
        }
    });
}
