use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::{
    HeapCons, HeapProd, HeapRb,
    traits::{Consumer, Producer, Split},
};

use crate::audio::clock::AudioClock;

pub mod clock;

pub struct AudioOutput {
    producer: HeapProd<f32>,
    clock: Arc<AudioClock>,
    stream: cpal::Stream,
}

impl AudioOutput {
    pub fn new(
        sample_rate: u32,
        channels: u16,
        clock: Arc<AudioClock>,
    ) -> Result<Self, AudioError> {
        let rb = HeapRb::<f32>::new(sample_rate as usize * channels as usize / 2);
        let (producer, mut consumer) = rb.split();

        let device = cpal::default_host()
            .default_output_device()
            .ok_or(AudioError::NoAudioDevice)?;
        let config = device.default_output_config()?.config();

        let clock_cb = clock.clone();
        let stream = device.build_output_stream(
            config,
            move |output: &mut [f32], _| {
                let filled = consumer.pop_slice(output);
                if filled < output.len() {
                    output[filled..].fill(0.0);
                }
                clock_cb.advance_by_samples(filled);
            },
            |err| eprintln!("audio stream error: {err}"),
            None,
        )?;

        stream.play()?;
        Ok(Self {
            producer,
            clock,
            stream,
        })
    }

    pub fn play(&self) -> Result<(), AudioError> {
        self.stream.play().map_err(Into::into)
    }
    pub fn pause(&self) -> Result<(), AudioError> {
        self.stream.pause().map_err(Into::into)
    }

    pub fn push_samples(&mut self, samples: &[f32]) -> usize {
        self.producer.push_slice(samples)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AudioError {
    #[error("no audio device")]
    NoAudioDevice,

    #[error("cpal error: {0}")]
    Cpal(#[from] cpal::Error),
}
