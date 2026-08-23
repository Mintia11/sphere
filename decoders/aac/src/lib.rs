use std::{
    sync::{Arc, atomic::AtomicBool},
    thread::JoinHandle,
    time::Duration,
};

use common::{
    audio::{AudioBuffer, AudioInfo},
    bit_io::BitReader,
    byte_io::ByteReader,
    packet::{Error, Frame, Packet, PacketDecoder},
    track::TrackInfo,
};
use crossbeam_channel::{Receiver, Sender};

use crate::{bitstream::SyntaxElement, config::Config, dsp::run_dsp, ics::Ics};

mod bitstream;
mod config;
mod dsp;
mod ics;
mod tables;
mod window;

#[derive(Default)]
pub struct AacDecoder {
    config: Option<Config>,
    buffers: Option<Receiver<AudioBuffer>>,
    raw_data: Option<Sender<Vec<Ics>>>,
    dsp_thread: Option<(Arc<AtomicBool>, JoinHandle<()>)>,
}

impl PacketDecoder for AacDecoder {
    fn track(&mut self, track: &TrackInfo) -> Result<(), Error> {
        let private_data = track.extra_data.as_ref().ok_or(Error::InvalidData(
            "Track has no codec private data".to_string(),
        ))?;

        self.config = Some(Config::parse(private_data)?);

        Ok(())
    }

    fn info_strings(&self) -> Vec<String> {
        let mut info_strings = Vec::new();

        if let Some(config) = self.config {
            info_strings.push(format!("Object Type: {:?}", config.object_type));
            info_strings.push(format!("Sample Rate: {} Hz", config.sampling_frequency));
            info_strings.push(format!(
                "Channel Configuration: {}",
                config.channel_configuration
            ));
        }

        info_strings
    }

    fn can_decode_track(&self) -> Result<bool, Error> {
        Ok(self.config.is_some())
    }

    fn start_decode_session(&mut self) -> Result<(), Error> {
        let (raw_data, recv) = crossbeam_channel::bounded(32);
        let (send, buffers) = crossbeam_channel::bounded(32);

        let stop = Arc::new(AtomicBool::new(false));

        let stop_cb = stop.clone();
        let handle = std::thread::Builder::new()
            .name("aac dsp thread".to_string())
            .spawn(move || {
                run_dsp(stop_cb, recv, send);
            })
            .unwrap();

        self.dsp_thread = Some((stop, handle));
        self.buffers = Some(buffers);
        self.raw_data = Some(raw_data);

        Ok(())
    }

    fn send_packet(&mut self, packet: Packet) -> Result<(), Error> {
        let reader = ByteReader::new(packet.data);
        let mut reader = BitReader::new(reader);

        let syntax_elements = SyntaxElement::parse_all(&mut reader, self.config.as_ref().unwrap())?;
        for syntax_element in syntax_elements {
            match syntax_element {
                SyntaxElement::SingleChannel { ics } => {
                    if let Some(raw_data) = &self.raw_data {
                        raw_data.send(vec![ics]).unwrap();
                    }
                }
                SyntaxElement::Fill { extension } => {
                    let _ = extension;
                }
            }
        }

        Ok(())
    }

    fn grab_frame(&self) -> Result<Option<Frame>, Error> {
        if let Some(receiver) = &self.buffers {
            match receiver.try_recv() {
                Ok(samples) => Ok(Some(Frame::Audio { samples })),
                Err(crossbeam_channel::TryRecvError::Empty) => Ok(None),
                Err(crossbeam_channel::TryRecvError::Disconnected) => {
                    Err(Error::InvalidData("DSP thread disconnected".to_string()))
                }
            }
        } else {
            Ok(None)
        }
    }

    fn audio_info(&self) -> Option<AudioInfo> {
        let config = self.config?;

        Some(AudioInfo {
            channel_count: config.channel_configuration.channel_count(),
            sample_rate: config.sampling_frequency,
        })
    }
}
