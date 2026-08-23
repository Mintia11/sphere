use std::sync::Arc;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::{
    HeapProd, HeapRb,
    traits::{Consumer, Split},
};

use crate::audio::clock::AudioClock;

pub mod clock;

pub struct AudioOutput {
    stream: cpal::Stream,
    pub channel_count: u16,
}

impl AudioOutput {
    pub fn new(
        sample_rate: u32,
        clock: Arc<AudioClock>,
    ) -> Result<(Self, HeapProd<f32>), AudioError> {
        let device = cpal::default_host()
            .default_output_device()
            .ok_or(AudioError::NoAudioDevice)?;

        let channels = device.default_output_config().unwrap().channels();
        let rb = HeapRb::<f32>::new(sample_rate as usize * channels as usize / 2); // ~500ms
        let (producer, mut consumer) = rb.split();

        let config = cpal::StreamConfig {
            channels,
            sample_rate,
            buffer_size: cpal::BufferSize::Default,
        };

        let stream = device.build_output_stream(
            config,
            move |output: &mut [f32], _| {
                let filled = consumer.pop_slice(output);
                if filled < output.len() {
                    output[filled..].fill(0.0);
                }
                clock.advance_by_samples(filled / channels as usize);
            },
            |err| eprintln!("audio stream error: {err}"),
            None,
        )?;

        Ok((
            Self {
                stream,
                channel_count: config.channels,
            },
            producer,
        ))
    }

    pub fn play(&self) -> Result<(), AudioError> {
        self.stream.play().map_err(Into::into)
    }
    pub fn pause(&self) -> Result<(), AudioError> {
        self.stream.pause().map_err(Into::into)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AudioError {
    #[error("no audio device")]
    NoAudioDevice,

    #[error("cpal error: {0}")]
    Cpal(#[from] cpal::Error),
}
