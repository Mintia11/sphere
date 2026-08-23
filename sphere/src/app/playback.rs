use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::JoinHandle,
    time::Duration,
};

use aac::AacDecoder;
use common::{
    demuxer::{Demuxer, DemuxingError},
    packet::{Frame, Packet, PacketDecoder},
    time::Timestamp,
    track::{CodecId, TrackId, TrackInfo},
};
use etna::Image;
use h264::H264Decoder;

use crate::{
    audio::{AudioOutput, clock::AudioClock},
    renderer::Renderer,
};

pub struct Playback {
    pub playing: bool,
    pub current_pts: Timestamp,
    pub tracks: Vec<TrackInfo>,
    pub duration: Timestamp,
    track_info: Arc<Mutex<HashMap<TrackId, Vec<String>>>>,

    audio_clock: Option<Arc<AudioClock>>,
    audio_output: Option<AudioOutput>,

    video_frame_rx: Option<crossbeam_channel::Receiver<Frame>>,
    pending_frame: Option<(Image, Timestamp)>,
    current_frame: Option<Image>,

    decode_threads: Vec<(Arc<AtomicBool>, JoinHandle<()>)>,
}

impl Default for Playback {
    fn default() -> Self {
        Self {
            playing: false,
            current_pts: Timestamp::default(),
            duration: Timestamp::default(),
            tracks: Vec::new(),
            track_info: Arc::new(Mutex::new(HashMap::new())),

            audio_clock: None,
            audio_output: None,

            video_frame_rx: None,
            pending_frame: None,
            current_frame: None,

            decode_threads: Vec::new(),
        }
    }
}

impl Playback {
    pub fn load(
        &mut self,
        demuxer: Box<dyn Demuxer>,
        renderer: &Renderer,
    ) -> Result<(), DemuxingError> {
        self.stop_decode_threads();
        self.duration = demuxer.duration()?;
        self.current_pts = Timestamp::default();

        let mut decoders = HashMap::new();
        for track in demuxer.tracks() {
            self.tracks.push(track.clone());

            let decoder: Option<Box<dyn PacketDecoder>> = match track.codec {
                CodecId::Aac => Some(Box::new(AacDecoder::default())),
                CodecId::H264 => Some(Box::new(H264Decoder::new(&renderer.device))),
                _ => None,
            };

            if let Some(mut decoder) = decoder {
                decoder
                    .track(track)
                    .expect("Failed to give track to decoder");

                if let Ok(true) = decoder.can_decode_track() {
                    decoders.insert(track.id, decoder);
                } else {
                    println!("Cannot decode track {} using selected decoder", track.id);
                }
            }
        }

        let (packet_txs, mut per_track_rx): (HashMap<TrackId, _>, HashMap<TrackId, _>) = decoders
            .keys()
            .map(|&id| {
                let (tx, rx) = crossbeam_channel::bounded::<Packet>(32);
                ((id, tx), (id, rx))
            })
            .unzip();

        let shutdown_demux = Arc::new(AtomicBool::new(false));
        let demux_handle = {
            let shutdown = shutdown_demux.clone();
            let mut demuxer = demuxer;
            std::thread::spawn(move || {
                while !shutdown.load(Ordering::Relaxed) {
                    match demuxer.read_packet() {
                        Ok(Some(packet)) => {
                            if let Some(tx) = packet_txs.get(&packet.track)
                                && tx.send(packet).is_err()
                            {
                                break;
                            }
                        }
                        Ok(None) => break,
                        Err(e) => {
                            eprintln!("demux error: {e}");
                            break;
                        }
                    }
                }
            })
        };
        self.decode_threads.push((shutdown_demux, demux_handle));

        let (video_tx, video_rx) = crossbeam_channel::bounded(4);
        self.video_frame_rx = Some(video_rx);

        for (track_id, mut decoder) in decoders {
            let packet_rx = per_track_rx.remove(&track_id).unwrap();
            let track_info = self.track_info.clone();
            let shutdown = Arc::new(AtomicBool::new(false));
            let shutdown_clone = shutdown.clone();
            let video_tx = video_tx.clone();
            let audio_clock = self.audio_clock.clone();

            let handle = std::thread::spawn(move || {
                decoder
                    .start_decode_session()
                    .expect("failed to start decode session");
                track_info
                    .lock()
                    .unwrap()
                    .insert(track_id, decoder.info_strings());
                while !shutdown_clone.load(Ordering::Relaxed) {
                    let Ok(packet) = packet_rx.recv_timeout(Duration::from_millis(100)) else {
                        continue; // no packet yet, check shutdown flag again
                    };
                    if decoder.send_packet(packet).is_err() {
                        break;
                    }
                    while let Ok(Some(frame)) = decoder.grab_frame() {
                        match frame {
                            Frame::Video { .. } => {
                                let _ = video_tx.send(frame);
                            }
                            Frame::Audio { samples, .. } => {
                                // push into AudioOutput's ring buffer directly, or
                                // route through another channel if AudioOutput
                                // isn't Send/shareable across this closure
                            }
                        }
                    }
                }
            });
            self.decode_threads.push((shutdown, handle));
        }

        Ok(())
    }

    pub fn toggle_play(&mut self) {
        self.playing = !self.playing;
        if let Some(audio_output) = &self.audio_output {
            let result = if self.playing {
                audio_output.play()
            } else {
                audio_output.pause()
            };
            if let Err(e) = result {
                eprintln!("failed to toggle audio stream: {e}");
                self.playing = !self.playing;
            }
        }
    }

    pub fn advance(&mut self) {
        if !self.playing {
            return;
        }

        let Some(audio_clock) = &self.audio_clock else {
            return;
        };
        let clock_now = audio_clock.current_time();

        loop {
            let (image, pts) = match self.pending_frame.take() {
                Some(f) => f,
                None => match self.video_frame_rx.as_ref().unwrap().try_recv() {
                    Ok(f) => match f {
                        Frame::Video { image, pts } => (image, pts),
                        _ => unreachable!("expected video frame got another type of frame"),
                    },
                    Err(_) => break,
                },
            };

            let clock_in_pts_tb = clock_now.rescale(pts.timebase);
            let diff_seconds = (pts.value - clock_in_pts_tb.value) as f64 * pts.timebase.num as f64
                / pts.timebase.den as f64;

            if diff_seconds > 0.005 {
                self.pending_frame = Some((image, pts));
                break;
            } else if diff_seconds < -0.05 {
                continue;
            } else {
                self.current_frame = Some(image);
                self.current_pts = pts;
            }
        }
    }

    pub fn progress(&mut self) -> f32 {
        let Some(audio_clock) = &self.audio_clock else {
            return 0.0;
        };
        self.current_pts = audio_clock.current_time();

        let duration_secs = self.duration.to_seconds();
        if duration_secs <= 0.0 {
            return 0.0;
        }

        (self.current_pts.to_seconds() / duration_secs) as f32
    }

    pub fn track_info(&self, track_id: TrackId) -> Option<Vec<String>> {
        self.track_info.lock().unwrap().get(&track_id).cloned()
    }

    fn stop_decode_threads(&mut self) {
        for (flag, _) in &self.decode_threads {
            flag.store(true, Ordering::Relaxed);
        }
        for (_, handle) in self.decode_threads.drain(..) {
            let _ = handle.join();
        }
        self.track_info.lock().unwrap().clear();
    }
}

impl Drop for Playback {
    fn drop(&mut self) {
        self.stop_decode_threads();
    }
}
