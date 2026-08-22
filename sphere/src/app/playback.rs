use std::collections::HashMap;

use common::{
    demuxer::{Demuxer, DemuxingError},
    packet::{Error, PacketDecoder},
    time::Timestamp,
    track::{CodecId, TrackId},
};
use h264::H264Decoder;

use crate::renderer::Renderer;

#[derive(Default)]
pub struct Playback {
    demuxer: Option<Box<dyn Demuxer>>,
    decoders: HashMap<TrackId, Box<dyn PacketDecoder>>,
    pub playing: bool,
    pub current_pts: Timestamp,
    duration: Timestamp,
}

impl Playback {
    pub fn load(
        &mut self,
        demuxer: Box<dyn Demuxer>,
        renderer: &Renderer,
    ) -> Result<(), DemuxingError> {
        self.decoders.clear();

        for track in demuxer.tracks() {
            let decoder: Option<Box<dyn PacketDecoder>> = match track.codec {
                CodecId::H264 => Some(Box::new(H264Decoder::new(&renderer.device))),
                _ => None,
            };

            if let Some(mut decoder) = decoder {
                decoder
                    .track(track)
                    .expect("Failed to give track to decoder");

                if let Ok(true) = decoder.can_decode_track() {
                    self.decoders.insert(track.id, decoder);
                } else {
                    println!("Cannot decode track {} using selected decoder", track.id);
                }
            }
        }

        self.current_pts = Timestamp::default();
        self.duration = demuxer.duration()?;
        self.demuxer = Some(demuxer);

        Ok(())
    }

    pub fn toggle_play(&mut self) {
        self.playing = !self.playing;
        if self.playing {
            self.start_playing()
                .expect("Failed to start decode session");
        }
    }

    pub fn advance(&mut self) {
        if let Some(demuxer) = self.demuxer.as_deref_mut()
            && let Some(packet) = demuxer.read_packet().expect("Failed to read packet")
        {}
    }

    pub fn progress(&mut self) -> f32 {
        (self.current_pts.to_seconds() / self.duration.to_seconds()) as f32
    }

    pub fn demuxer(&self) -> Option<&dyn Demuxer> {
        self.demuxer.as_deref()
    }

    pub fn decoders(&self) -> &HashMap<TrackId, Box<dyn PacketDecoder>> {
        &self.decoders
    }

    fn start_playing(&mut self) -> Result<(), Error> {
        for decoder in self.decoders.values_mut() {
            decoder.start_decode_session()?;
        }

        Ok(())
    }
}
