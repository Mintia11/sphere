use std::collections::HashMap;

use common::{demuxer::Demuxer, packet::PacketDecoder, track::TrackId};
use egui::Ui;

pub fn window(
    ui: &mut Ui,
    demuxer: Option<&dyn Demuxer>,
    decoders: &HashMap<TrackId, Box<dyn PacketDecoder>>,
) {
    egui::Window::new("Track Info").show(ui.ctx(), |ui| match demuxer {
        Some(demuxer) => {
            for track in demuxer.tracks() {
                ui.label(format!("Track {}", track.id));
                ui.label(format!("\tKind: {:?}", track.kind));
                ui.label(format!("\tCodec: {:?}", track.codec));

                if let Some(decoder) = decoders.get(&track.id) {
                    for info_string in decoder.info_strings() {
                        ui.label(info_string);
                    }
                }
            }
        }
        None => {
            ui.label("No file loaded");
        }
    });
}
